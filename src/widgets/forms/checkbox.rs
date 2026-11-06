use egui::{IdSalt, Response, Ui, Widget};
use std::borrow::Cow;

/// Sharp checkbox with a wrapped, clickable label and native focus/activation.
#[must_use = "Add this checkbox to a Ui"]
pub struct Checkbox<'a> {
    id: IdSalt,
    checked: &'a mut bool,
    label: Cow<'a, str>,
    enabled: bool,
}

impl<'a> Checkbox<'a> {
    /// Bind a stable ID, host value, and accessible visible label.
    pub fn new(
        id: impl egui::AsIdSalt,
        checked: &'a mut bool,
        label: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            checked,
            label: label.into(),
            enabled: true,
        }
    }
    /// Disable interaction, also honoring a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id);
        ui.add_enabled(self.enabled, |ui: &mut Ui| {
            super::toggle::show(ui, id, &self.label, self.checked, false, true)
        })
    }
}
