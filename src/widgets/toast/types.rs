/// Notification severity. Toast warnings are polite; errors are assertive.
pub use crate::AlertVariant as ToastVariant;
use egui::Response;
/// Notification surface treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToastAppearance {
    /// Live web treatment: solid severity fill and uppercase text.
    #[default]
    Solid,
    /// Documented treatment: background fill, severity outline and body text.
    Outline,
}
/// Stack attachment corner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToastPosition {
    /// Top left.
    TopLeft,
    /// Top right.
    TopRight,
    /// Bottom left.
    BottomLeft,
    /// Bottom right, matching the source default.
    #[default]
    BottomRight,
}
impl ToastPosition {
    pub(super) fn anchor(self) -> egui::Align2 {
        match self {
            Self::TopLeft => egui::Align2::LEFT_TOP,
            Self::TopRight => egui::Align2::RIGHT_TOP,
            Self::BottomLeft => egui::Align2::LEFT_BOTTOM,
            Self::BottomRight => egui::Align2::RIGHT_BOTTOM,
        }
    }
}
/// Opaque key, unique among a queue's live entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub(super) u64);
/// Why a notification left the queue without an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToastDismissal {
    /// Manual close-button activation.
    CloseButton,
    /// Its unpaused visible lifetime elapsed.
    Expired,
}
/// Notification outcome; the library never executes the action payload.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToastEvent<T> {
    /// Removed by the user or lifetime policy.
    Dismissed {
        /// Removed notification key.
        id: ToastId,
        /// Removal source.
        reason: ToastDismissal,
    },
    /// Action activated; the notification is removed and its payload is returned once.
    Action {
        /// Removed notification key.
        id: ToastId,
        /// Payload for host execution.
        action: T,
    },
}
/// Native geometry and controls of a rendered notification.
#[non_exhaustive]
pub struct ToastResponse {
    /// Queue key.
    pub id: ToastId,
    /// Named status/alert surface.
    pub response: Response,
    /// Optional close control.
    pub close: Option<Response>,
    /// Optional action control.
    pub action: Option<Response>,
}
impl ToastResponse {
    pub(super) fn controls(&self) -> impl Iterator<Item = &Response> {
        self.close.iter().chain(self.action.iter())
    }
}
/// Stack geometry and outcomes, also returned while the queue is empty or suspended.
#[non_exhaustive]
pub struct ToastsResponse<T> {
    /// Stack response, or None when not rendered.
    pub response: Option<Response>,
    /// Rendered entries in oldest-first reading order; some can leave this frame.
    pub visible: Vec<ToastResponse>,
    /// Notifications waiting for a visible slot, excluding rendered entries.
    pub pending: usize,
    /// Dismissals and action payloads for the host to handle.
    pub events: Vec<ToastEvent<T>>,
}
