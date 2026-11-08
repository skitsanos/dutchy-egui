mod core;
mod toolbar;
use crate::{ButtonVariant, ControlSize, Icon};
use egui::{IdSalt, Response, Sense, Ui, Widget};
use std::borrow::Cow;
pub use toolbar::{Toolbar, ToolbarEvent, ToolbarItem, ToolbarOrientation, ToolbarResponse};

/// Named square icon action, with native pointer/keyboard behavior.
/// An empty tooltip disables the default name hint; the accessible name remains.
#[must_use = "Add the icon button to a Ui"]
pub struct IconButton<'a> {
    id: IdSalt,
    name: Cow<'a, str>,
    icon: Icon,
    variant: ButtonVariant,
    size: ControlSize,
    enabled: bool,
    loading: bool,
    tooltip: Option<Cow<'a, str>>,
}
impl<'a> IconButton<'a> {
    /// Bind stable identity, a descriptive action name and decorative glyph.
    pub fn new(id: impl egui::AsIdSalt, name: impl Into<Cow<'a, str>>, icon: Icon) -> Self {
        Self {
            id: IdSalt::new(id),
            name: name.into(),
            icon,
            variant: ButtonVariant::Ghost,
            size: ControlSize::Medium,
            enabled: true,
            loading: false,
            tooltip: None,
        }
    }
    /// Shared action treatment; Ghost by default.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Square Small/Medium/Large target.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disable activation, honoring disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Show a square spinner, expose busy and disable activation.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Override the default name hint; an empty string hides it.
    pub fn tooltip(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}
impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(("dutchy-icon-button", self.id));
        core::show(
            ui,
            core::Action {
                id,
                label: &self.name,
                icon: Some(self.icon),
                icon_only: true,
                pressed: None,
                variant: self.variant,
                size: self.size,
                enabled: self.enabled,
                loading: self.loading,
                tooltip: self.tooltip.as_deref(),
                inside_focus: false,
            },
            Sense::click(),
            None,
        )
    }
}

/// Two-state button bound to an application-owned bool. Its name stays stable.
/// `changed()` reports only user toggles; external host changes are not edits.
#[must_use = "Add the toggle button to a Ui"]
pub struct ToggleButton<'a> {
    id: IdSalt,
    on: &'a mut bool,
    label: Cow<'a, str>,
    icon: Option<Icon>,
    icon_only: bool,
    variant: ButtonVariant,
    size: ControlSize,
    enabled: bool,
    loading: bool,
    tooltip: Option<Cow<'a, str>>,
}
impl<'a> ToggleButton<'a> {
    /// Bind stable identity, the host value and a stable visible/accessible label.
    pub fn new(id: impl egui::AsIdSalt, on: &'a mut bool, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            on,
            label: label.into(),
            icon: None,
            icon_only: false,
            variant: ButtonVariant::Outline,
            size: ControlSize::Medium,
            enabled: true,
            loading: false,
            tooltip: None,
        }
    }
    /// Decorative built-in or host-painted icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Hide the visible label when an icon is present, retaining the name/hint.
    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = icon_only;
        self
    }
    /// Off-state action treatment; the pressed state uses the inverse surface.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Shared control size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disable activation without changing the host value.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Busy state with a spinner and no activation.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Optional help; icon-only toggles default to their name. Empty hides it.
    pub fn tooltip(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}
impl Widget for ToggleButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(("dutchy-toggle-button", self.id));
        let pressed = *self.on;
        core::show(
            ui,
            core::Action {
                id,
                label: &self.label,
                icon: self.icon,
                icon_only: self.icon_only,
                pressed: Some(pressed),
                variant: self.variant,
                size: self.size,
                enabled: self.enabled,
                loading: self.loading,
                tooltip: self.tooltip.as_deref(),
                inside_focus: false,
            },
            Sense::click(),
            Some(self.on),
        )
    }
}
