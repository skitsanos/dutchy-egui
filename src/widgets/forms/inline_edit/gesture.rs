//! Keep the gesture that opened a rename from acting on the new controls.
use egui::{Event, Id, Key, PointerButton, Pos2, Response, Ui};

#[derive(Clone, Copy)]
enum Gesture {
    Pointer { origin: Pos2, last: f64 },
    Space,
}

fn key(ui: &Ui) -> Id {
    ui.scope_id().with("start-gesture")
}

fn store(ui: &Ui, gesture: Gesture) {
    ui.data_mut(|data| data.insert_temp(key(ui), gesture));
}

pub(super) fn disarm(ui: &Ui) {
    ui.data_mut(|data| data.remove::<Gesture>(key(ui)));
}

pub(super) fn arm(ui: &Ui, trigger: &Response, space: bool) {
    if let Some(origin) = primary_position(trigger, false) {
        store(
            ui,
            Gesture::Pointer {
                origin,
                last: ui.input(|i| i.time),
            },
        );
    } else if space {
        store(ui, Gesture::Space);
    } else {
        disarm(ui);
    }
}

/// Filter only the held activation press, in event order. A release and a new
/// press may arrive together, so the final key state alone is insufficient.
pub(super) fn swallow_space(ui: &Ui, owned: [Id; 3]) {
    if !matches!(ui.data(|data| data.get_temp(key(ui))), Some(Gesture::Space)) {
        return;
    }
    if !ui.is_enabled() || !ui.memory(|m| m.focused().is_some_and(|id| owned.contains(&id))) {
        disarm(ui);
        return;
    }
    let held = ui.input_mut(|input| {
        let down = input.key_down(Key::Space);
        let mut held = down
            || input.events.iter().any(|event| {
                matches!(
                    event,
                    Event::Key {
                        key: Key::Space,
                        pressed: false,
                        ..
                    }
                )
            });
        input.events.retain(|event| {
            if !held {
                return true;
            }
            match event {
                Event::Key {
                    key: Key::Space,
                    pressed: false,
                    ..
                } => {
                    held = false;
                    true
                }
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    modifiers,
                    ..
                } => !modifiers.is_none(),
                Event::Text(text) => text != " ",
                _ => true,
            }
        });
        held && down
    });
    if !held {
        disarm(ui);
    }
}

/// Recognize a continuation on any newly drawn part of the editor, including
/// Save and Cancel. Use the same time and distance limits as egui's clicks.
pub(super) fn pointer_target(ui: &Ui, targets: [&Response; 3]) -> Option<Id> {
    let Some(Gesture::Pointer { origin, last }) = ui.data(|data| data.get_temp(key(ui))) else {
        return None;
    };
    let (now, down, pressed) = ui.input(|i| {
        (
            i.time,
            i.pointer.primary_down(),
            i.pointer.button_pressed(PointerButton::Primary),
        )
    });
    let (delay, distance) = ui.ctx().options(|o| {
        (
            o.input_options.max_double_click_delay,
            o.input_options.max_click_dist,
        )
    });
    if now - last >= delay {
        disarm(ui);
        return None;
    }
    let Some((id, position)) = targets
        .into_iter()
        .find_map(|response| primary_position(response, down).map(|pos| (response.id, pos)))
    else {
        if pressed && now > last {
            disarm(ui);
        }
        return None;
    };
    if position.distance_sq(origin) >= distance * distance {
        disarm(ui);
        return None;
    }
    store(ui, Gesture::Pointer { origin, last: now });
    Some(id)
}

fn primary_position(response: &Response, down: bool) -> Option<Pos2> {
    if response.clicked_by(PointerButton::Primary) {
        // A move after the click can replace the latest position in the same
        // frame. Match the release in the widget's coordinate space instead.
        let local = response.ctx.layer_transform_from_global(response.layer_id);
        let release = response.ctx.input(|input| {
            input.raw.events.iter().rev().find_map(|event| {
                let Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    ..
                } = event
                else {
                    return None;
                };
                let pos = local.map_or(*pos, |transform| transform * *pos);
                response.interact_rect.contains(pos).then_some(pos)
            })
        });
        release.or_else(|| response.interact_pointer_pos())
    } else if down && response.is_pointer_button_down_on() {
        response.interact_pointer_pos()
    } else {
        None
    }
}
