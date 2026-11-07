use super::{style, typed};
use crate::{
    ControlSize, Family, Weight,
    calendar::{Time, TimeSegment},
    fonts,
    widgets::navigation::{Axis, lock_arrows},
};
use egui::{Align, Id, IdSalt, Key, Layout, Modifiers, Response, TextEdit, Ui, Widget, vec2};
use std::borrow::Cow;

const SEGMENTS: [TimeSegment; 3] = [TimeSegment::Hour, TimeSegment::Minute, TimeSegment::Second];

/// `HH:MM` or `HH:MM:SS` editor over an application-owned optional time.
///
/// Typed text commits when it parses and clears the value when emptied; it
/// reverts to the committed value on blur. Up/Down step the segment under the
/// caret, wrapping inside that segment. Time zones stay with the host.
#[must_use = "Add this time picker to a Ui"]
pub struct TimePicker<'a> {
    id: IdSalt,
    value: &'a mut Option<Time>,
    seconds: bool,
    name: Option<Cow<'a, str>>,
    size: ControlSize,
    enabled: bool,
    width: Option<f32>,
}

impl<'a> TimePicker<'a> {
    /// Bind a stable ID and host value; None shows the format hint.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut Option<Time>) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            seconds: false,
            name: None,
            size: ControlSize::Medium,
            enabled: true,
            width: None,
        }
    }
    /// Edit and show seconds as a third segment.
    pub fn seconds(mut self, seconds: bool) -> Self {
        self.seconds = seconds;
        self
    }
    /// Name for standalone use; Field supplies one otherwise.
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

    fn formatted(&self) -> String {
        self.value
            .map_or_else(String::new, |time| time.format(self.seconds))
    }

    /// Up/Down step the caret's segment while the editor has focus.
    fn step_by_keys(&mut self, ui: &mut Ui, id: Id) -> bool {
        if !self.enabled || !typed::editable_focus(ui, id) {
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
        let last = if self.seconds { 2 } else { 1 };
        let index = typed::caret_segment(ui.ctx(), id, &current, ':').min(last);
        let next = self.value.unwrap_or_default().step(SEGMENTS[index], delta);
        let changed = *self.value != Some(next);
        *self.value = Some(next);
        typed::retain(ui.ctx(), id, true, &next.format(self.seconds));
        changed
    }

    fn parse(&mut self, text: &str) -> bool {
        let trimmed = text.trim();
        let parsed = if trimmed.is_empty() {
            None
        } else {
            match Time::parse(trimmed) {
                Some(time) => Some(time),
                None => return false,
            }
        };
        let changed = parsed != *self.value;
        *self.value = parsed;
        changed
    }
}

impl Widget for TimePicker<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id);
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());
        let height = self.size.height();
        let mut changed = self.step_by_keys(ui, id);
        let mut text = typed::draft(ui.ctx(), id, &self.formatted());
        let hint = if self.seconds { "HH:MM:SS" } else { "HH:MM" };
        let mut response = ui
            .allocate_ui_with_layout(vec2(width, height), Layout::top_down(Align::Min), |ui| {
                ui.style_mut().spacing.interact_size.y = height;
                ui.visuals_mut().override_text_color = None;
                if !self.enabled {
                    ui.disable();
                }
                let frame = style::frame(ui, self.size);
                let response = TextEdit::singleline(&mut text)
                    .id(id)
                    .frame(frame)
                    .align(egui::Align2::LEFT_CENTER)
                    .desired_width(width)
                    .min_size(vec2(width, height))
                    .font(fonts::font_id(
                        Family::Body,
                        Weight::Regular,
                        self.size.text_size().points(),
                    ))
                    .hint_text(hint)
                    .show(ui)
                    .response
                    .response;
                style::finish(ui, &response);
                response
            })
            .inner;
        if response.changed() {
            changed |= self.parse(&text);
        }
        if response.has_focus() {
            lock_arrows(ui, id, Axis::Vertical);
        }
        typed::retain(ui.ctx(), id, response.has_focus(), &text);
        if changed {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        if let Some(name) = self.name {
            response = response.accessible_name(name.into_owned());
        }
        response
    }
}
