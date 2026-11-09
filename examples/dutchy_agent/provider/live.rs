//! Native streaming HTTP with bounded delivery, cooperative cancellation and
//! an idle watchdog.
use super::{Change, Event, Request, protocol};
use ehttp::streaming::Part;
use std::{
    collections::BTreeMap,
    ops::ControlFlow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    time::Duration,
};

const ENDPOINT: &str = "https://api.openai.com/v1/responses";
/// Seconds without any bytes from OpenAI, before or after headers, after which
/// the example stops waiting. Silence, not total reply time, is measured.
pub(crate) const IDLE_LIMIT: f64 = 300.0;
/// ureq's body timeout is a total budget, not an idle limit, so it is set well
/// past any reply the output limit allows. It only ends reader threads that
/// the watchdog has already abandoned.
const READER_BACKSTOP: Duration = Duration::from_secs(15 * 60);
pub(crate) const UNREACHABLE: &str =
    "Could not reach OpenAI. Check the network connection and retry.";
pub(crate) const INTERRUPTED: &str = "The connection to OpenAI was interrupted. Please retry.";
pub(crate) const STALLED: &str =
    "OpenAI sent nothing for 5 minutes, so DutchyAgent stopped waiting. Please retry.";
const ENDED: &str = "The OpenAI connection ended before completion. Please retry.";

/// Host-side view of one request.
struct Pending {
    receiver: Receiver<Change>,
    cancel: Arc<AtomicBool>,
    /// Count of transport parts the reader has seen, bumped for every part.
    activity: Arc<AtomicU64>,
    seen: u64,
    /// Frame time at which the last activity was observed.
    quiet_since: f64,
}
enum Drain {
    Done,
    More,
    Waiting,
}
impl Pending {
    fn drain(&mut self, id: u64, now: f64, events: &mut Vec<Event>) -> Drain {
        let activity = self.activity.load(Ordering::Acquire);
        if activity != self.seen {
            self.seen = activity;
            self.quiet_since = now;
        }
        for _ in 0..64 {
            let change = match self.receiver.try_recv() {
                Ok(change) => change,
                Err(TryRecvError::Empty) => return self.watch(id, now, events),
                Err(TryRecvError::Disconnected) => Change::Failed(ENDED),
            };
            let terminal = matches!(change, Change::Complete(_) | Change::Failed(_));
            events.push(Event {
                request: id,
                change,
            });
            if terminal {
                return Drain::Done;
            }
        }
        Drain::More
    }
    fn watch(&mut self, id: u64, now: f64, events: &mut Vec<Event>) -> Drain {
        if now - self.quiet_since < IDLE_LIMIT {
            return Drain::Waiting;
        }
        self.cancel.store(true, Ordering::Release);
        events.push(Event {
            request: id,
            change: Change::Failed(STALLED),
        });
        Drain::Done
    }
}

/// Reader-thread side of one request; ehttp calls it for every part.
struct Reader {
    decoder: Mutex<protocol::Decoder>,
    sender: SyncSender<Change>,
    cancel: Arc<AtomicBool>,
    activity: Arc<AtomicU64>,
    responded: AtomicBool,
    context: Option<egui::Context>,
}
impl Reader {
    fn part(&self, part: ehttp::Result<Part>) -> ControlFlow<()> {
        self.activity.fetch_add(1, Ordering::Release);
        if self.cancel.load(Ordering::Acquire) {
            return ControlFlow::Break(());
        }
        let Ok(mut decoder) = self.decoder.lock() else {
            return ControlFlow::Break(());
        };
        let changes = self.changes(&mut decoder, part);
        let terminal = decoder.terminal()
            || changes
                .iter()
                .any(|c| matches!(c, Change::Complete(_) | Change::Failed(_)));
        for change in changes {
            if self.cancel.load(Ordering::Acquire) || self.sender.send(change).is_err() {
                return ControlFlow::Break(());
            }
            if let Some(context) = &self.context {
                context.request_repaint();
            }
        }
        if terminal {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
    fn changes(&self, decoder: &mut protocol::Decoder, part: ehttp::Result<Part>) -> Vec<Change> {
        match part {
            Ok(Part::Response(response)) => {
                self.responded.store(true, Ordering::Release);
                if response.ok {
                    Vec::new()
                } else {
                    vec![Change::Failed(status_error(response.status))]
                }
            }
            Ok(Part::Chunk(bytes)) if bytes.is_empty() => decoder.finish(),
            Ok(Part::Chunk(bytes)) => decoder.feed(&bytes),
            Err(_) if self.responded.load(Ordering::Acquire) => vec![Change::Failed(INTERRUPTED)],
            Err(_) => vec![Change::Failed(UNREACHABLE)],
        }
    }
}

pub(crate) struct OpenAi {
    pending: BTreeMap<u64, Pending>,
    context: Option<egui::Context>,
    endpoint: String,
    immediate: Vec<Event>,
}
impl Default for OpenAi {
    fn default() -> Self {
        Self {
            pending: BTreeMap::new(),
            context: None,
            endpoint: ENDPOINT.into(),
            immediate: Vec::new(),
        }
    }
}
impl OpenAi {
    pub fn context(&mut self, context: egui::Context) {
        self.context = Some(context);
    }
    #[cfg(test)]
    pub fn test_endpoint(endpoint: String) -> Self {
        let mut provider = Self::default();
        provider.endpoint = endpoint;
        provider
    }
    pub fn start(&mut self, request: Request, key: Option<&str>, now: f64) {
        let Some(key) = key.filter(|k| !k.is_empty() && !k.chars().any(char::is_whitespace)) else {
            self.fail(
                request.id,
                "Configure OPENAI_API_KEY in .env, then restart DutchyAgent.",
            );
            return;
        };
        let body = match protocol::body(&request) {
            Ok(body) => body,
            Err(message) => {
                self.fail(request.id, message);
                return;
            }
        };
        let accept = if request.options.streaming {
            "text/event-stream"
        } else {
            "application/json"
        };
        let mut http = ehttp::Request::new(
            ehttp::Method::POST,
            &self.endpoint,
            &[("Content-Type", "application/json"), ("Accept", accept)],
        )
        .with_body(body);
        http.headers
            .insert("Authorization", format!("Bearer {key}"));
        http.timeout = Some(READER_BACKSTOP);
        let (sender, receiver) = mpsc::sync_channel(64);
        let cancel = Arc::new(AtomicBool::new(false));
        let activity = Arc::new(AtomicU64::new(0));
        self.pending.insert(
            request.id,
            Pending {
                receiver,
                cancel: cancel.clone(),
                activity: activity.clone(),
                seen: 0,
                quiet_since: now,
            },
        );
        let reader = Reader {
            decoder: Mutex::new(protocol::Decoder::new(request.options.streaming)),
            sender,
            cancel,
            activity,
            responded: AtomicBool::new(false),
            context: self.context.clone(),
        };
        ehttp::streaming::fetch(http, move |part| reader.part(part));
    }
    pub fn poll(&mut self, now: f64) -> Vec<Event> {
        let mut events = std::mem::take(&mut self.immediate);
        let context = &self.context;
        self.pending
            .retain(|id, pending| match pending.drain(*id, now, &mut events) {
                Drain::Done => false,
                Drain::Waiting => true,
                Drain::More => {
                    if let Some(context) = context {
                        context.request_repaint();
                    }
                    true
                }
            });
        events
    }
    /// The next frame time at which a silent request reaches the idle limit.
    pub fn next_due(&self) -> Option<f64> {
        self.pending
            .values()
            .map(|p| p.quiet_since + IDLE_LIMIT)
            .min_by(f64::total_cmp)
    }
    pub fn cancel(&mut self, id: u64) {
        if let Some(pending) = self.pending.remove(&id) {
            pending.cancel.store(true, Ordering::Release);
        }
        self.immediate.retain(|e| e.request != id);
    }
    fn fail(&mut self, id: u64, message: &'static str) {
        self.immediate.push(Event {
            request: id,
            change: Change::Failed(message),
        });
        if let Some(ctx) = &self.context {
            ctx.request_repaint();
        }
    }
}
impl Drop for OpenAi {
    fn drop(&mut self) {
        for pending in self.pending.values() {
            pending.cancel.store(true, Ordering::Release);
        }
    }
}
fn status_error(status: u16) -> &'static str {
    match status {
        401 | 403 => {
            "OpenAI rejected the credentials or project access. Check OPENAI_API_KEY and restart the app."
        }
        429 => "OpenAI rate or usage limit reached. Check account limits and retry later.",
        400 | 404 => {
            "OpenAI rejected the model or request settings. Check the model name and effort."
        }
        500..=599 => "OpenAI is temporarily unavailable. Please retry later.",
        _ => "OpenAI returned an HTTP error. Please retry.",
    }
}
