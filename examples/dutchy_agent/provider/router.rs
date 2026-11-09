//! Keep provider choice explicit and cancellation scoped to the original run.
use super::{Event, Mode, Provider, Request, Scripted, live::OpenAi};

#[derive(Default)]
pub(crate) struct Router {
    mock: Scripted,
    live: OpenAi,
}
impl Router {
    pub fn context(&mut self, context: egui::Context) {
        self.live.context(context);
    }
    pub fn start(&mut self, request: Request, mode: Mode, key: Option<&str>, now: f64) {
        match mode {
            Mode::Mock => self.mock.start(request, now),
            Mode::OpenAi => self.live.start(request, key),
        }
    }
    pub fn poll(&mut self, now: f64) -> Vec<Event> {
        let mut events = self.mock.poll(now);
        events.extend(self.live.poll());
        events
    }
    pub fn cancel(&mut self, id: u64) {
        self.mock.cancel(id);
        self.live.cancel(id);
    }
    pub fn next_due(&self) -> Option<f64> {
        self.mock.next_due()
    }
}
