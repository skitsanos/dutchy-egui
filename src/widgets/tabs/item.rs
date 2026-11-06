use egui::IdSalt;
use std::borrow::Cow;

/// One tab, keyed by a unique durable host value.
#[derive(Clone, Debug)]
pub struct Tab<'a, T> {
    /// Host selection key and default widget identity.
    pub value: T,
    /// Visible and accessible name.
    pub label: Cow<'a, str>,
    /// Whether the tab can be focused, selected, or closed.
    pub enabled: bool,
    /// Whether closing may be requested for this tab.
    pub closable: bool,
    pub(super) id: IdSalt,
}

impl<'a, T: egui::AsIdSalt> Tab<'a, T> {
    /// Create a tab with identity independent of its label and position.
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&value),
            value,
            label: label.into(),
            enabled: true,
            closable: false,
        }
    }
    /// Disable this tab without removing it.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Offer a named close button and Delete shortcut.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}
