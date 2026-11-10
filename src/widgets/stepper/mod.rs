mod item;
mod layout;
mod paint;
mod state;
use crate::{
    Progress, Text,
    tokens::{space, stepper as geometry},
};
use egui::{IdSalt, Response, Ui, UiBuilder, WidgetInfo};
pub use item::Step;
pub use state::StepperState;
use std::borrow::Cow;
/// Workflow progress presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum StepperVariant {
    /// Equal columns with filled indicators; automatically vertical in narrow columns.
    #[default]
    Horizontal,
    /// Leading square indicators, descriptions, and vertical connectors.
    Vertical,
    /// Numbered squares in a horizontally scrollable sequence.
    Compact,
    /// Position percentage track and reflowing step labels.
    Progress,
}
/// Geometry, resolved view position, and an optional host navigation request.
#[non_exhaustive]
pub struct StepperResponse<T> {
    /// Named workflow region.
    pub response: Response,
    /// Requested workflow key. The host decides whether to apply it.
    pub requested_step: Option<T>,
    /// Resolved one-based current position, or None for other/unknown states.
    pub current_step: Option<usize>,
    /// Position percentage, not a claim that the host's work is completed.
    pub progress: f64,
    /// Native step content responses in source order.
    pub steps: Vec<Response>,
    /// Effective presentation, including horizontal-to-vertical adaptation.
    pub rendered_variant: StepperVariant,
}
/// Controlled linear progress tracker. Does not validate fields, run work, or advance state.
#[must_use = "Call show to render the stepper"]
pub struct Stepper<'a, T> {
    id: IdSalt,
    state: &'a StepperState<T>,
    steps: &'a [Step<'a, T>],
    variant: StepperVariant,
    name: Cow<'a, str>,
    enabled: bool,
}
impl<'a, T: Clone + PartialEq> Stepper<'a, T> {
    /// Bind immutable host-owned workflow state and keyed steps.
    pub fn new(
        id: impl egui::AsIdSalt,
        state: &'a StepperState<T>,
        steps: &'a [Step<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            state,
            steps,
            variant: StepperVariant::default(),
            name: Cow::Borrowed("Workflow progress"),
            enabled: true,
        }
    }
    /// Presentation variant.
    pub fn variant(mut self, variant: StepperVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Workflow region name; the step list appends "steps" to this name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable optional navigation, respecting disabled parent UIs.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render status without changing state. Only explicitly navigable, enabled
    /// steps emit requests through native pointer/Enter/Space activation.
    pub fn show(self, ui: &mut Ui) -> StepperResponse<T> {
        let root = ui.make_persistent_id(("dutchy-stepper", self.id));
        let model = state::Model::new(self.state, self.steps);
        let variant = if self.variant == StepperVariant::Horizontal
            && ui.available_width() < self.steps.len() as f32 * geometry::MIN_COLUMN
        {
            StepperVariant::Vertical
        } else {
            self.variant
        };
        let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            if variant == StepperVariant::Progress {
                let progress = ui.add(
                    Progress::new(model.progress)
                        .id_salt("position")
                        .label(model.summary.as_str())
                        .accessible_name(format!("{} position", self.name))
                        .show_value(true),
                );
                ui.ctx().accesskit_node_builder(progress.id, |node| {
                    node.set_value(format!("{} · {:.0}%", model.summary, model.progress));
                });
                ui.add_space(space::SM);
            } else {
                ui.add(Text::label(model.summary.as_str()));
                ui.add_space(space::LG);
            }
            let list = ui.scope_builder(UiBuilder::new().scope_id(root.with("steps")), |ui| {
                layout::show(ui, root, self.steps, &model, variant)
            });
            list.response.widget_info(|| {
                WidgetInfo::labeled(
                    egui::accesskit::Role::List,
                    ui.is_enabled(),
                    format!("{} steps", self.name),
                )
            });
            ui.ctx().accesskit_node_builder(list.response.id, |node| {
                node.set_size_of_set(self.steps.len())
            });
            list.inner
        });
        let mut request = None;
        for (index, response) in result.inner.iter().enumerate() {
            if response.clicked() {
                response.request_focus();
                if model.current != Some(index) {
                    request = Some(self.steps[index].value.clone());
                }
            }
        }
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Group,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        ui.ctx().accesskit_node_builder(result.response.id, |node| {
            node.set_description(model.summary)
        });
        StepperResponse {
            response: result.response,
            requested_step: request,
            current_step: model.current.map(|index| index + 1),
            progress: model.progress,
            steps: result.inner,
            rendered_variant: variant,
        }
    }
}
