use egui::{Color32, IdSalt, Painter, Rect};
use std::borrow::Cow;
/// Decorative icon painter; the item's label remains its accessible name.
pub type BreadcrumbIconPainter = fn(&Painter, Rect, Color32);
/// One keyed destination. The last item is always the noninteractive current page.
#[derive(Clone, Debug)]
pub struct BreadcrumbItem<'a, T> {
    /// Destination returned to the host when an ancestor activates.
    pub value: T,
    /// Human-readable destination name.
    pub label: Cow<'a, str>,
    /// Whether the ancestor can activate.
    pub enabled: bool,
    /// Optional decorative icon replacing the visible label.
    pub icon: Option<BreadcrumbIconPainter>,
    pub(crate) id: IdSalt,
}
impl<'a, T: egui::AsIdSalt> BreadcrumbItem<'a, T> {
    /// Stable identity derives from the destination independently of its label/order.
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&value),
            value,
            label: label.into(),
            enabled: true,
            icon: None,
        }
    }
}
impl<T> BreadcrumbItem<'_, T> {
    /// Keep an unavailable ancestor visible without permitting activation.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Replace visible text with a decorative icon; preserve the named target.
    pub fn icon(mut self, icon: BreadcrumbIconPainter) -> Self {
        self.icon = Some(icon);
        self
    }
}
