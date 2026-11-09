//! Stable, application-owned records. Nothing is persisted by the library.
use crate::provider::{Mode, Role, Usage};

pub(crate) const MAX_PROMPT: usize = 4_000;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Status {
    Complete,
    Receiving,
    Stopped,
    Failed(&'static str),
}
#[derive(Clone, Debug)]
pub(crate) struct Message {
    pub id: u64,
    pub role: Role,
    pub text: String,
    pub status: Status,
    pub provider: Mode,
}
#[derive(Clone, Debug)]
pub(crate) struct Conversation {
    pub id: u64,
    pub title: String,
    pub draft: String,
    pub messages: Vec<Message>,
    pub active: Option<u64>,
    pub usage: Option<Usage>,
}
impl Conversation {
    pub fn retryable(&self) -> bool {
        self.active.is_none()
            && self
                .messages
                .last()
                .is_some_and(|m| matches!(m.status, Status::Stopped | Status::Failed(_)))
    }
}

#[derive(Default)]
pub(super) struct Ids(u64);
impl Ids {
    pub fn take(&mut self) -> Result<u64, &'static str> {
        let next = self
            .0
            .checked_add(1)
            .ok_or("The example has exhausted its record IDs.")?;
        let id = self.0;
        self.0 = next;
        Ok(id)
    }
}
