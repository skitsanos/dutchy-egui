use super::Tabs;
use crate::widgets::navigation::{self, Axis};
use egui::{Id, Ui};

pub(super) fn trigger(root: Id, salt: egui::IdSalt) -> Id {
    root.with(salt).with("tab")
}
pub(super) fn close(root: Id, salt: egui::IdSalt) -> Id {
    root.with(salt).with("close")
}
pub(super) fn panel(root: Id, salt: egui::IdSalt) -> Id {
    root.with(salt).with("panel")
}

pub(super) fn prepare<T: Clone + PartialEq>(ui: &Ui, root: Id, tabs: &Tabs<'_, T>) {
    let ids: Vec<_> = tabs
        .items
        .iter()
        .filter(|item| item.enabled)
        .map(|item| trigger(root, item.id))
        .collect();
    let controls: Vec<_> = tabs
        .items
        .iter()
        .filter(|item| item.enabled)
        .flat_map(|item| {
            let mut ids = vec![trigger(root, item.id)];
            if item.closable {
                ids.push(close(root, item.id));
            }
            ids
        })
        .collect();
    let key = root.with("previous-focus-controls");
    let previous = ui
        .ctx()
        .data(|data| data.get_temp::<Vec<Id>>(key).unwrap_or_default());
    let focused = ui.memory(|memory| memory.focused());
    if ui.is_enabled()
        && focused.is_some_and(|id| previous.contains(&id) && !controls.contains(&id))
    {
        let destination = tabs
            .selected_index()
            .map(|index| trigger(root, tabs.items[index].id))
            .or(tabs.empty_focus);
        if let Some(destination) = destination {
            ui.memory_mut(|memory| memory.request_focus(destination));
        }
    }
    ui.ctx().data_mut(|data| data.insert_temp(key, controls));
    navigation::navigate(ui, &ids, Axis::Horizontal);
}

pub(super) fn tab_stop<T: Clone + PartialEq>(
    ui: &Ui,
    root: Id,
    tabs: &Tabs<'_, T>,
) -> Option<usize> {
    let focused = ui.memory(|memory| memory.focused());
    tabs.items
        .iter()
        .position(|item| {
            item.enabled
                && (focused == Some(trigger(root, item.id))
                    || focused == Some(close(root, item.id)))
        })
        .or_else(|| tabs.selected_index())
}
