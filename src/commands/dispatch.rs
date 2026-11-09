use super::{Command, CommandRegistry, CommandScope, ShortcutConflict};
use egui::{Context, Event, Id, ImeEvent, Key, KeyboardShortcut, Modifiers, os::OperatingSystem};

fn physical(modifiers: Modifiers, mac: bool) -> (bool, bool, bool, bool) {
    (
        modifiers.ctrl || (modifiers.command && !mac),
        modifiers.mac_cmd || (modifiers.command && mac),
        modifiers.alt,
        modifiers.shift,
    )
}
fn is_mac(ctx: &Context) -> bool {
    ctx.os() == OperatingSystem::Mac
}
/// Exact physical comparison: Control, Command, Alt and Shift must all agree.
fn chord_is(mac: bool, key: Key, modifiers: Modifiers, shortcut: KeyboardShortcut) -> bool {
    shortcut.logical_key == key && physical(modifiers, mac) == physical(shortcut.modifiers, mac)
}
/// Compare single chords including platform aliases and exact Shift/Alt modifiers.
pub fn same_shortcut(ctx: &Context, a: KeyboardShortcut, b: KeyboardShortcut) -> bool {
    chord_is(is_mac(ctx), a.logical_key, a.modifiers, b)
}
/// Keys that operate the focused control when pressed without Command, Control
/// or Alt: activation, focus movement and navigation. Shift alone stays bare.
pub(crate) fn operates_focus(key: Key, modifiers: Modifiers) -> bool {
    !(modifiers.ctrl || modifiers.command || modifiers.mac_cmd || modifiers.alt)
        && matches!(
            key,
            Key::Enter
                | Key::Space
                | Key::Tab
                | Key::Escape
                | Key::ArrowUp
                | Key::ArrowDown
                | Key::ArrowLeft
                | Key::ArrowRight
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown
        )
}
/// Input ownership observed once per dispatch.
struct Gate {
    mac: bool,
    focus: Option<Id>,
    editing: bool,
    modal: bool,
}
impl Gate {
    fn new(ctx: &Context) -> Self {
        Self {
            mac: is_mac(ctx),
            focus: ctx.memory(|memory| memory.focused()),
            editing: ctx.text_edit_focused(),
            modal: ctx.memory(|memory| memory.top_modal_layer()).is_some()
                || egui::Popup::is_any_open(ctx)
                || overlay_open(ctx),
        }
    }
    fn admits<T>(&self, entry: &Command<T>, key: Key, modifiers: Modifiers) -> bool {
        entry.enabled
            && (!self.editing || entry.allow_in_text)
            && (!self.modal || entry.allow_in_overlays)
            && entry
                .shortcut
                .is_some_and(|shortcut| chord_is(self.mac, key, modifiers, shortcut))
            && match entry.scope {
                // A focused control keeps its own activation and navigation keys.
                CommandScope::Application => {
                    self.focus.is_none() || !operates_focus(key, modifiers)
                }
                CommandScope::Focused(id) => self.focus == Some(id),
            }
    }
}

#[derive(Clone, Copy)]
struct Capture {
    owner: Id,
    frame: u64,
}
#[derive(Clone)]
struct Dispatched {
    frame: u64,
    consumed: Vec<KeyboardShortcut>,
}
fn capture_id(ctx: &Context) -> Id {
    Id::unique(("dutchy-shortcut-capture", ctx.viewport_id()))
}
pub(crate) fn set_capture(ctx: &Context, owner: Id, recording: bool) {
    let frame = ctx.cumulative_frame_nr();
    let key = capture_id(ctx);
    ctx.data_mut(|data| {
        if recording {
            data.insert_temp(key, Capture { owner, frame });
        } else if data
            .get_temp::<Capture>(key)
            .is_some_and(|capture| capture.owner == owner)
        {
            data.remove::<Capture>(key);
        }
    });
}
pub(crate) fn capturing(ctx: &Context) -> bool {
    let key = capture_id(ctx);
    ctx.data(|data| data.get_temp::<Capture>(key))
        .is_some_and(|capture| capture.frame.saturating_add(1) >= ctx.cumulative_frame_nr())
}
pub(crate) fn ime_active(ctx: &Context) -> bool {
    let key = Id::unique(("dutchy-command-ime", ctx.viewport_id()));
    let mut active = ctx.data(|data| data.get_temp::<bool>(key).unwrap_or(false));
    let composing = ctx.input(|input| {
        input
            .events
            .iter()
            .any(|event| matches!(event, Event::Ime(_)))
    });
    if !ctx.text_edit_focused() && !composing {
        active = false;
    }
    ctx.input(|input| {
        for event in &input.events {
            match event {
                Event::Ime(ImeEvent::Preedit { text, .. }) => active = !text.is_empty(),
                Event::Ime(ImeEvent::Commit(_)) => active = false,
                _ => {}
            }
        }
    });
    ctx.data_mut(|data| data.insert_temp(key, active));
    active || composing
}
fn overlay_id(ctx: &Context) -> Id {
    Id::unique(("dutchy-open-command-overlay", ctx.viewport_id()))
}
pub(crate) fn mark_overlay(ctx: &Context) {
    let id = overlay_id(ctx);
    let frame = ctx.cumulative_frame_nr();
    ctx.data_mut(|data| data.insert_temp(id, frame));
}
fn overlay_open(ctx: &Context) -> bool {
    let id = overlay_id(ctx);
    let frame = ctx.cumulative_frame_nr();
    ctx.data(|data| data.get_temp::<u64>(id))
        .is_some_and(|last| last.saturating_add(1) >= frame)
}
impl<T: Clone + PartialEq> CommandRegistry<T> {
    /// Report ambiguous enabled bindings. Focused bindings intentionally shadow
    /// application bindings; different focus targets can reuse a chord.
    pub fn conflicts(&self, ctx: &Context) -> Vec<ShortcutConflict<T>> {
        let mut result = Vec::new();
        for (index, a) in self.entries.iter().enumerate() {
            for b in self.entries.iter().skip(index + 1) {
                if a.enabled
                    && b.enabled
                    && a.scope == b.scope
                    && let (Some(left), Some(right)) = (a.shortcut, b.shortcut)
                    && same_shortcut(ctx, left, right)
                {
                    result.push(ShortcutConflict {
                        first: a.value.clone(),
                        second: b.value.clone(),
                        shortcut: left,
                    });
                }
            }
        }
        result
    }
    /// Consume eligible non-repeated key presses once per frame for this stable
    /// registry ID. Call before drawing controls. Ambiguous bindings do not fire.
    /// Text editors, IME, modals and KeyRecorder retain input unless opted in.
    pub fn dispatch(&self, ctx: &Context) -> Vec<super::CommandRequest<T>> {
        let frame = ctx.cumulative_frame_nr();
        let stamp = self.id.with(ctx.viewport_id());
        if let Some(previous) = ctx.data(|data| data.get_temp::<Dispatched>(stamp))
            && previous.frame == frame
        {
            let mac = is_mac(ctx);
            ctx.input_mut(|input| input.events.retain(|event| {
                !matches!(event, Event::Key {key,pressed:true,modifiers,..}
                    if previous.consumed.iter().any(|shortcut| chord_is(mac, *key, *modifiers, *shortcut)))
            }));
            return Vec::new();
        }
        ctx.data_mut(|data| {
            data.insert_temp(
                stamp,
                Dispatched {
                    frame,
                    consumed: Vec::new(),
                },
            )
        });
        if capturing(ctx) || ime_active(ctx) {
            return Vec::new();
        }
        let gate = Gate::new(ctx);
        let mut requests = Vec::new();
        let mut consumed = Vec::new();
        ctx.input_mut(|input| {
            input.events.retain(|event| {
                let Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } = event
                else {
                    return true;
                };
                let candidates: Vec<_> = self
                    .entries
                    .iter()
                    .filter(|entry| gate.admits(entry, *key, *modifiers))
                    .collect();
                let priority = candidates
                    .iter()
                    .map(|entry| matches!(entry.scope, CommandScope::Focused(_)) as u8)
                    .max();
                let mut best = candidates.into_iter().filter(|entry| {
                    Some(matches!(entry.scope, CommandScope::Focused(_)) as u8) == priority
                });
                let Some(entry) = best.next() else {
                    return true;
                };
                if best.next().is_some() {
                    return true;
                }
                if let Some(shortcut) = entry.shortcut {
                    consumed.push(shortcut);
                }
                if !requests
                    .iter()
                    .any(|request: &super::CommandRequest<T>| request.value == entry.value)
                {
                    requests.push(super::CommandRequest {
                        value: entry.value.clone(),
                        checked: entry.checked.map(|checked| !checked),
                    });
                }
                false
            });
        });
        ctx.data_mut(|data| data.insert_temp(stamp, Dispatched { frame, consumed }));
        requests
    }
}
