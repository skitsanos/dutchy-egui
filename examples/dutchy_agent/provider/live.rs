//! Native streaming HTTP with bounded delivery and cooperative cancellation.
use super::{Change, Event, Request, protocol};
use ehttp::streaming::Part;
use std::{
    collections::BTreeMap,
    ops::ControlFlow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    time::Duration,
};

const ENDPOINT: &str = "https://api.openai.com/v1/responses";
struct Pending {
    receiver: Receiver<Change>,
    cancel: Arc<AtomicBool>,
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
        Self {
            endpoint,
            pending: BTreeMap::new(),
            context: None,
            immediate: Vec::new(),
        }
    }
    pub fn start(&mut self, request: Request, key: Option<&str>) {
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
        http.timeout = Some(Duration::from_secs(90));
        let (sender, receiver) = mpsc::sync_channel(64);
        let cancel = Arc::new(AtomicBool::new(false));
        self.pending.insert(
            request.id,
            Pending {
                receiver,
                cancel: cancel.clone(),
            },
        );
        let decoder = Mutex::new(protocol::Decoder::new(request.options.streaming));
        let context = self.context.clone();
        ehttp::streaming::fetch(http, move |part| {
            if cancel.load(Ordering::Acquire) {
                return ControlFlow::Break(());
            }
            let Ok(mut decoder) = decoder.lock() else {
                return ControlFlow::Break(());
            };
            let changes = match part {
                Ok(Part::Response(response)) if !response.ok => {
                    vec![Change::Failed(status_error(response.status))]
                }
                Ok(Part::Response(_)) => Vec::new(),
                Ok(Part::Chunk(bytes)) if bytes.is_empty() => decoder.finish(),
                Ok(Part::Chunk(bytes)) => decoder.feed(&bytes),
                Err(_) => vec![Change::Failed(
                    "Could not reach OpenAI or the connection was interrupted. Please retry.",
                )],
            };
            let terminal = decoder.terminal()
                || changes
                    .iter()
                    .any(|c| matches!(c, Change::Complete(_) | Change::Failed(_)));
            for change in changes {
                if cancel.load(Ordering::Acquire) || sender.send(change).is_err() {
                    return ControlFlow::Break(());
                }
                if let Some(context) = &context {
                    context.request_repaint();
                }
            }
            if terminal {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
    }
    pub fn poll(&mut self) -> Vec<Event> {
        let mut events = std::mem::take(&mut self.immediate);
        self.pending.retain(|id, pending| {
            for _ in 0..64 {
                match pending.receiver.try_recv() {
                    Ok(change) => {
                        let terminal = matches!(change, Change::Complete(_) | Change::Failed(_));
                        events.push(Event {
                            request: *id,
                            change,
                        });
                        if terminal {
                            return false;
                        }
                    }
                    Err(TryRecvError::Empty) => return true,
                    Err(TryRecvError::Disconnected) => {
                        events.push(Event {
                            request: *id,
                            change: Change::Failed(
                                "The OpenAI connection ended before completion. Please retry.",
                            ),
                        });
                        return false;
                    }
                }
            }
            if let Some(context) = &self.context {
                context.request_repaint();
            }
            true
        });
        events
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
