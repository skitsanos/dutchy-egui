/// Application-owned rename session. The committed value is supplied separately.
/// A hidden control keeps its draft; call `cancel` when the host removes a row.
#[derive(Clone, Debug, Default)]
pub struct InlineEditState {
    pub(super) draft: Option<String>,
    pub(super) focus_pending: bool,
}

impl InlineEditState {
    /// Start/restart editing, selecting the seed text on the next enabled draw.
    /// Programmatic starts do not emit a user `Started` event.
    pub fn begin(&mut self, value: impl Into<String>) {
        self.draft = Some(value.into());
        self.focus_pending = true;
    }
    /// Discard the session. Programmatic cancellation emits no user event.
    pub fn cancel(&mut self) {
        self.draft = None;
        self.focus_pending = false;
    }
    /// Whether a rename session is open, including while hidden or disabled.
    pub fn is_editing(&self) -> bool {
        self.draft.is_some()
    }
    /// Current uncommitted text; not overwritten by changes to the host label.
    pub fn draft(&self) -> Option<&str> {
        self.draft.as_deref()
    }
    /// Edit a retained draft from host code, without emitting a widget event.
    pub fn draft_mut(&mut self) -> Option<&mut String> {
        self.draft.as_mut()
    }
}

/// One explicit user outcome. Persistence and acceptance belong to the host.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum InlineEditEvent {
    /// The label was activated by pointer, Enter, Space or F2.
    Started,
    /// Submitted text, verbatim, including empty or unchanged strings. The
    /// session ends; the host decides whether to accept this proposed value.
    Commit(String),
    /// The draft was discarded through Escape or the cancel action.
    Cancelled,
}
