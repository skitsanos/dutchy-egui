use super::Input;
use crate::{ControlSize, Text, TextColor, TextSize, tokens::control};
use egui::{Response, Ui, Widget};
use std::borrow::Cow;

/// Multiline editor with optional Unicode scalar counting.
#[must_use = "Add the textarea to a Ui"]
pub struct Textarea<'a> {
    input: Input<'a>,
    show_count: bool,
}

impl<'a> Textarea<'a> {
    /// Bind a stable editor ID to an application-owned value.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut String) -> Self {
        let mut input = Input::new(id, value);
        input.multiline = true;
        input.rows = control::TEXTAREA_ROWS;
        Self {
            input,
            show_count: false,
        }
    }
    /// Empty-state hint.
    pub fn placeholder(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.input = self.input.placeholder(hint);
        self
    }
    /// Name for standalone use without Field.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.input = self.input.accessible_name(name);
        self
    }
    /// Shared typography/control-size tier.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.input = self.input.size(size);
        self
    }
    /// Disable interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.input = self.input.enabled(enabled);
        self
    }
    /// Keep focus/selection/copy while preventing edits.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.input = self.input.read_only(read_only);
        self
    }
    /// Count Unicode scalar values consistently with the editing limit.
    pub fn show_count(mut self, show: bool) -> Self {
        self.show_count = show;
        self
    }
    /// Editing limit; does not truncate externally supplied values.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.input = self.input.char_limit(limit);
        self
    }
    /// Visible rows, with a 120-point minimum height.
    pub fn rows(mut self, rows: usize) -> Self {
        self.input.rows = rows.max(1);
        self
    }
    /// Preferred outer width.
    pub fn width(mut self, width: f32) -> Self {
        self.input = self.input.width(width);
        self
    }
}

impl Widget for Textarea<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let limit = self.input.limit;
        let (response, count) = self.input.render(ui);
        if self.show_count {
            let caption = limit.map_or_else(
                || format!("{count} characters"),
                |max| format!("{count} / {max}"),
            );
            ui.add(
                Text::new(caption.clone())
                    .size(TextSize::Xs)
                    .color(TextColor::Muted),
            );
            response
                .ctx
                .accesskit_node_builder(response.id, |node| node.set_description(caption));
        }
        response
    }
}
