//! Timer-driven simulation: no threads, blocking sleeps or network requests.
use super::{Change, Event, Provider, Request, Scenario, Usage, UsageKind, script};
use std::collections::{BTreeMap, VecDeque};

pub(crate) const FIRST_REPLY_DELAY: f64 = 0.45;
const CHUNK_DELAY: f64 = 0.055;
struct Pending {
    at: f64,
    chunks: VecDeque<String>,
    usage: Usage,
    scenario: Scenario,
    emitted: usize,
    fail_after: usize,
}
#[derive(Default)]
pub(crate) struct Scripted {
    pending: BTreeMap<u64, Pending>,
}

impl Provider for Scripted {
    fn start(&mut self, request: Request, now: f64) {
        let reply = script::reply(&request.messages, &request.options);
        let usage = Usage {
            input: request
                .messages
                .iter()
                .map(|p| p.text.chars().count())
                .sum::<usize>()
                .div_ceil(4),
            output: reply.chars().count().div_ceil(4),
            kind: UsageKind::Estimated,
            ..Default::default()
        };
        self.pending.insert(
            request.id,
            Pending {
                at: now + FIRST_REPLY_DELAY,
                chunks: script::chunks(&reply, request.options.streaming),
                usage,
                scenario: request.scenario,
                emitted: 0,
                fail_after: if request.scenario == Scenario::Network && request.options.streaming {
                    2
                } else {
                    0
                },
            },
        );
    }

    fn poll(&mut self, now: f64) -> Vec<Event> {
        let mut events = Vec::new();
        self.pending.retain(|id, pending| {
            for _ in 0..8 {
                if pending.at > now {
                    return true;
                }
                if let Some(error) = pending.error() {
                    events.push(Event {
                        request: *id,
                        change: Change::Failed(error),
                    });
                    return false;
                }
                let Some(text) = pending.chunks.pop_front() else {
                    events.push(Event {
                        request: *id,
                        change: Change::Complete(pending.usage.clone()),
                    });
                    return false;
                };
                events.push(Event {
                    request: *id,
                    change: Change::Delta(text),
                });
                pending.emitted += 1;
                pending.at += CHUNK_DELAY;
            }
            true
        });
        events
    }

    fn cancel(&mut self, request: u64) {
        self.pending.remove(&request);
    }
    fn next_due(&self) -> Option<f64> {
        self.pending.values().map(|p| p.at).min_by(f64::total_cmp)
    }
}
impl Pending {
    fn error(&self) -> Option<&'static str> {
        if self.emitted < self.fail_after {
            None
        } else {
            self.scenario.error()
        }
    }
}
