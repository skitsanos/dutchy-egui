use super::{ToolbarEvent, ToolbarItem, focus::Memory};
use crate::widgets::popup_focus;
use crate::{Dropdown, MenuEvent};
use egui::{Id, Key, Response, Ui};

pub(super) fn tab_exit(ui: &Ui, memory: &mut Memory) {
    let owns = ui
        .memory(|m| m.focused())
        .is_some_and(|id| memory.popup_ids.contains(&id));
    if owns
        && let Some(direction) = popup_focus::tab_direction(ui)
        && let Some(trigger) = memory.open.take()
    {
        popup_focus::move_past(ui, trigger, direction);
    }
}

pub(super) fn show<T: Clone>(
    ui: &mut Ui,
    root: Id,
    entries: &[ToolbarItem<'_, T>],
    responses: &[(T, Response)],
    memory: &mut Memory,
) -> Option<ToolbarEvent<T>> {
    let activate = entries.iter().zip(responses).position(|(item, (_, r))| {
        item.menu.is_some()
            && r.enabled()
            && (r.clicked()
                || (r.has_focus()
                    && ui.input(|i| {
                        i.key_pressed(Key::ArrowDown)
                            || i.key_pressed(Key::ArrowUp)
                            || i.key_pressed(Key::ArrowRight)
                    })))
    });
    let selected = activate.or_else(|| {
        responses
            .iter()
            .position(|(_, r)| Some(r.id) == memory.open)
    });
    let Some(index) = selected else {
        memory.open = None;
        memory.popup_ids.clear();
        return None;
    };
    let (item, (_, trigger)) = (&entries[index], &responses[index]);
    let Some(items) = item.menu.filter(|_| item.interactive() && ui.is_enabled()) else {
        memory.open = None;
        memory.popup_ids.clear();
        return None;
    };
    let mut open = memory.open == Some(trigger.id);
    if trigger.has_focus()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowRight))
    {
        open = true;
    }
    let result = Dropdown::new(root.with(("toolbar-menu", item.id)), &mut open, items)
        .accessible_name(item.label.as_ref())
        .show_for(ui, trigger);
    memory.open = open.then_some(trigger.id);
    memory.popup_ids = result
        .menu
        .as_ref()
        .map_or_else(Vec::new, |m| m.focus_ids.clone());
    if open {
        memory.tab = Some(trigger.id);
    }
    result.menu.and_then(|m| m.event).map(|event| match event {
        MenuEvent::Activated(value) => ToolbarEvent::Activated(value),
        MenuEvent::Checked { value, checked } => ToolbarEvent::Toggled {
            value,
            pressed: checked,
        },
    })
}
