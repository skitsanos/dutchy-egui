use egui::{Color32, IdSalt, Painter, Rect};
use std::borrow::Cow;

/// Decorative icon callback. The item's label supplies its accessible name.
pub type MenuIconPainter = fn(&Painter, Rect, Color32);

/// Action emphasis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuItemVariant {
    /// Standard action.
    #[default]
    Default,
    /// Destructive action, with destructive hover/focus fill.
    Destructive,
}

/// One host-owned action. Keys must be unique within the menu or command list.
#[derive(Clone, Debug)]
pub struct MenuItem<'a, T> {
    /// Value returned when activated; never executed by the library.
    pub value: T,
    /// Visible/accessibility name.
    pub label: Cow<'a, str>,
    /// Optional consecutive group label.
    pub group: Option<Cow<'a, str>>,
    /// Display-only shortcut hint; the host binds shortcuts.
    pub shortcut: Option<Cow<'a, str>>,
    /// Action treatment.
    pub variant: MenuItemVariant,
    /// Whether this item can activate.
    pub enabled: bool,
    /// Strong divider before this item.
    pub separator_before: bool,
    /// Optional repository/host-painted decorative icon.
    pub icon: Option<MenuIconPainter>,
    pub(crate) id: IdSalt,
}
impl<'a, T: egui::AsIdSalt> MenuItem<'a, T> {
    /// Stable identity derives from the value, independently of the label/order.
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&value),
            value,
            label: label.into(),
            group: None,
            shortcut: None,
            variant: MenuItemVariant::Default,
            enabled: true,
            separator_before: false,
            icon: None,
        }
    }
}
impl<'a, T> MenuItem<'a, T> {
    /// Consecutive equal group labels share a named group and heading.
    pub fn group(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.group = Some(label.into());
        self
    }
    /// Shortcut hint, without installing a global binding.
    pub fn shortcut(mut self, shortcut: impl Into<Cow<'a, str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    /// Action treatment.
    pub fn variant(mut self, variant: MenuItemVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Disable the action while keeping it visible.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Draw a strong separator before this action.
    pub fn separator_before(mut self, separate: bool) -> Self {
        self.separator_before = separate;
        self
    }
    /// Supply a decorative icon painter; no extra accessible node is created.
    pub fn icon(mut self, painter: MenuIconPainter) -> Self {
        self.icon = Some(painter);
        self
    }
}
