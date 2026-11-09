use super::CommandRegistry;
use crate::{MenuItem, ToolbarItem};
use egui::Context;
impl<T: Clone + PartialEq + egui::AsIdSalt> CommandRegistry<T> {
    /// Owned menu description with current name, availability, check and shortcut.
    pub fn menu_item(&self, ctx: &Context, value: &T) -> Option<MenuItem<'static, T>> {
        let command = self.get(value)?;
        let mut item =
            MenuItem::new(command.value.clone(), command.label.clone()).enabled(command.enabled);
        item.checked = command.checked;
        if let Some(hint) = self.shortcut_label(ctx, value) {
            item = item.shortcut(hint);
        }
        Some(item)
    }
    /// Owned toolbar description using the same command metadata and key hint.
    pub fn toolbar_item(&self, ctx: &Context, value: &T) -> Option<ToolbarItem<'static, T>> {
        let command = self.get(value)?;
        let mut item = ToolbarItem::action(command.value.clone(), command.label.clone())
            .enabled(command.enabled);
        item.pressed = command.checked;
        item.icon = command.icon;
        let hint = self.shortcut_label(ctx, value).map_or_else(
            || command.label.clone(),
            |hint| format!("{} ({hint})", command.label),
        );
        Some(item.tooltip(hint))
    }
    /// Flat palette descriptions in catalog order.
    pub fn menu_items(&self, ctx: &Context) -> Vec<MenuItem<'static, T>> {
        self.commands()
            .iter()
            .filter_map(|command| self.menu_item(ctx, &command.value))
            .collect()
    }
}
