use egui::{EventFilter, Id, Key, Modifiers, Ui};

#[derive(Clone, Copy)]
pub(super) enum Axis {
    Horizontal,
    Vertical,
}

/// Move only within this component, skipping disabled entries supplied by callers.
pub(super) fn navigate(ui: &Ui, ids: &[Id], axis: Axis) {
    if !ui.is_enabled() {
        return;
    }
    let focused = ui.memory(|memory| memory.focused());
    let Some(current) = ids.iter().position(|id| Some(*id) == focused) else {
        return;
    };
    let (previous, next) = match axis {
        Axis::Horizontal => (Key::ArrowLeft, Key::ArrowRight),
        Axis::Vertical => (Key::ArrowUp, Key::ArrowDown),
    };
    let destination = ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, next) {
            Some((current + 1) % ids.len())
        } else if input.consume_key(Modifiers::NONE, previous) {
            Some((current + ids.len() - 1) % ids.len())
        } else if input.consume_key(Modifiers::NONE, Key::Home) {
            Some(0)
        } else if input.consume_key(Modifiers::NONE, Key::End) {
            Some(ids.len() - 1)
        } else {
            None
        }
    });
    if let Some(destination) = destination {
        ui.memory_mut(|memory| {
            memory.move_focus(egui::FocusDirection::None);
            memory.request_focus(ids[destination]);
        });
    }
}

pub(super) fn lock_arrows(ui: &Ui, id: Id, axis: Axis) {
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            id,
            EventFilter {
                horizontal_arrows: matches!(axis, Axis::Horizontal),
                vertical_arrows: matches!(axis, Axis::Vertical),
                ..Default::default()
            },
        )
    });
}
