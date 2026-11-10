mod geometry;
mod input;
mod session;
mod state;
mod view;
use crate::{ControlSize, interaction::reveal_focused};
use egui::{IdSalt, Response, Sense, Ui, UiBuilder};
pub use state::{InlineEditEvent, InlineEditState};
use std::borrow::Cow;

/// Primary label/editor response and one explicit user outcome.
#[non_exhaustive]
pub struct InlineEditResponse {
    /// Stable per-mode control identity. Its rectangle covers the composition;
    /// changed() reports draft edits, not acceptance of a rename by the host.
    pub response: Response,
    /// User activation, a commit request, or cancellation; never emitted by blur.
    pub event: Option<InlineEditEvent>,
    /// Whether the application-owned rename session remains open.
    pub editing: bool,
}

/// A label that opens a native editor with explicit save/cancel actions.
/// The label is borrowed; only the host can apply a proposed rename.
#[must_use = "Call show to render InlineEdit"]
pub struct InlineEdit<'a> {
    id: IdSalt,
    value: &'a str,
    state: &'a mut InlineEditState,
    name: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    size: ControlSize,
    width: Option<f32>,
    limit: Option<usize>,
    enabled: bool,
    read_only: bool,
}
impl<'a> InlineEdit<'a> {
    /// Bind stable identity, the current committed label and a host-owned draft.
    pub fn new(id: impl egui::AsIdSalt, value: &'a str, state: &'a mut InlineEditState) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            state,
            name: "Name".into(),
            placeholder: "Untitled".into(),
            size: ControlSize::Small,
            width: None,
            limit: None,
            enabled: true,
            read_only: false,
        }
    }
    /// Name of the renamed property, independent of its current value.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Displayed when the committed label is empty; never used as draft content.
    pub fn placeholder(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// Shared control height and text scale; Small by default.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Preferred outer width, bounded by the available width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// Limit typed/pasted Unicode scalars without truncating a host-provided seed.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Disable all interaction, preserving any open draft and disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Block starting, editing and committing. An existing draft can still be
    /// selected/copied or explicitly cancelled.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Enter/check requests a commit; Escape/close cancels. Blur and Tab retain
    /// the draft. Empty, unchanged and whitespace values are submitted verbatim.
    pub fn show(self, ui: &mut Ui) -> InlineEditResponse {
        let root = ui.make_persistent_id(("dutchy-inline-edit", self.id));
        let width = geometry::width(self.width, ui.available_width());
        let anchor = ui.min_rect().min;
        let (rect, _) = ui.allocate_exact_size(
            geometry::extent(width, self.size, self.state.is_editing()),
            Sense::hover(),
        );
        let mut child = ui.new_child(UiBuilder::new().scope_id(root).max_rect(rect));
        child.set_clip_rect(ui.clip_rect().intersect(rect));
        if !self.enabled {
            child.disable();
        }
        if !rect.is_positive() {
            child.set_invisible();
        }
        let options = view::Options {
            size: self.size,
            name: &self.name,
            read_only: self.read_only,
            limit: self.limit,
        };
        let trigger = child.make_persistent_id("trigger");
        if !self.state.is_editing() && self.state.focus_pending && child.is_enabled() {
            if session::may_return_focus(&child, trigger) {
                child.memory_mut(|m| m.request_focus(trigger));
            }
            self.state.focus_pending = false;
        }
        let (mut response, event) = if self.state.is_editing() {
            session::edit(&mut child, rect, self.state, options)
        } else {
            let response = view::idle(
                &mut child,
                trigger,
                rect,
                self.value,
                &self.placeholder,
                options,
            );
            let start = !self.read_only
                && child.is_enabled()
                && (response.clicked() || input::f2(&child, trigger));
            let event = start.then(|| {
                let space = input::consume_activation(&child);
                let pointer = response.clicked_by(egui::PointerButton::Primary);
                input::arm(&child, pointer, space);
                self.state.begin(self.value);
                child.ctx().request_repaint();
                InlineEditEvent::Started
            });
            (response, event)
        };
        response.rect = rect;
        reveal_focused(&child, &response, anchor);
        InlineEditResponse {
            response,
            event,
            editing: self.state.is_editing(),
        }
    }
}
