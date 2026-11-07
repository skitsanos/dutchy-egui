use super::{style, typed};
use crate::{
    ControlSize, Family, Theme, Weight, fonts,
    interaction::paint_focus_inside,
    tokens::{border, control, space},
    widgets::navigation::{Axis, lock_arrows},
};
use egui::accesskit::Role;
use egui::{
    Align, Align2, Frame, Id, IdSalt, Key, Layout, Margin, Modifiers, Rect, Response, Sense,
    Stroke, TextEdit, Ui, UiBuilder, Widget, WidgetInfo, pos2, vec2,
};
use std::borrow::Cow;
use std::ops::RangeInclusive;

/// Numeric editor with square decrease/increase buttons and Up/Down stepping.
///
/// Typed text commits only when it parses; it reverts to the committed value
/// when the editor loses focus. Values clamp to the range.
#[must_use = "Add this number input to a Ui"]
pub struct NumberInput<'a> {
    id: IdSalt,
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    step: f64,
    decimals: usize,
    name: Option<Cow<'a, str>>,
    size: ControlSize,
    enabled: bool,
    read_only: bool,
    width: Option<f32>,
}

/// Frame, editor and button rectangles of one control.
struct Geometry {
    outer: Rect,
    editor: Rect,
    buttons: [Rect; 2],
}

impl Geometry {
    fn new(outer: Rect) -> Self {
        let inner = outer.shrink(border::STANDARD);
        let square = inner.height();
        let editor = Rect::from_min_max(
            inner.min,
            pos2(
                (inner.right() - 2.0 * square).max(inner.left()),
                inner.max.y,
            ),
        );
        let button = |index: f32| {
            Rect::from_min_size(
                pos2(editor.right() + square * index, inner.top()),
                vec2(square, square),
            )
        };
        Self {
            outer,
            editor,
            buttons: [button(0.0), button(1.0)],
        }
    }
}

impl<'a> NumberInput<'a> {
    /// Bind an editor to an application-owned value.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut f64) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            range: f64::NEG_INFINITY..=f64::INFINITY,
            step: 1.0,
            decimals: 0,
            name: None,
            size: ControlSize::Medium,
            enabled: true,
            read_only: false,
            width: None,
        }
    }
    /// Inclusive bounds; typed and stepped values clamp to them.
    pub fn range(mut self, range: RangeInclusive<f64>) -> Self {
        self.range = range;
        self
    }
    /// Increment for the buttons and Up/Down keys.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step.abs();
        self
    }
    /// Fixed decimal places for display and rounding of stepped values.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }
    /// Name for standalone use and for the step buttons; Field supplies one otherwise.
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
    /// Allow focus/selection/copy without editing or stepping.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Preferred outer width; defaults to all available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }

    fn format(&self) -> String {
        format!("{:.*}", self.decimals, *self.value)
    }

    /// Round to the configured decimals, clamp, and report whether the value moved.
    fn commit(&mut self, value: f64) -> bool {
        if !value.is_finite() {
            return false;
        }
        let scale = 10f64.powi(self.decimals as i32);
        let scaled = value * scale;
        let rounded = if scaled.is_finite() && scale > 0.0 {
            scaled.round() / scale
        } else {
            value
        };
        let clamped = rounded.clamp(*self.range.start(), *self.range.end());
        let changed = clamped != *self.value;
        *self.value = clamped;
        changed
    }

    fn step_value(&mut self, ui: &Ui, id: Id, direction: f64) -> bool {
        let changed = self.commit(*self.value + direction * self.step);
        if ui.memory(|memory| memory.has_focus(id)) {
            typed::retain(ui.ctx(), id, true, &self.format());
        }
        changed
    }

    /// Up/Down step while the editor has focus; consumed before the editor sees them.
    fn step_by_keys(&mut self, ui: &mut Ui, id: Id) -> bool {
        if self.read_only || !typed::editable_focus(ui, id) {
            return false;
        }
        let direction = ui.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                1.0
            } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                -1.0
            } else {
                0.0
            }
        });
        direction != 0.0 && self.step_value(ui, id, direction)
    }

    fn editor(&mut self, ui: &mut Ui, id: Id, rect: Rect) -> (Response, bool) {
        let formatted = self.format();
        let mut text = typed::draft(ui.ctx(), id, &formatted);
        let mut editor_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        editor_ui.visuals_mut().override_text_color = None;
        // egui's single-line editor is at least `interact_size.y` tall; match the slot.
        editor_ui.spacing_mut().interact_size.y = rect.height();
        let mut immutable;
        let buffer: &mut dyn egui::TextBuffer = if self.read_only {
            immutable = text.as_str();
            &mut immutable
        } else {
            &mut text
        };
        let response = TextEdit::singleline(buffer)
            .id(id)
            .frame(Frame::NONE.inner_margin(Margin::symmetric(space::MD as i8, 0)))
            .desired_width(rect.width() - 2.0 * space::MD)
            .min_size(rect.size())
            .align(Align2::LEFT_CENTER)
            .font(fonts::font_id(
                Family::Body,
                Weight::Regular,
                self.size.text_size().points(),
            ))
            .show(&mut editor_ui)
            .response
            .response;
        let mut changed = false;
        if response.changed()
            && !self.read_only
            && let Ok(parsed) = text.trim().parse::<f64>()
        {
            changed = self.commit(parsed);
        }
        if response.has_focus() {
            lock_arrows(ui, id, Axis::Vertical);
        }
        typed::retain(ui.ctx(), id, response.has_focus(), &text);
        (response, changed)
    }

    fn step_buttons(&mut self, ui: &mut Ui, id: Id, geometry: &Geometry, theme: Theme) -> bool {
        let mut changed = false;
        for (rect, (verb, direction)) in geometry
            .buttons
            .into_iter()
            .zip([("Decrease", -1.0), ("Increase", 1.0)])
        {
            let at_bound = if direction < 0.0 {
                *self.value <= *self.range.start()
            } else {
                *self.value >= *self.range.end()
            };
            let mut button_ui = ui.new_child(UiBuilder::new().max_rect(rect));
            if at_bound || self.read_only {
                button_ui.disable();
            }
            let button = button_ui.interact(rect, id.with(verb), Sense::click());
            let label = self
                .name
                .as_ref()
                .map_or_else(|| verb.to_owned(), |name| format!("{verb} {name}"));
            button
                .widget_info(|| WidgetInfo::labeled(Role::Button, button_ui.is_enabled(), &label));
            if button.clicked() {
                changed |= self.step_value(ui, id, direction);
            }
            paint_step(&button_ui, &button, rect, direction > 0.0, theme);
        }
        changed
    }

    fn declare(&self, response: &Response) {
        let (value, step, read_only) = (*self.value, self.step, self.read_only);
        let (min, max) = (*self.range.start(), *self.range.end());
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_role(Role::SpinButton);
            node.set_numeric_value(value);
            node.set_numeric_value_step(step);
            if min.is_finite() {
                node.set_min_numeric_value(min);
            }
            if max.is_finite() {
                node.set_max_numeric_value(max);
            }
            if read_only {
                node.set_read_only();
            }
        });
    }
}

impl Widget for NumberInput<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id);
        let theme = Theme::from_context(ui.ctx());
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());
        let (outer, _) = ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());
        let mut scope = ui.new_child(UiBuilder::new().max_rect(outer));
        if !self.enabled {
            scope.disable();
        }
        let ui = &mut scope;
        let geometry = Geometry::new(outer);
        style::paint_frame(ui, outer, theme);
        let mut changed = self.step_by_keys(ui, id);
        let (mut response, typed) = self.editor(ui, id, geometry.editor);
        changed |= typed;
        changed |= self.step_buttons(ui, id, &geometry, theme);
        if changed {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        if let Some(name) = self.name.clone() {
            response = response.accessible_name(name.into_owned());
        }
        self.declare(&response);
        let mut outline = response.clone();
        outline.rect = geometry.outer;
        style::finish(ui, &outline);
        response
    }
}

fn paint_step(ui: &Ui, response: &Response, rect: Rect, plus: bool, theme: Theme) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let p = theme.palette;
    if response.hovered() && ui.is_enabled() {
        ui.painter().rect_filled(rect, 0, p.muted);
    }
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min, vec2(border::THIN, rect.height())),
        0,
        p.border,
    );
    let half = control::STEP_GLYPH / 2.0;
    let center = rect.center();
    let stroke = Stroke::new(border::STANDARD, p.foreground);
    ui.painter().line_segment(
        [
            pos2(center.x - half, center.y),
            pos2(center.x + half, center.y),
        ],
        stroke,
    );
    if plus {
        ui.painter().line_segment(
            [
                pos2(center.x, center.y - half),
                pos2(center.x, center.y + half),
            ],
            stroke,
        );
    }
    paint_focus_inside(ui, response, p.primary);
}
