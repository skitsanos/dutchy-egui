use crate::{
    Family, Text, TextColor, TextSize, Theme,
    interaction::paint_focus,
    tokens::{border, control, space},
};
use egui::style::HandleShape;
use egui::{
    Align, IdSalt, Layout, Rect, Response, Shape, SliderClamping, Stroke, Ui, UiBuilder, Widget,
    pos2, vec2,
};
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
                let rail_index = ui.painter().add(Shape::Noop);
                let fill_index = ui.painter().add(Shape::Noop);
                let (min, max) = (*self.range.start(), *self.range.end());
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
                let p = theme.palette;
                let rect = response.rect;
                // egui derives the handle radius from the control height.
                let radius = rect.height() / 2.5;
                let positions = rect.x_range().shrink(radius);
                let fraction = if max > min {
                    ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                };
                let center_x = positions.min + (positions.max - positions.min) * fraction;
                let rail =
                    Rect::from_center_size(rect.center(), vec2(rect.width(), border::ACCENT));
                ui.painter()
                    .set(rail_index, Shape::rect_filled(rail, 0, p.border));
                ui.painter().set(
                    fill_index,
                    Shape::rect_filled(
                        Rect::from_min_max(rail.min, pos2(center_x, rail.max.y)),
                        0,
                        p.primary,
                    ),
                );
                let mut focus = response.clone();
                focus.rect = Rect::from_center_size(
                    pos2(center_x, rect.center().y),
                    vec2(2.0 * radius, 2.0 * radius),
                );
                paint_focus(ui, &focus, p.primary);
                if let Some(label) = &self.label {
                    ui.add_space(space::MD);
                    ui.allocate_ui_with_layout(
                        vec2(control::SLIDER_VALUE_WIDTH, rect.height()),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.add(
                                Text::new(label(value))
                                    .family(Family::Mono)
                                    .size(TextSize::Sm)
                                    .color(TextColor::Foreground),
                            );
                        },
                    );
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

/// Zero-height egui rail (painted here instead), square handle colors, no strokes.
fn configure(ui: &mut Ui, theme: Theme, rail_width: f32) {
    let p = theme.palette;
    let spacing = ui.spacing_mut();
    spacing.slider_width = rail_width;
    spacing.slider_rail_height = 0.0;
    spacing.interact_size.y = control::SLIDER_HEIGHT;
    let visuals = ui.visuals_mut();
    visuals.slider_trailing_fill = false;
    let widgets = &mut visuals.widgets;
    for (state, fill) in [
        (&mut widgets.noninteractive, p.primary),
        (&mut widgets.inactive, p.primary),
        (&mut widgets.hovered, p.foreground),
        (&mut widgets.active, p.foreground),
    ] {
        state.bg_fill = fill;
        state.fg_stroke = Stroke::NONE;
        state.expansion = 0.0;
    }
}
