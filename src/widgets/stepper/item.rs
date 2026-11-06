use egui::IdSalt;
use std::borrow::Cow;
/// One keyed workflow step. Navigation is opt-in and never runs work itself.
#[derive(Clone, Debug)]
pub struct Step<'a, T> {
    /// Stable workflow key returned in a navigation request.
    pub value: T,
    /// Visible title and accessible name.
    pub label: Cow<'a, str>,
    /// Supporting detail, visible vertically and exposed in every variant's metadata.
    pub description: Option<Cow<'a, str>>,
    /// Whether this step can emit navigation requests. Defaults to false.
    pub navigable: bool,
    /// Whether optional navigation is enabled. Does not change progress state.
    pub enabled: bool,
    pub(crate) id: IdSalt,
}
impl<'a, T: egui::AsIdSalt> Step<'a, T> {
    /// Derive stable identity from a unique immutable workflow key.
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&value),
            value,
            label: label.into(),
            description: None,
            navigable: false,
            enabled: true,
        }
    }
}
impl<'a, T> Step<'a, T> {
    /// Supporting instruction; it remains accessible in compact presentations.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Allow native pointer/Enter/Space navigation requests for this step.
    pub fn navigable(mut self, navigable: bool) -> Self {
        self.navigable = navigable;
        self
    }
    /// Disable optional navigation while preserving the step's workflow status.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
