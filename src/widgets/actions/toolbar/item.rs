use crate::{ButtonVariant, Icon};
use egui::IdSalt;
use std::borrow::Cow;

/// One toolbar action or controlled toggle. IDs derive from values, not labels.
#[derive(Clone, Debug)]
pub struct ToolbarItem<'a, T> {
    /// Value returned to the host on activation.
    pub value: T,
    /// Stable action name; optionally shown beside the icon.
    pub label: Cow<'a, str>,
    /// Optional decorative glyph.
    pub icon: Option<Icon>,
    /// None is an action; Some is the host's current pressed state.
    pub pressed: Option<bool>,
    /// Whether the action can activate.
    pub enabled: bool,
    /// Busy items are visible but cannot activate or receive toolbar focus.
    pub loading: bool,
    /// Action emphasis/intent.
    pub variant: ButtonVariant,
    /// Hide the label if an icon is present; defaults to true.
    pub icon_only: bool,
    /// Draw a separator before this item.
    pub separator_before: bool,
    /// Optional help; icon-only items default to their action name.
    pub tooltip: Option<Cow<'a, str>>,
    pub(super) id: IdSalt,
}
impl<'a, T: egui::AsIdSalt> ToolbarItem<'a, T> {
    /// Plain action. Values/explicit IDs must be unique within the toolbar.
    pub fn action(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&value),
            value,
            label: label.into(),
            icon: None,
            pressed: None,
            enabled: true,
            loading: false,
            variant: ButtonVariant::Ghost,
            icon_only: true,
            separator_before: false,
            tooltip: None,
        }
    }
    /// Controlled toggle. The toolbar returns a request; it never mutates state.
    pub fn toggle(value: T, label: impl Into<Cow<'a, str>>, pressed: bool) -> Self {
        Self {
            pressed: Some(pressed),
            ..Self::action(value, label)
        }
    }
}
impl<'a, T> ToolbarItem<'a, T> {
    /// Explicit stable salt for repeated action values.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = IdSalt::new(id);
        self
    }
    /// Decorative built-in or custom icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Show label with icon when false. Text remains visible without an icon.
    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = icon_only;
        self
    }
    /// Disabled items stay visible and are skipped by navigation.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Busy items display a spinner and are skipped by navigation.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Action treatment.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Optional group boundary before the item.
    pub fn separator_before(mut self, separate: bool) -> Self {
        self.separator_before = separate;
        self
    }
    /// Help override. Empty hides the icon-only default hint.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

/// Explicit host request; navigation alone never produces one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolbarEvent<T> {
    /// Run the named application action.
    Activated(T),
    /// Apply or reject the proposed new pressed state in the host.
    Toggled {
        /// Host action key.
        value: T,
        /// Requested new state.
        pressed: bool,
    },
}
