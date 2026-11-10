use crate::commands::dispatch::ime_active;
use egui::{Event, Id, Key, Ui};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Intent {
    Commit,
    Cancel,
}

pub(super) fn edit(ui: &Ui, editor: Id, actions: [Id; 2]) -> Option<Intent> {
    let focused = ui.memory(|m| m.focused());
    if !ui.is_enabled()
        || !focused.is_some_and(|id| id == editor || actions.contains(&id))
        || ime_active(ui.ctx())
    {
        return None;
    }
    let mut intent = None;
    ui.input_mut(|input| {
        input.events.retain(|event| {
            let Event::Key {
                key,
                pressed: true,
                modifiers,
                repeat,
                ..
            } = event
            else {
                return true;
            };
            if !modifiers.is_none() {
                return true;
            }
            let next = match key {
                Key::Escape => Some(Intent::Cancel),
                Key::Enter if focused == Some(editor) => Some(Intent::Commit),
                _ => None,
            };
            if !repeat && intent.is_none() {
                intent = next;
            }
            next.is_none()
        })
    });
    intent
}

pub(super) fn f2(ui: &Ui, id: Id) -> bool {
    if !ui.is_enabled() || !ui.memory(|m| m.has_focus(id)) || ime_active(ui.ctx()) {
        return false;
    }
    let mut start = false;
    ui.input_mut(|input| {
        input.events.retain(|event| {
            if let Event::Key {
                key: Key::F2,
                pressed: true,
                modifiers,
                repeat,
                ..
            } = event
                && modifiers.is_none()
            {
                start |= !repeat;
                false
            } else {
                true
            }
        })
    });
    start
}

/// Remove this frame's activation keys; returns whether Space was among them.
pub(super) fn consume_activation(ui: &Ui) -> bool {
    ui.input_mut(|input| {
        let space = input.events.iter().any(|e| matches!(e, Event::Key { key: Key::Space, pressed: true, modifiers, .. } if modifiers.is_none()));
        input.events.retain(|e| !matches!(e, Event::Key { key: Key::Enter | Key::Space, pressed: true, modifiers, .. } if modifiers.is_none())
            && !(space && matches!(e, Event::Text(text) if text == " ")));
        space
    })
}

pub(super) fn lock_escape(ui: &Ui, id: Id, editor: bool) {
    ui.memory_mut(|m| {
        m.set_focus_lock_filter(
            id,
            egui::EventFilter {
                escape: true,
                horizontal_arrows: editor,
                vertical_arrows: editor,
                ..Default::default()
            },
        )
    });
}
