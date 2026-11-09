//! Shared popup keyboard routing uses each key's modifier snapshot.
use egui::{Event, FocusDirection, Id, Key, Ui};

struct FocusRecorder;
impl egui::Plugin for FocusRecorder {
    fn debug_name(&self) -> &'static str {
        "dutchy-popup-focus"
    }
    fn on_begin_pass(&mut self, ui: &mut Ui) {
        let id = deferred_id(ui.ctx());
        let pending = ui
            .ctx()
            .data_mut(|data| data.remove_temp::<Option<(Id, FocusDirection)>>(id))
            .flatten();
        if let Some((trigger, direction)) = pending {
            ui.memory_mut(|memory| {
                if memory.focused() == Some(trigger) {
                    memory.move_focus(direction);
                }
            });
        }
    }
    fn on_end_pass(&mut self, ui: &mut Ui) {
        let focused = ui.memory(|m| m.focused());
        let id = snapshot_id(ui.ctx());
        ui.ctx().data_mut(|data| data.insert_temp(id, focused));
    }
}

fn snapshot_id(ctx: &egui::Context) -> Id {
    Id::unique(("dutchy-popup-focus-before-input", ctx.viewport_id()))
}

fn deferred_id(ctx: &egui::Context) -> Id {
    Id::unique(("dutchy-popup-focus-deferred-move", ctx.viewport_id()))
}

pub(crate) fn track(ctx: &egui::Context) {
    if ctx.plugin_opt::<FocusRecorder>().is_none() {
        ctx.add_plugin(FocusRecorder);
    }
}

pub(crate) fn previous_focus(ctx: &egui::Context) -> Option<Id> {
    let id = snapshot_id(ctx);
    ctx.data(|data| data.get_temp::<Option<Id>>(id).flatten())
}

pub(crate) fn tab_direction(ui: &Ui) -> Option<FocusDirection> {
    ui.input(|input| {
        input.events.iter().rev().find_map(|event| {
            let Event::Key {
                key: Key::Tab,
                pressed: true,
                modifiers,
                ..
            } = event
            else {
                return None;
            };
            if modifiers.shift_only() {
                Some(FocusDirection::Previous)
            } else if !modifiers.any() {
                Some(FocusDirection::Next)
            } else {
                None
            }
        })
    })
}

pub(crate) fn consume_tab(ui: &Ui) {
    ui.input_mut(|input| {
        input.events.retain(|event| {
            !matches!(event,
                Event::Key { key: Key::Tab, pressed: true, modifiers, .. }
                    if !modifiers.any() || modifiers.shift_only()
            )
        })
    });
}

pub(crate) fn move_past(ui: &Ui, trigger: Id, direction: FocusDirection) {
    consume_tab(ui);
    ui.memory_mut(|memory| {
        memory.request_focus(trigger);
        memory.move_focus(direction);
    });
}

/// Leave a popup past its trigger. A trigger the host already drew this pass
/// (`drawn`) registered for focus before the popup; egui only hands focus
/// onward while the focused widget registers, so that move starts at the
/// beginning of the next pass, before the host draws the trigger again.
pub(crate) fn leave(ui: &Ui, trigger: Id, direction: FocusDirection, drawn: bool) {
    if !drawn {
        return move_past(ui, trigger, direction);
    }
    consume_tab(ui);
    track(ui.ctx());
    ui.memory_mut(|memory| {
        memory.move_focus(FocusDirection::None);
        memory.request_focus(trigger);
    });
    let id = deferred_id(ui.ctx());
    ui.ctx()
        .data_mut(|data| data.insert_temp(id, Some((trigger, direction))));
    ui.ctx().request_repaint();
}
