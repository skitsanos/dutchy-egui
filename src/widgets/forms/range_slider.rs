use super::slider_rail::{Rail, configure, handle_rect, paint_handle_focus, value_column};
use crate::{
    Theme,
    tokens::{control, space},
};
use egui::accesskit::Role;
use egui::style::HandleShape;
use egui::{IdSalt, Rangef, Response, SliderClamping, Ui, UiBuilder, Widget};
use std::borrow::Cow;
use std::ops::RangeInclusive;

type RangeLabel<'a> = Box<dyn Fn(f64, f64) -> String + 'a>;

/// Two-ended range control over egui's range slider: native drag of the nearer
/// handle, arrow-key stepping and one Slider node per handle, under the Slider
/// rail with a primary fill between two 20-point square handles. The handles
/// may meet but never cross.
#[must_use = "Add this range slider to a Ui"]
pub struct RangeSlider<'a> {
    id: IdSalt,
    low: &'a mut f64,
    high: &'a mut f64,
    range: RangeInclusive<f64>,
    step: Option<f64>,
    min_separation: f64,
    name: Option<Cow<'a, str>>,
    handle_names: Option<(Cow<'a, str>, Cow<'a, str>)>,
    label: Option<RangeLabel<'a>>,
    width: Option<f32>,
    enabled: bool,
}

impl<'a> RangeSlider<'a> {
    /// Bind a stable ID, the host's low and high values and an inclusive range.
    /// Values outside the range, or in the wrong order, are pulled back in.
    pub fn new(
        id: impl egui::AsIdSalt,
        low: &'a mut f64,
        high: &'a mut f64,
        range: RangeInclusive<f64>,
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            low,
            high,
            range,
            step: None,
            min_separation: 0.0,
            name: None,
            handle_names: None,
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
    /// Closest the two values may come; zero lets the range collapse.
    pub fn min_separation(mut self, separation: f64) -> Self {
        self.min_separation = separation.max(0.0);
        self
    }
    /// Name the range; the handles announce it with "minimum" and "maximum".
    /// Set it inside a Field too, which can only name the rail.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Replace the generated handle names, for example in another language.
    pub fn handle_names(
        mut self,
        low: impl Into<Cow<'a, str>>,
        high: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.handle_names = Some((low.into(), high.into()));
        self
    }
    /// Show host-formatted values beside the rail in a reserved column.
    pub fn value_label(mut self, format: impl Fn(f64, f64) -> String + 'a) -> Self {
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

    fn names(&self) -> [String; 2] {
        if let Some((low, high)) = &self.handle_names {
            return [low.to_string(), high.to_string()];
        }
        match &self.name {
            Some(name) => [format!("{name} minimum"), format!("{name} maximum")],
            None => ["Minimum".to_owned(), "Maximum".to_owned()],
        }
    }

    fn rail_width(&self, ui: &Ui) -> f32 {
        let total = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());
        let reserved = if self.label.is_some() {
            control::RANGE_SLIDER_VALUE_WIDTH + space::MD
        } else {
            0.0
        };
        (total - reserved).max(0.0)
    }

    fn show(self, ui: &mut Ui, theme: Theme) -> Response {
        let names = self.names();
        let rail = Rail::reserve(ui);
        let mut slider =
            egui::RangeSlider::new(&mut *self.low, &mut *self.high, self.range.clone())
                .show_value(false)
                .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                .min_separation(self.min_separation)
                .clamping(SliderClamping::Always);
        if let Some(step) = self.step {
            slider = slider.step_by(step);
        }
        let response = ui.add(slider);
        let (low, high) = (*self.low, *self.high);
        let rect = response.rect;
        let handles = [low, high].map(|value| handle_rect(rect, &self.range, value));
        rail.paint(
            ui,
            theme,
            rect,
            Rangef::new(handles[0].center().x, handles[1].center().x),
        );
        // egui keys each handle's focus stop and Slider node off the rail's ID.
        let ids = ["low", "high"].map(|end| response.id.with(end));
        for ((id, handle), name) in ids.into_iter().zip(handles).zip(names) {
            paint_handle_focus(ui, &response, id, handle, theme);
            ui.ctx()
                .accesskit_node_builder(id, |node| node.set_label(name));
        }
        // The rail carries no value of its own; the two handles are the sliders.
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(Role::Group);
            if let Some(name) = &self.name {
                node.set_label(name.to_string());
            }
        });
        if let Some(label) = &self.label {
            ui.add_space(space::MD);
            value_column(
                ui,
                control::RANGE_SLIDER_VALUE_WIDTH,
                rect.height(),
                label(low, high),
            );
        }
        response
    }
}

impl Widget for RangeSlider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        let rail_width = self.rail_width(ui);
        let id = ui.make_persistent_id(self.id);
        ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
            ui.horizontal(|ui| {
                configure(ui, theme, rail_width);
                if !self.enabled {
                    ui.disable();
                }
                self.show(ui, theme)
            })
            .inner
        })
        .inner
    }
}
