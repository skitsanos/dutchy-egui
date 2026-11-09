use egui::{Context, Event, Id, Key, KeyboardShortcut, Modifiers, os::OperatingSystem};
/// Why an uncommitted chord was refused.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Rejection {
    /// Another host binding already uses the chord.
    Reserved,
    /// A bare activation or navigation key would take input from focused controls.
    Modifier,
}
#[derive(Clone, Default)]
pub(super) struct Memory {
    pub recording: bool,
    pub error: Option<Rejection>,
    pub button: Option<Id>,
    pub consumed_frame: Option<u64>,
}
pub(super) enum Captured {
    None,
    Cancel,
    Shortcut(KeyboardShortcut),
}
pub(super) fn take(ctx: &Context) -> Captured {
    let mac = ctx.os() == OperatingSystem::Mac;
    let mut captured = Captured::None;
    ctx.input_mut(|input| {
        input.events.retain(|event| match event {
            Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } if matches!(captured, Captured::None) => {
                if *key == Key::Tab {
                    captured = Captured::Cancel;
                    return true;
                }
                captured = if *key == Key::Escape {
                    Captured::Cancel
                } else {
                    Captured::Shortcut(KeyboardShortcut::new(portable(*modifiers, mac), *key))
                };
                false
            }
            Event::Key { .. } | Event::Text(_) => false,
            _ => true,
        });
    });
    captured
}
/// Store the platform Command key as portable COMMAND. On macOS, Command and
/// Control are separate keys, so Control stays part of the chord.
fn portable(modifiers: Modifiers, mac: bool) -> Modifiers {
    let mut result = modifiers;
    if mac {
        result.command |= result.mac_cmd;
    } else if result.command || result.ctrl {
        result.command = true;
        result.ctrl = false;
    }
    result.mac_cmd = false;
    result
}

pub(super) fn suppress(ctx: &Context) {
    ctx.input_mut(|input| {
        input.events.retain(|event| {
            !matches!(event,
        Event::Key {key,..} if *key != Key::Tab)
                && !matches!(event, Event::Text(_))
        })
    });
}
