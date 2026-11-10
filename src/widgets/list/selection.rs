use super::{ListSelectionMode, ListState, model::Model};
use egui::Modifiers;
use std::{collections::HashSet, hash::Hash};

/// Command on macOS, Control elsewhere. egui reports either as `command`;
/// a bare macOS Control press is the secondary click, not a toggle.
pub(in crate::widgets) fn additive(modifiers: Modifiers) -> bool {
    modifiers.command || modifiers.mac_cmd
}

pub(super) fn pointer_modifiers(ui: &egui::Ui, rect: egui::Rect) -> Modifiers {
    ui.input(|i| {
        i.events
            .iter()
            .rev()
            .find_map(|event| match event {
                egui::Event::PointerButton {
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers,
                    pos,
                    ..
                } if rect.contains(*pos) => Some(*modifiers),
                _ => None,
            })
            .unwrap_or(i.modifiers)
    })
}

/// Apply one selection gesture and report whether the selected set changed.
pub(super) fn select<K: Clone + Eq + Hash>(
    state: &mut ListState<K>,
    model: &Model<'_, K>,
    mode: ListSelectionMode,
    index: usize,
    modifiers: Modifiers,
    toggle: bool,
) -> bool {
    let key = &model.items[index].key;
    match mode {
        ListSelectionMode::None => false,
        ListSelectionMode::Multiple if modifiers.shift => range(state, model, index, modifiers),
        _ => {
            state.anchor = Some(key.clone());
            if toggle && additive(modifiers) {
                if !state.selected.remove(key) {
                    if mode == ListSelectionMode::Single {
                        state.selected.clear();
                    }
                    state.selected.insert(key.clone());
                }
                true
            } else {
                replace(&mut state.selected, key)
            }
        }
    }
}

/// Select the enabled rows from the anchor to `index`, adding to the existing
/// selection when the additive modifier is held.
fn range<K: Clone + Eq + Hash>(
    state: &mut ListState<K>,
    model: &Model<'_, K>,
    index: usize,
    modifiers: Modifiers,
) -> bool {
    let anchor = model
        .index(state.anchor.as_ref())
        .or_else(|| model.index(state.focused.as_ref()))
        .unwrap_or(index);
    state.anchor = Some(model.items[anchor].key.clone());
    let span = anchor.min(index)..=anchor.max(index);
    let keys = model
        .enabled
        .iter()
        .filter(|i| span.contains(i))
        .map(|&i| model.items[i].key.clone());
    if additive(modifiers) {
        keys.fold(false, |changed, key| state.selected.insert(key) | changed)
    } else {
        let keys: HashSet<K> = keys.collect();
        let changed = state.selected != keys;
        state.selected = keys;
        changed
    }
}

fn replace<K: Clone + Eq + Hash>(selected: &mut HashSet<K>, key: &K) -> bool {
    if selected.len() == 1 && selected.contains(key) {
        return false;
    }
    selected.clear();
    selected.insert(key.clone());
    true
}

pub(in crate::widgets) fn select_focused<K: Clone + Eq + Hash>(
    state: &mut ListState<K>,
    mode: ListSelectionMode,
    key: &K,
) -> bool {
    if mode == ListSelectionMode::None {
        return false;
    }
    state.anchor = Some(key.clone());
    replace(&mut state.selected, key)
}
