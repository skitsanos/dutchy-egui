use egui::{Id, Key, Modifiers, Response, Ui};

#[derive(Clone, Copy)]
pub(super) enum Movement {
    Previous,
    Next,
    First,
    Last,
}
#[derive(Default)]
pub(crate) struct SearchKeys {
    pub(super) movement: Option<Movement>,
    pub activate: bool,
    pub escape: bool,
    pub tab: bool,
}
impl SearchKeys {
    /// Read search navigation before TextEdit consumes Enter/arrow input.
    pub fn take(ui: &Ui, input: Id, open: bool) -> Self {
        if !ui.is_enabled() {
            return Self::default();
        }
        let focused = ui.memory(|memory| memory.has_focus(input));
        let mut keys = Self::default();
        if open {
            keys.escape = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
        }
        if !focused {
            return keys;
        }
        ui.input_mut(|input| {
            keys.tab = open && input.key_pressed(Key::Tab);
            keys.movement = if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                Some(Movement::Next)
            } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                Some(Movement::Previous)
            } else if open && input.consume_key(Modifiers::NONE, Key::Home) {
                Some(Movement::First)
            } else if open && input.consume_key(Modifiers::NONE, Key::End) {
                Some(Movement::Last)
            } else {
                None
            };
            keys.activate = open && input.consume_key(Modifiers::NONE, Key::Enter);
        });
        if keys.movement.is_some() {
            ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        }
        keys
    }
    pub fn opens(&self) -> bool {
        self.movement.is_some()
    }
}

pub(crate) fn input_metadata(input: &Response, open: bool, list: Option<Id>, active: Option<Id>) {
    input.ctx.accesskit_node_builder(input.id, |node| {
        node.set_role(egui::accesskit::Role::EditableComboBox);
        node.set_has_popup(egui::accesskit::HasPopup::Listbox);
        node.set_auto_complete(egui::accesskit::AutoComplete::List);
        node.set_expanded(open);
        if open && let Some(list) = list {
            node.push_controlled(list.accesskit_id());
        }
        if open && let Some(active) = active {
            node.set_active_descendant(active.accesskit_id());
        }
    });
    input.ctx.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            input.id,
            egui::EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                escape: true,
                ..Default::default()
            },
        )
    });
}
