use super::slider_rail::{
    Rail, configure, handle_rect, minimum_rail_width, paint_handle_focus, value_column,
};
use crate::{
    Theme,
    tokens::{control, space},
};
use egui::accesskit::{Action, ActionData, ActionRequest, Role};
use egui::style::HandleShape;
use egui::{
    Align, Id, IdSalt, Layout, Rangef, Response, SliderClamping, Ui, UiBuilder, Widget, WidgetInfo,
};
use std::borrow::Cow;
use std::ops::RangeInclusive;

type RangeLabel<'a> = Box<dyn Fn(f64, f64) -> String + 'a>;

const EDIT_ACTIONS: [Action; 3] = [Action::SetValue, Action::Increment, Action::Decrement];

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
    /// Snap dragged values and arrow-key steps to increments from the range start.
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
    /// Set it inside a Field too to include the field name in each handle's name.
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
    /// Show host-formatted values beside the rail, or below it in a narrow layout.
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

    fn layout(&self, ui: &Ui) -> (f32, bool) {
        let minimum = minimum_rail_width(ui);
        let total = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width())
            .max(minimum);
        let beside = total - control::RANGE_SLIDER_VALUE_WIDTH - space::MD;
        if self.label.is_some() && beside >= minimum {
            (beside, true)
        } else {
            (total, false)
        }
    }

    fn show_group(&mut self, ui: &mut Ui, theme: Theme) -> Response {
        let builder = UiBuilder::new()
            .id_salt("range-group")
            .accessibility_role(Role::Group);
        let group = ui.scope_builder(builder, |ui| self.show_rail(ui, theme));
        // Field attaches its name and help to the group. Keep native drag and change flags.
        let response = group.response.union(group.inner);
        response.widget_info(|| {
            WidgetInfo::labeled(
                Role::Group,
                response.enabled(),
                self.name.as_deref().unwrap_or_default(),
            )
        });
        response
    }

    fn show_rail(&mut self, ui: &mut Ui, theme: Theme) -> Response {
        let names = self.names();
        let rail = Rail::reserve(ui);
        let rail_key = ui.scope_id().with("native-rail");
        let editable = ui.is_enabled() && ui.memory(|m| m.allows_interaction(ui.layer_id()));
        filter_assistive_edits(ui, rail_key, editable);
        let before = (*self.low, *self.high);
        let mut slider =
            egui::RangeSlider::new(&mut *self.low, &mut *self.high, self.range.clone())
                .show_value(false)
                .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                .min_separation(self.min_separation)
                .clamping(SliderClamping::Always);
        if let Some(step) = self.step {
            slider = slider.step_by(step);
        }
        let mut response = ui.add(slider);
        ui.data_mut(|data| data.insert_temp(rail_key, response.id));
        let (low, high) = (*self.low, *self.high);
        // egui measures change after its initial clamping. Hosts observe that correction too.
        if (low, high) != before {
            response.mark_changed();
        }
        let rect = response.rect;
        let handles = [low, high].map(|value| handle_rect(rect, &self.range, value));
        rail.paint(
            ui,
            theme,
            rect,
            Rangef::new(handles[0].center().x, handles[1].center().x).as_positive(),
        );
        for ((id, handle), name) in handle_ids(response.id).into_iter().zip(handles).zip(names) {
            paint_handle_focus(ui, &response, id, handle, theme);
            ui.ctx().accesskit_node_builder(id, |node| {
                node.set_label(name);
                if !editable {
                    for action in EDIT_ACTIONS {
                        node.remove_action(action);
                    }
                }
            });
        }
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(Role::GenericContainer);
            node.set_hidden();
        });
        response
    }

    fn show_label(&self, ui: &mut Ui, width: f32, height: f32, gap: f32) {
        if let Some(label) = &self.label {
            ui.add_space(gap);
            value_column(ui, width, height, label(*self.low, *self.high));
        }
    }
}

impl Widget for RangeSlider<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        let (rail_width, beside) = self.layout(ui);
        let id = ui.make_persistent_id(self.id);
        ui.scope_builder(
            UiBuilder::new()
                .scope_id(id)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                let response = ui
                    .horizontal(|ui| {
                        configure(ui, theme, rail_width);
                        let response = self.show_group(ui, theme);
                        if beside {
                            self.show_label(
                                ui,
                                control::RANGE_SLIDER_VALUE_WIDTH,
                                response.rect.height(),
                                space::MD,
                            );
                        }
                        response
                    })
                    .inner;
                if !beside {
                    self.show_label(ui, rail_width, 0.0, 0.0);
                }
                response
            },
        )
        .inner
    }
}

fn handle_ids(rail: Id) -> [Id; 2] {
    ["low", "high"].map(|end| rail.with(end))
}

fn filter_assistive_edits(ui: &Ui, rail_key: Id, editable: bool) {
    // An action targets a handle that egui already exposed in a previous frame.
    if let Some(rail) = ui.data(|data| data.get_temp::<Id>(rail_key)) {
        ui.input_mut(|input| {
            for id in handle_ids(rail) {
                input.consume_accesskit_action_requests(id, |request| {
                    blocked_edit(request, editable)
                });
            }
        });
    }
}

fn blocked_edit(request: &ActionRequest, editable: bool) -> bool {
    match request.action {
        Action::SetValue => {
            !editable || !matches!(request.data, Some(ActionData::NumericValue(v)) if v.is_finite())
        }
        Action::Increment | Action::Decrement => !editable,
        _ => false,
    }
}
