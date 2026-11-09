use super::{
    calendar_grid::{self, Bounds, Outcome},
    style, typed,
};
use crate::{
    ControlSize, Family, Theme, Weight,
    calendar::{Date, DateSegment},
    fonts,
    interaction::paint_focus_inside,
    tokens::{border, control, space},
    widgets::navigation::{Axis, lock_arrows},
};
use egui::accesskit::Role;
use egui::{
    Align, Align2, Frame, Id, IdSalt, Key, Layout, Margin, Modifiers, Rect, Response, Sense,
    Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Widget, WidgetInfo, pos2, vec2,
};
use std::borrow::Cow;

/// Where stepping and the calendar start when there is no value, bound or today.
pub(super) const FALLBACK: Date = match Date::new(2000, 1, 1) {
    Some(date) => date,
    None => panic!("2000-01-01 is a valid date"),
};

const SEGMENTS: [DateSegment; 3] = [DateSegment::Year, DateSegment::Month, DateSegment::Day];

/// ISO `YYYY-MM-DD` editor with a calendar popup over an application-owned
/// optional date.
///
/// Typed text commits when it parses, clears the value when emptied, clamps to
/// the bounds and reverts on blur. Up/Down step the segment under the caret.
/// Supply `today` from the host clock to mark it and to anchor an empty picker;
/// the library reads no clock.
#[must_use = "Add this date picker to a Ui"]
pub struct DatePicker<'a> {
    id: IdSalt,
    value: &'a mut Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
    today: Option<Date>,
    name: Option<Cow<'a, str>>,
    size: ControlSize,
    enabled: bool,
    width: Option<f32>,
}

impl<'a> DatePicker<'a> {
    /// Bind a stable ID and host value; None shows the format hint.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut Option<Date>) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            min: None,
            max: None,
            today: None,
            name: None,
            size: ControlSize::Medium,
            enabled: true,
            width: None,
        }
    }
    /// Earliest selectable date.
    pub fn min(mut self, min: Date) -> Self {
        self.min = Some(min);
        self
    }
    /// Latest selectable date.
    pub fn max(mut self, max: Date) -> Self {
        self.max = Some(max);
        self
    }
    /// The host's current date, outlined in the calendar and used as the anchor.
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }
    /// Name for standalone use and for the calendar button; Field supplies one otherwise.
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
    /// Preferred outer width; defaults to all available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }

    fn anchor(&self) -> Date {
        self.value
            .or(self.today)
            .or(self.min)
            .or(self.max)
            .unwrap_or(FALLBACK)
            .clamp_to(self.min, self.max)
    }

    fn formatted(&self) -> String {
        self.value.map_or_else(String::new, |date| date.to_string())
    }

    fn commit(&mut self, date: Option<Date>) -> bool {
        let date = date.map(|date| date.clamp_to(self.min, self.max));
        let changed = date != *self.value;
        *self.value = date;
        changed
    }

    /// Up/Down step the caret's segment while the editor has focus.
    fn step_by_keys(&mut self, ui: &mut Ui, id: Id) -> bool {
        if !typed::editable_focus(ui, id) {
            return false;
        }
        let delta = ui.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                1
            } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                -1
            } else {
                0
            }
        });
        if delta == 0 {
            return false;
        }
        let current = typed::draft(ui.ctx(), id, &self.formatted());
        let index = typed::caret_segment(ui.ctx(), id, &current, '-').min(2);
        let Some(next) = self.anchor().step(SEGMENTS[index], delta) else {
            return false;
        };
        let changed = self.commit(Some(next));
        typed::retain(ui.ctx(), id, true, &self.formatted());
        changed
    }

    fn parse(&mut self, text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return self.commit(None);
        }
        match Date::parse(trimmed) {
            Some(date) => self.commit(Some(date)),
            None => false,
        }
    }

    fn editor(&mut self, ui: &mut Ui, id: Id, rect: Rect) -> (Response, bool) {
        let mut text = typed::draft(ui.ctx(), id, &self.formatted());
        let mut editor_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        editor_ui.visuals_mut().override_text_color = None;
        // egui's single-line editor is at least `interact_size.y` tall; match the slot.
        editor_ui.spacing_mut().interact_size.y = rect.height();
        let response = TextEdit::singleline(&mut text)
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
            .hint_text("YYYY-MM-DD")
            .show(&mut editor_ui)
            .response
            .response;
        let changed = response.changed() && self.parse(&text);
        if response.has_focus() {
            lock_arrows(ui, id, Axis::Vertical);
        }
        typed::retain(ui.ctx(), id, response.has_focus(), &text);
        (response, changed)
    }

    /// The calendar button and its popup; returns whether the value changed.
    fn calendar(&mut self, ui: &mut Ui, id: Id, rect: Rect, field: Rect, theme: Theme) -> bool {
        let open_id = id.with("calendar-open");
        let mut open = ui.is_enabled()
            && ui
                .ctx()
                .data(|data| data.get_temp::<bool>(open_id))
                .unwrap_or(false);
        let button_ui = ui.new_child(UiBuilder::new().max_rect(rect));
        let button = button_ui.interact(rect, id.with("choose"), Sense::click());
        let label = self
            .name
            .as_ref()
            .map_or_else(|| "Choose date".to_owned(), |name| format!("Choose {name}"));
        button.widget_info(|| WidgetInfo::labeled(Role::Button, button_ui.is_enabled(), &label));
        button.ctx.accesskit_node_builder(button.id, |node| {
            node.set_has_popup(egui::accesskit::HasPopup::Dialog);
            node.set_expanded(open);
        });
        paint_calendar_button(&button_ui, &button, rect, theme);
        if button.clicked() {
            open = !open;
            if open {
                calendar_grid::reset(ui, id, self.anchor());
            }
        }
        if open && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
            open = false;
            button.request_focus();
        }
        let mut changed = false;
        if open {
            let p = theme.palette;
            let frame = Frame::NONE
                .fill(p.background)
                .stroke(Stroke::new(border::STANDARD, p.border))
                .inner_margin(Margin::same(space::LG as i8));
            let (value, today, bounds) = (
                *self.value,
                self.today,
                Bounds {
                    min: self.min,
                    max: self.max,
                },
            );
            // Hang the grid below the whole field, aligned with its start; the
            // button only toggles it.
            let outcome = {
                crate::commands::dispatch::mark_overlay(ui.ctx());
                egui::Popup::new(id.with("calendar"), ui.ctx().clone(), field, ui.layer_id())
                    .anchor_widget(button.id)
                    .align(egui::RectAlign::BOTTOM_START)
                    .open_bool(&mut open)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .gap(space::XS)
                    .frame(frame)
                    .show(|ui| calendar_grid::show(ui, id, value, today, bounds))
            }
            .map(|inner| inner.inner);
            if let Some(Outcome::Selected(date)) = outcome {
                changed = self.commit(Some(date));
                open = false;
                button.request_focus();
            }
        }
        ui.ctx().data_mut(|data| data.insert_temp(open_id, open));
        changed
    }
}

impl Widget for DatePicker<'_> {
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
        let inner = outer.shrink(border::STANDARD);
        let square = inner.height();
        let editor_rect = Rect::from_min_max(
            inner.min,
            pos2((inner.right() - square).max(inner.left()), inner.max.y),
        );
        let button_rect =
            Rect::from_min_size(pos2(editor_rect.right(), inner.top()), vec2(square, square));
        style::paint_frame(ui, outer, theme);
        let mut changed = self.step_by_keys(ui, id);
        let (mut response, typed) = self.editor(ui, id, editor_rect);
        changed |= typed;
        changed |= self.calendar(ui, id, button_rect, outer, theme);
        if changed {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        if let Some(name) = self.name.clone() {
            response = response.accessible_name(name.into_owned());
        }
        let mut outline = response.clone();
        outline.rect = outer;
        style::finish(ui, &outline);
        response
    }
}

/// Divider, hover fill and a small calendar glyph: an outlined square with a header bar.
fn paint_calendar_button(ui: &Ui, response: &Response, rect: Rect, theme: Theme) {
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
    let glyph = Rect::from_center_size(
        rect.center(),
        vec2(control::STEP_GLYPH, control::STEP_GLYPH),
    );
    ui.painter().rect_stroke(
        glyph,
        0,
        Stroke::new(border::THIN, p.foreground),
        StrokeKind::Inside,
    );
    ui.painter().rect_filled(
        Rect::from_min_size(
            glyph.min,
            vec2(glyph.width(), border::STANDARD + border::THIN),
        ),
        0,
        p.foreground,
    );
    paint_focus_inside(ui, response, p.primary);
}
