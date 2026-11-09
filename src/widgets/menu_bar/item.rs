use crate::MenuItem;
use egui::IdSalt;
use std::borrow::Cow;

/// A stable top-level entry in an in-window application menu bar.
#[derive(Clone, Debug)]
pub struct MenuBarItem<'a, T> {
    /// Visible/accessibility name, independent of identity.
    pub label: Cow<'a, str>,
    /// Borrowed command tree.
    pub items: &'a [MenuItem<'a, T>],
    /// Disabled entries stay visible and are skipped by navigation.
    pub enabled: bool,
    pub(super) id: IdSalt,
}
impl<'a, T> MenuBarItem<'a, T> {
    /// Bind a unique immutable key, label and command tree.
    pub fn new(
        id: impl egui::AsIdSalt,
        label: impl Into<Cow<'a, str>>,
        items: &'a [MenuItem<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            label: label.into(),
            items,
            enabled: true,
        }
    }
    /// Disable this menu, including its descendants.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub(super) fn interactive(&self) -> bool {
        self.enabled && !self.items.is_empty()
    }
}

/// Host-owned top-level visibility. Child popup/focus paths are transient.
#[derive(Clone, Debug, Default)]
pub struct MenuBarState {
    pub(super) open: Option<IdSalt>,
}
impl MenuBarState {
    /// Open the menu with this immutable key. Missing/disabled keys close safely.
    pub fn open(&mut self, id: impl egui::AsIdSalt) {
        self.open = Some(IdSalt::new(id));
    }
    /// Close all top-level and nested menus.
    pub fn close(&mut self) {
        self.open = None;
    }
    /// Whether this key is the host's currently open top-level menu.
    pub fn is_open(&self, id: impl egui::AsIdSalt) -> bool {
        self.open == Some(IdSalt::new(id))
    }
}
