use super::geometry::{Axis, Bounds};
use crate::tokens::split_pane as tokens;
use egui::{
    Event, Id, Key, Rect, Ui,
    accesskit::{Action, ActionData},
};

pub(super) fn read(
    ui: &Ui,
    id: Id,
    rect: Rect,
    axis: Axis,
    bounds: Bounds,
    value: f32,
    read_only: bool,
) -> f32 {
    if !ui.is_enabled() || !ui.is_visible() || read_only {
        super::pointer::cancel(ui, id);
        return value;
    }
    let Some(response) = ui.ctx().read_response(id) else {
        return value;
    };
    let focused = response.has_focus();
    let events = ui.input_mut(|input| {
        let mut ours = Vec::new();
        input.events.retain(|event| {
            let take = match event {
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    focused
                        && !modifiers.alt
                        && !modifiers.ctrl
                        && !modifiers.command
                        && !modifiers.mac_cmd
                        && key_value(axis, bounds, value, *key, modifiers.shift).is_some()
                }
                Event::AccessKitActionRequest(r) => {
                    r.target_node == id.accesskit_id()
                        && matches!(
                            r.action,
                            Action::Increment | Action::Decrement | Action::SetValue
                        )
                }
                _ => false,
            };
            if take {
                ours.push(event.clone());
            }
            !take
        });
        ours
    });
    let mut next = value;
    for event in events {
        let candidate = match event {
            Event::Key { key, modifiers, .. } => {
                key_value(axis, bounds, next, key, modifiers.shift)
            }
            Event::AccessKitActionRequest(r) => assistive(next, bounds, r.action, r.data),
            _ => None,
        };
        if let Some(candidate) = candidate {
            next = bounds.clamp(candidate);
        }
    }
    super::pointer::read(ui, &response, rect, axis, bounds, next)
}
fn key_value(axis: Axis, bounds: Bounds, value: f32, key: Key, shift: bool) -> Option<f32> {
    let step = if shift {
        tokens::LARGE_STEP
    } else {
        tokens::STEP
    };
    match (axis, key) {
        (_, Key::Home) => Some(bounds.min),
        (_, Key::End) => Some(bounds.max),
        (Axis::Horizontal, Key::ArrowLeft) | (Axis::Vertical, Key::ArrowUp) => Some(value - step),
        (Axis::Horizontal, Key::ArrowRight) | (Axis::Vertical, Key::ArrowDown) => {
            Some(value + step)
        }
        _ => None,
    }
}
fn assistive(value: f32, bounds: Bounds, action: Action, data: Option<ActionData>) -> Option<f32> {
    match action {
        Action::Increment => Some(value + tokens::STEP),
        Action::Decrement => Some(value - tokens::STEP),
        Action::SetValue => {
            let value = match data? {
                ActionData::NumericValue(value) => value,
                ActionData::Value(value) => value.parse::<f64>().ok()?,
                _ => return None,
            };
            value
                .is_finite()
                .then(|| value.clamp(bounds.min as f64, bounds.max as f64) as f32)
        }
        _ => None,
    }
}
