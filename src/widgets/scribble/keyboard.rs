use super::{
    ScribbleAction,
    capture::{Outcome, Source, State},
};
use crate::{ScribbleBrush, ScribbleDrawing, tokens::scribble as tokens};
use egui::{Key, Modifiers, Ui, Vec2, vec2};

pub(super) const HELP: &str = "Arrow keys move the pen. Space starts or finishes a stroke. Enter adds a dot or finishes. Escape cancels. Delete clears; Ctrl/Cmd+Z undoes, Shift+Ctrl/Cmd+Z redoes.";
pub(super) fn key(
    ui: &Ui,
    drawing: &mut ScribbleDrawing,
    state: &mut State,
    brush: ScribbleBrush,
    outcome: &mut Outcome,
    key: Key,
    modifiers: Modifiers,
) -> bool {
    let handled = if modifiers.command || modifiers.ctrl {
        shortcut(state, outcome, key, modifiers.shift)
    } else if !modifiers.alt {
        edit(drawing, state, brush, outcome, key, modifiers.shift)
    } else {
        false
    };
    if handled {
        ui.input_mut(|input| input.consume_key(modifiers, key));
    }
    handled
}
fn shortcut(state: &mut State, outcome: &mut Outcome, key: Key, shift: bool) -> bool {
    let action = match key {
        Key::Z if shift => ScribbleAction::Redo,
        Key::Z => ScribbleAction::Undo,
        Key::Y => ScribbleAction::Redo,
        _ => return false,
    };
    if state.draft.take().is_none() {
        outcome.actions.push(action);
    }
    true
}
fn edit(
    drawing: &mut ScribbleDrawing,
    state: &mut State,
    brush: ScribbleBrush,
    outcome: &mut Outcome,
    key: Key,
    shift: bool,
) -> bool {
    let delta = match key {
        Key::ArrowLeft => vec2(-1.0, 0.0),
        Key::ArrowRight => vec2(1.0, 0.0),
        Key::ArrowUp => vec2(0.0, -1.0),
        Key::ArrowDown => vec2(0.0, 1.0),
        _ => Vec2::ZERO,
    };
    if delta != Vec2::ZERO {
        let step = if shift {
            tokens::KEY_STEP_LARGE
        } else {
            tokens::KEY_STEP
        };
        let cursor = state.cursor.unwrap_or((drawing.size() / 2.0).to_pos2());
        let point = (cursor.to_vec2() + delta * step)
            .clamp(Vec2::ZERO, drawing.size())
            .to_pos2();
        if let Err(error) = state.append(point, drawing) {
            // The draft ends at the cap; the captured ink is kept.
            state.finish(drawing, outcome);
            outcome.error.get_or_insert(error);
        }
        return true;
    }
    match key {
        Key::Space => {
            if state.draft.is_some() {
                state.finish(drawing, outcome);
            } else {
                state.start(state.cursor.unwrap(), brush, Source::Keyboard, drawing);
            }
        }
        Key::Enter => {
            if state.draft.is_none() {
                state.start(state.cursor.unwrap(), brush, Source::Keyboard, drawing);
            }
            state.finish(drawing, outcome);
        }
        Key::Escape => {
            state.draft = None;
        }
        Key::Delete | Key::Backspace => {
            state.draft = None;
            outcome.actions.push(ScribbleAction::Clear);
        }
        _ => return false,
    }
    true
}
