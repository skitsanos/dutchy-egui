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

/// The rest of the gesture that started a session, which must not edit the
/// selected name: a double/triple click on the new editor, or a held Space.
#[derive(Clone, Copy)]
enum Gesture {
    Pointer { last: f64 },
    Space,
}

fn gesture_id(ui: &Ui) -> Id {
    ui.scope_id().with("start-gesture")
}

/// Remember how a session started; keyboard starts other than Space need no guard.
pub(super) fn arm(ui: &Ui, pointer: bool, space: bool) {
    let key = gesture_id(ui);
    let gesture = if pointer {
        Some(Gesture::Pointer {
            last: ui.input(|i| i.time),
        })
    } else {
        space.then_some(Gesture::Space)
    };
    if let Some(gesture) = gesture {
        ui.data_mut(|data| data.insert_temp(key, gesture));
    } else {
        disarm(ui);
    }
}

pub(super) fn disarm(ui: &Ui) {
    let key = gesture_id(ui);
    ui.data_mut(|data| {
        data.remove::<Gesture>(key);
    });
}

/// While the Space press that started the session is held, drop its repeats
/// and typed spaces before the editor sees them.
pub(super) fn swallow_held_space(ui: &Ui) {
    if !matches!(
        ui.data(|d| d.get_temp(gesture_id(ui))),
        Some(Gesture::Space)
    ) {
        return;
    }
    ui.input_mut(|input| {
        input.events.retain(|event| match event {
            Event::Key {
                key: Key::Space,
                pressed: true,
                ..
            } => false,
            Event::Text(text) => text != " ",
            _ => true,
        })
    });
    if !ui.input(|i| i.key_down(Key::Space)) {
        disarm(ui);
    }
}

/// Whether a pointer press on the editor still belongs to the click sequence
/// that started the session, so the whole name should stay selected. Holding
/// the button extends the sequence; it ends one double-click delay later.
pub(super) fn continues_start_click(ui: &Ui, editor: &egui::Response) -> bool {
    let Some(Gesture::Pointer { last }) = ui.data(|d| d.get_temp(gesture_id(ui))) else {
        return false;
    };
    let now = ui.input(|i| i.time);
    let window = ui.ctx().options(|o| o.input_options.max_double_click_delay);
    if now - last > window {
        disarm(ui);
        return false;
    }
    let pressed = editor.is_pointer_button_down_on()
        || editor.clicked()
        || editor.double_clicked()
        || editor.triple_clicked();
    if pressed {
        ui.data_mut(|d| d.insert_temp(gesture_id(ui), Gesture::Pointer { last: now }));
    }
    pressed
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
