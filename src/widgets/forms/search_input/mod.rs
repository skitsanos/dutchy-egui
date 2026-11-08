mod geometry;

use super::{Input, style};
use crate::{
    ControlSize, Kbd, Theme,
    tokens::{border, search_input as tokens},
    widgets::close,
};
use egui::{
    IdSalt, Rect, Response, Sense, Stroke, Ui, UiBuilder, Widget, WidgetInfo, accesskit::Role, vec2,
};
use std::borrow::Cow;

/// Native query editor with magnifier, independent clear button and optional key hint.
///
/// Filtering and global shortcuts belong to the host. Escape is not consumed.
/// Clearing is an edit and returns focus to the editor. Narrow layouts omit
/// decorative slots before reducing editor space.
#[must_use = "Add the search input to a Ui"]
pub struct SearchInput<'a> {
    id: IdSalt,
    value: &'a mut String,
    name: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    hint: Option<Cow<'a, str>>,
    size: ControlSize,
    width: Option<f32>,
    limit: Option<usize>,
    enabled: bool,
    read_only: bool,
    clearable: bool,
}

impl<'a> SearchInput<'a> {
    /// Bind stable identity and an application-owned query.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut String) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            name: "Search".into(),
            placeholder: "Search…".into(),
            hint: None,
            size: ControlSize::Medium,
            width: None,
            limit: None,
            enabled: true,
            read_only: false,
            clearable: true,
        }
    }
    /// Standalone name; Field may supply the editor's visible label.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Empty-query hint, independent of the accessible name.
    pub fn placeholder(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// Decorative Kbd chip. The host must bind and document any shortcut.
    pub fn key_hint(mut self, key: impl Into<Cow<'a, str>>) -> Self {
        self.hint = Some(key.into());
        self
    }
    /// Shared Small/Medium/Large height and type scale.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Preferred outer width, constrained to available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }
    /// Limit typed/pasted Unicode scalars without truncating a host-set query.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Disable editing and clearing, honoring disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Retain focus/selection/copy while blocking edits and clearing.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Show/remove the conditional clear button; defaults to true.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }
}

impl Widget for SearchInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let root = ui.make_persistent_id(("dutchy-search-input", self.id));
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width())
            .max(0.0);
        let (outer, _) = ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());
        let mut child = ui.new_child(UiBuilder::new().scope_id(root).max_rect(outer));
        if !self.enabled {
            child.disable();
        }
        let ui = &mut child;
        let theme = Theme::from_context(ui.ctx());
        style::paint_frame(ui, outer, theme);
        let hint = self.hint.as_deref().map(Kbd::new);
        let slots = geometry::Geometry::new(
            outer,
            self.clearable && !self.value.is_empty(),
            hint.as_ref().map(|kbd| kbd.desired_size(ui)),
        );
        let mut editor_ui = ui.new_child(UiBuilder::new().max_rect(slots.editor));
        let mut response = editor_ui.add(
            Input::new("query", self.value)
                .accessible_name(self.name.as_ref())
                .placeholder(self.placeholder.as_ref())
                .size(self.size)
                .read_only(self.read_only)
                .char_limit(self.limit.unwrap_or(usize::MAX))
                .width(slots.editor.width())
                .without_frame(slots.editor.height()),
        );
        response
            .ctx
            .accesskit_node_builder(response.id, |node| node.set_role(Role::SearchInput));
        focus_adornments(ui, root, outer, &slots, &response);
        if let Some(rect) = slots.icon {
            magnifier(ui, rect, theme);
        }
        if let (Some(rect), Some(hint)) = (slots.hint, hint) {
            let mut hint_ui = ui.new_child(
                UiBuilder::new()
                    .scope_id(root.with("key-hint"))
                    .max_rect(rect),
            );
            hint_ui.add(hint);
        }
        if let Some(rect) = slots.clear {
            let clear = ui
                .add_enabled_ui(!self.read_only, |ui| {
                    let clear = ui.interact(rect, root.with("clear"), Sense::click());
                    clear.widget_info(|| {
                        WidgetInfo::labeled(
                            Role::Button,
                            ui.is_enabled(),
                            format!("Clear {}", self.name),
                        )
                    });
                    let p = theme.palette;
                    close::paint(
                        ui,
                        &clear,
                        tokens::CLEAR_ICON,
                        close::Colors {
                            idle: p.muted_foreground,
                            foreground: p.foreground,
                            focus: p.primary,
                        },
                    );
                    clear
                })
                .inner;
            if clear.clicked() {
                self.value.clear();
                response.mark_changed();
                response.request_focus();
                ui.ctx().request_repaint();
            }
        }
        if self.read_only || !ui.is_enabled() {
            response.flags.remove(egui::response::Flags::CHANGED);
        }
        response.rect = outer;
        style::finish(ui, &response);
        response
    }
}

fn focus_adornments(
    ui: &Ui,
    root: egui::Id,
    outer: Rect,
    slots: &geometry::Geometry,
    editor: &Response,
) {
    // A full-field click target competes with TextEdit's pointer focus. Only
    // register the decorative margins, leaving editor and clear targets alone.
    let leading = Rect::from_min_max(outer.min, egui::pos2(slots.editor.left(), outer.bottom()));
    let right = slots
        .clear
        .map_or(slots.editor.right(), |rect| rect.right());
    let trailing = Rect::from_min_max(egui::pos2(right, outer.top()), outer.max);
    for (key, rect) in [("leading", leading), ("trailing", trailing)] {
        if ui.interact(rect, root.with(key), Sense::CLICK).clicked() {
            editor.request_focus();
        }
    }
}

fn magnifier(ui: &Ui, rect: Rect, theme: Theme) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let stroke = Stroke::new(border::STANDARD, theme.palette.muted_foreground);
    let radius = rect.width() / 3.0;
    let center = rect.min + vec2(radius + border::STANDARD, radius + border::STANDARD);
    painter.circle_stroke(center, radius, stroke);
    painter.line_segment(
        [
            center + vec2(radius * 0.7, radius * 0.7),
            rect.right_bottom(),
        ],
        stroke,
    );
}
