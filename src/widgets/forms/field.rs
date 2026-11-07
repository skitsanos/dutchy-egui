use super::style;
use crate::{
    Family, Spinner, SpinnerSize, Text, TextColor, TextSize, Theme, Tracking, Weight,
    tokens::space, validation::ValidationState,
};
use egui::accesskit::{Invalid, Live, Role};
use egui::{IdSalt, Response, Ui};
use std::borrow::Cow;

/// Visible label/help/validation decoration around one form control.
///
/// Returns the control's Response so change/blur validation observes the editor,
/// rather than the surrounding layout. Use `id_salt` for repeated field labels.
pub struct Field<'a> {
    label: Cow<'a, str>,
    id: Option<IdSalt>,
    help: Option<Cow<'a, str>>,
    validation: Option<&'a ValidationState>,
    required: bool,
}

impl<'a> Field<'a> {
    /// A labeled field with unchecked validation.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            id: None,
            help: None,
            validation: None,
            required: false,
        }
    }
    /// Stable namespace, needed when sibling fields have identical labels.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = Some(IdSalt::new(id));
        self
    }
    /// Secondary guidance, hidden by an error or pending message.
    pub fn help(mut self, help: impl Into<Cow<'a, str>>) -> Self {
        self.help = Some(help.into());
        self
    }
    /// Present host-owned validation state.
    pub fn validation(mut self, state: &'a ValidationState) -> Self {
        self.validation = Some(state);
        self
    }
    /// Show a required marker and expose required metadata; rules enforce it.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Render one control. No focus is moved when feedback changes.
    pub fn show(self, ui: &mut Ui, control: impl FnOnce(&mut Ui) -> Response) -> Response {
        let salt = self.id.unwrap_or_else(|| IdSalt::new(self.label.as_ref()));
        ui.scope_builder(egui::UiBuilder::new().id_salt(salt), |ui| {
            let theme = Theme::from_context(ui.ctx());
            self.set_border(ui, theme);
            let label = self.label(ui, theme);
            crate::layout::gap_after(ui, space::SM);
            let response = control(ui)
                .labelled_by(label.id)
                .accessible_name(self.label.to_string());
            if label.clicked() && response.enabled() && response.sense.is_focusable() {
                response.request_focus();
            }
            self.feedback(ui);
            self.metadata(&response);
            response
        })
        .inner
    }

    fn set_border(&self, ui: &mut Ui, theme: Theme) {
        match self.validation {
            Some(ValidationState::Invalid(_)) => style::set_border(ui, theme.palette.destructive),
            Some(ValidationState::Valid) => style::set_border(ui, theme.palette.success),
            _ => {}
        }
    }

    fn label(&self, ui: &mut Ui, theme: Theme) -> Response {
        // Inverse surfaces install a scoped text color; the label follows it.
        let inherited = ui.visuals().override_text_color;
        let mut job = Text::new(self.label.as_ref())
            .family(Family::Body)
            .size(TextSize::Xs)
            .weight(Weight::Bold)
            .tracking(Tracking::Wider)
            .uppercase(true)
            .color(inherited.map_or(TextColor::Foreground, TextColor::Custom))
            .layout_job(&theme, ui.available_width());
        if self.required {
            let mut format = job.sections[0].format.clone();
            format.color = inherited.unwrap_or(theme.palette.primary);
            job.append(" *", 0.0, format);
        }
        let response = ui.add(egui::Label::new(job).wrap().sense(egui::Sense::CLICK));
        crate::text::name_as_written(&response, self.label.as_ref());
        response
    }

    fn feedback(&self, ui: &mut Ui) {
        match self.validation {
            Some(ValidationState::Invalid(error)) => {
                let response = ui.add(
                    Text::new(error.as_str())
                        .size(TextSize::Sm)
                        .color(TextColor::Destructive),
                );
                response.ctx.accesskit_node_builder(response.id, |node| {
                    node.set_role(Role::Alert);
                    node.set_label(error.to_string());
                    node.set_live(Live::Assertive);
                });
            }
            Some(ValidationState::Pending) => {
                ui.horizontal(|ui| {
                    ui.add(Spinner::new().size(SpinnerSize::Small));
                    let message = ui.add(
                        Text::new("Validating…")
                            .size(TextSize::Sm)
                            .color(TextColor::Muted),
                    );
                    message.ctx.accesskit_node_builder(message.id, |node| {
                        node.set_role(Role::Status);
                        node.set_label("Validating…");
                        node.set_live(Live::Polite);
                    });
                });
            }
            _ => {
                if let Some(help) = &self.help {
                    ui.add(
                        Text::new(help.as_ref())
                            .size(TextSize::Sm)
                            .color(TextColor::Muted),
                    );
                }
            }
        }
    }

    fn metadata(&self, response: &Response) {
        response.ctx.accesskit_node_builder(response.id, |node| {
            if self.required {
                node.set_required();
            }
            let message = match self.validation {
                Some(ValidationState::Invalid(error)) => {
                    node.set_invalid(Invalid::True);
                    Some(error.as_str())
                }
                Some(ValidationState::Pending) => {
                    node.set_busy();
                    Some("Validating…")
                }
                _ => self.help.as_deref(),
            };
            if let Some(message) = message {
                let description = node.description().map_or_else(
                    || message.to_string(),
                    |existing| format!("{existing}. {message}"),
                );
                node.set_description(description);
            }
        });
    }
}
