use egui::{Id, Response};
/// Overlay drawer attachment edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerSide {
    /// Full-height panel at the left edge.
    Left,
    /// Full-height panel at the right edge.
    #[default]
    Right,
}
/// Explicit dismissal source. Business actions remain in the host closure result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayDismissal {
    /// Header close button activation.
    CloseButton,
    /// Escape, when no child popup is open.
    Escape,
    /// Pointer activation outside the surface.
    Backdrop,
    /// A content closure called Ui::close.
    Content,
}
/// Common modal behavior. Visibility and durable content values remain host-owned.
#[derive(Clone, Debug)]
pub struct OverlayOptions {
    /// Allow Escape dismissal. Defaults to true.
    pub dismiss_on_escape: bool,
    /// Allow backdrop dismissal. Defaults to true.
    pub dismiss_on_backdrop: bool,
    /// Show the accessible header close button. Defaults to true.
    pub show_close_button: bool,
    /// Optional actual egui control ID to focus on entry. Defaults to the close
    /// button (or heading); a content closure's explicit focus request is honored.
    pub initial_focus: Option<Id>,
    /// Focus target after closing. Defaults to the previously focused control.
    /// Supply a surviving target if the opener will be removed.
    pub return_focus: Option<Id>,
}
impl Default for OverlayOptions {
    fn default() -> Self {
        Self {
            dismiss_on_escape: true,
            dismiss_on_backdrop: true,
            show_close_button: true,
            initial_focus: None,
            return_focus: None,
        }
    }
}
/// Native overlay geometry and the host's content result.
pub struct OverlayResponse<R> {
    /// Surface response; the backdrop is a separate response.
    pub response: Response,
    /// Full-viewport input-blocking backdrop.
    pub backdrop_response: Response,
    /// Results returned by the body, or (body, footer) closures.
    pub inner: R,
    /// Dismissal applied to the bound open flag this frame, if any.
    pub dismissed: Option<OverlayDismissal>,
}
impl<R> OverlayResponse<R> {
    pub(super) fn map<T>(self, f: impl FnOnce(R) -> T) -> OverlayResponse<T> {
        OverlayResponse {
            response: self.response,
            backdrop_response: self.backdrop_response,
            inner: f(self.inner),
            dismissed: self.dismissed,
        }
    }
}

pub(in crate::widgets) fn dismissal<T>(
    modal: &egui::ModalResponse<T>,
    close: bool,
    options: &OverlayOptions,
) -> Option<OverlayDismissal> {
    if close {
        return Some(OverlayDismissal::CloseButton);
    }
    if modal.response.should_close() {
        return Some(OverlayDismissal::Content);
    }
    if !modal.is_top_modal {
        return None;
    }
    if options.dismiss_on_escape
        && !modal.any_popup_open
        && modal
            .response
            .ctx
            .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        return Some(OverlayDismissal::Escape);
    }
    if options.dismiss_on_backdrop && modal.backdrop_response.clicked() {
        return Some(OverlayDismissal::Backdrop);
    }
    None
}
