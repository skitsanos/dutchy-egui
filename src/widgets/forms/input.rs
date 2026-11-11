use super::style;
use crate::{ControlSize, Family, Weight, fonts, tokens::control};
use egui::{IdSalt, Response, TextBuffer, TextEdit, Ui, Widget, vec2};
use std::borrow::Cow;

/// Single-line text/password input. IDs must be stable within the enclosing UI.
#[must_use = "Add the input to a Ui"]
pub struct Input<'a> {
    value: &'a mut String,
    id: IdSalt,
    placeholder: Cow<'a, str>,
    name: Option<Cow<'a, str>>,
    pub(super) size: ControlSize,
    enabled: bool,
    read_only: bool,
    password: bool,
    pub(super) limit: Option<usize>,
    width: Option<f32>,
    pub(super) multiline: bool,
    pub(super) rows: usize,
    framed: bool,
    height_override: Option<f32>,
    return_key: bool,
}

impl<'a> Input<'a> {
    /// Bind an editor to an application-owned String.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut String) -> Self {
        Self {
            value,
            id: IdSalt::new(id),
            placeholder: Cow::Borrowed(""),
            name: None,
            size: ControlSize::Medium,
            enabled: true,
            read_only: false,
            password: false,
            limit: None,
            width: None,
            multiline: false,
            rows: 1,
            framed: true,
            height_override: None,
            return_key: true,
        }
    }
    /// Hint shown when empty; not a replacement for an accessible label.
    pub fn placeholder(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = hint.into();
        self
    }
    /// Accessible name for standalone use. Field supplies a visible label instead.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Shared control height/text scale.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disable interaction, honoring a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Allow focus/selection/copy without editing. Does not disable the control.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Mask the value and let egui restrict password clipboard operations.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }
    /// Restrict editing/paste by Unicode scalar count. Does not truncate a host-set value.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Preferred outer width; defaults to all available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }

    /// The composite search control supplies its outer frame and editor slot.
    pub(super) fn without_frame(mut self, height: f32) -> Self {
        self.framed = false;
        self.height_override = Some(height);
        self
    }

    /// InlineEdit handles submission after applying this frame's text events.
    pub(super) fn without_return_key(mut self) -> Self {
        self.return_key = false;
        self
    }

    pub(super) fn render(self, ui: &mut Ui) -> (Response, usize) {
        let id = ui.make_persistent_id(self.id);
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());
        let height = self.height_override.unwrap_or(if self.multiline {
            control::TEXTAREA_MIN_HEIGHT
        } else {
            self.size.height()
        });
        let response = ui
            .allocate_ui_with_layout(
                vec2(width, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.style_mut().spacing.interact_size.y =
                        self.height_override.unwrap_or(self.size.height());
                    // The editor keeps its own light fill, so inverse surfaces
                    // must not recolor its text.
                    ui.visuals_mut().override_text_color = None;
                    if !self.enabled {
                        ui.disable();
                    }
                    let frame = if self.framed {
                        style::frame(ui, self.size)
                    } else {
                        egui::Frame::NONE
                    };
                    // A focused editor can receive queued input in the frame it is disabled.
                    let editable = ui.is_enabled() && !self.read_only;
                    let mut immutable;
                    let buffer: &mut dyn TextBuffer = if !editable {
                        immutable = self.value.as_str();
                        &mut immutable
                    } else {
                        &mut *self.value
                    };
                    let editor = if self.multiline {
                        TextEdit::multiline(buffer)
                    } else {
                        TextEdit::singleline(buffer)
                    };
                    let editor = if self.return_key {
                        editor
                    } else {
                        editor.return_key(None)
                    };
                    let align = if self.multiline {
                        egui::Align2::LEFT_TOP
                    } else {
                        egui::Align2::LEFT_CENTER
                    };
                    let mut response = editor
                        .id(id)
                        .frame(frame)
                        .align(align)
                        .desired_width(width)
                        .desired_rows(self.rows)
                        .min_size(vec2(width, height))
                        .password(self.password)
                        .font(fonts::font_id(
                            Family::Body,
                            Weight::Regular,
                            self.size.text_size().points(),
                        ))
                        .hint_text(self.placeholder.into_owned())
                        .char_limit(self.limit.unwrap_or(usize::MAX))
                        .show(ui)
                        .response
                        .response;
                    if !editable {
                        // egui can mark a paste into an immutable buffer as changed.
                        response.flags.remove(egui::response::Flags::CHANGED);
                    }
                    if let Some(name) = self.name {
                        response = response.accessible_name(name.into_owned());
                    }
                    response.ctx.accesskit_node_builder(response.id, |node| {
                        if self.read_only {
                            node.set_read_only();
                        }
                    });
                    if self.framed {
                        style::finish(ui, &response);
                    }
                    response
                },
            )
            .inner;
        (response, self.value.chars().count())
    }
}

impl Widget for Input<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.render(ui).0
    }
}
