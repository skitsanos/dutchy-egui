use super::slider_rail::{Rail, configure, handle_rect, paint_handle_focus, value_column};
use crate::{
    Theme,
    tokens::{control, space},
};
use egui::style::HandleShape;
use egui::{IdSalt, Rangef, Response, SliderClamping, Ui, UiBuilder, Widget};
use std::borrow::Cow;
use std::ops::RangeInclusive;

/// Range control over egui's slider: native drag, arrow-key stepping and the
/// Slider accessibility role, under a 4-point rail with a primary trailing fill
/// and a 20-point square handle.
#[must_use = "Add this slider to a Ui"]
pub struct Slider<'a> {
    id: IdSalt,
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    step: Option<f64>,
    name: Option<Cow<'a, str>>,
    label: Option<Box<dyn Fn(f64) -> String + 'a>>,
    width: Option<f32>,
    enabled: bool,
}

impl<'a> Slider<'a> {
    /// Bind a stable ID, host value and inclusive range.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            range,
            step: None,
            name: None,
            label: None,
            width: None,
            enabled: true,
        }
    }
    /// Snap dragged values and arrow-key steps to multiples of `step`.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step.abs());
        self
    }
    /// Name for standalone use; Field supplies one otherwise.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Show a host-formatted value beside the rail in a reserved column.
    pub fn value_label(mut self, format: impl Fn(f64) -> String + 'a) -> Self {
        self.label = Some(Box::new(format));
        self
    }
    /// Preferred outer width; defaults to all available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }
    /// Disable interaction, honoring a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        let total = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());
        let reserved = if self.label.is_some() {
            control::SLIDER_VALUE_WIDTH + space::MD
        } else {
            0.0
        };
        let rail_width = (total - reserved).max(0.0);
        let id = ui.make_persistent_id(self.id);
        ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
            ui.horizontal(|ui| {
                configure(ui, theme, rail_width);
                if !self.enabled {
                    ui.disable();
                }
                let rail = Rail::reserve(ui);
                let mut slider = egui::Slider::new(&mut *self.value, self.range.clone())
                    .show_value(false)
                    .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                    .trailing_fill(false)
                    .clamping(SliderClamping::Always);
                if let Some(step) = self.step {
                    slider = slider.step_by(step);
                }
                let mut response = ui.add(slider);
                let value = *self.value;
                let rect = response.rect;
                let handle = handle_rect(rect, &self.range, value);
                let filled = Rangef::new(rect.left(), handle.center().x);
                rail.paint(ui, theme, rect, filled);
                paint_handle_focus(ui, &response, response.id, handle, theme);
                if let Some(label) = &self.label {
                    ui.add_space(space::MD);
                    let width = control::SLIDER_VALUE_WIDTH;
                    value_column(ui, width, rect.height(), label(value));
                }
                if let Some(name) = self.name {
                    response = response.accessible_name(name.into_owned());
                }
                response
            })
            .inner
        })
        .inner
    }
}
