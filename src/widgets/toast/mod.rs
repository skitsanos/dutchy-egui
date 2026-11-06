mod controls;
mod item;
mod memory;
mod paint;
mod queue;
mod render;
mod types;
use crate::tokens::toast as geometry;
use egui::{Context, Id, IdSalt};
pub use item::Toast;
pub use queue::ToastQueue;
pub use types::{
    ToastAppearance, ToastDismissal, ToastEvent, ToastId, ToastPosition, ToastResponse,
    ToastVariant, ToastsResponse,
};
/// Nonblocking corner stack over an application-owned FIFO queue.
/// Call once per frame, even with an empty queue, to track time and removed focus.
#[must_use = "Call show to display notifications and collect outcomes"]
pub struct Toasts<'a, T = ()> {
    id: IdSalt,
    queue: &'a mut ToastQueue<T>,
    position: ToastPosition,
    max_visible: usize,
    paused: bool,
    name: String,
    return_focus: Option<Id>,
}
impl<'a, T> Toasts<'a, T> {
    /// Bind a globally unique stack ID and host-owned queue.
    pub fn new(id: impl egui::AsIdSalt, queue: &'a mut ToastQueue<T>) -> Self {
        Self {
            id: IdSalt::new(id),
            queue,
            position: ToastPosition::default(),
            max_visible: geometry::DEFAULT_VISIBLE,
            paused: false,
            name: "Notifications".into(),
            return_focus: None,
        }
    }
    /// Attach to any viewport corner.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }
    /// Concurrent slots, clamped to 1–4. Pending entries do not expire.
    pub fn max_visible(mut self, count: usize) -> Self {
        self.max_visible = count.clamp(1, geometry::MAX_VISIBLE);
        self
    }
    /// Pause all visible lifetimes explicitly. Controls remain interactive.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }
    /// Accessible stack name. Individual toasts announce their own content.
    pub fn accessible_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    /// Preferred surviving focus target after removing a focused notification.
    /// Otherwise focus moves to another toast control, then the last outside control.
    pub fn return_focus_to(mut self, id: Id) -> Self {
        self.return_focus = Some(id);
        self
    }
    /// Render, age eligible visible entries, remove finished entries, and return
    /// action/dismissal events. Arrival never requests focus. Modals and inactive
    /// windows suspend the stack; hover/focus/clipping pause individual lifetimes.
    pub fn show(self, ctx: &Context) -> ToastsResponse<T> {
        render::show(self, ctx)
    }
}
