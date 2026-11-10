use super::{ListSelectionMode, ListState, model::Model, selection};
use crate::tokens::list as geometry;
use egui::{Event, Key, Modifiers, Ui};
use std::hash::Hash;

pub(super) struct Outcome<K> {
    pub reveal: bool,
    pub activated: Option<K>,
    /// Whether an edit altered the selected set.
    pub changed: bool,
}

pub(super) fn read<K: Clone + Eq + Hash>(
    ui: &Ui,
    model: &Model<'_, K>,
    state: &mut ListState<K>,
    mode: ListSelectionMode,
    read_only: bool,
    page: usize,
) -> Outcome<K> {
    let mut outcome = Outcome {
        reveal: false,
        activated: None,
        changed: false,
    };
    if !ui.is_enabled() {
        return outcome;
    }
    assistive(ui, model, state, mode, read_only, &mut outcome);
    let focused = ui.memory(|m| m.focused());
    if !focused.is_some_and(|id| model.ids.contains(&id)) {
        state.query.clear();
        return outcome;
    }
    let events = ui.input_mut(|input| {
        let mut ours = Vec::new();
        let mut modifiers = input.modifiers;
        input.events.retain(|event| {
            if let Event::ModifiersChanged(value) = event {
                modifiers = *value;
                return true;
            }
            let take = match event {
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    !modifiers.alt
                        && matches!(
                            key,
                            Key::ArrowUp
                                | Key::ArrowDown
                                | Key::Home
                                | Key::End
                                | Key::PageUp
                                | Key::PageDown
                                | Key::Space
                                | Key::Enter
                        )
                        || matches!(event, Event::Key { key: Key::A, pressed: true, modifiers, .. }
                        if selection::additive(*modifiers) && !modifiers.alt)
                }
                Event::Text(_) => !modifiers.ctrl && !modifiers.command && !modifiers.alt,
                _ => false,
            };
            if take {
                ours.push(event.clone());
            }
            !take
        });
        ours
    });
    for event in events {
        match event {
            // Space continues an active prefix; its Text event extends the search.
            Event::Key {
                key: Key::Space, ..
            } if typing(ui, state) => {}
            Event::Key { key, modifiers, .. } => key_event(
                state,
                model,
                mode,
                (key, modifiers),
                read_only,
                page,
                &mut outcome,
            ),
            Event::Text(text) => type_ahead(ui, state, model, mode, &text, read_only, &mut outcome),
            _ => {}
        }
    }
    if outcome.reveal {
        ui.memory_mut(|m| {
            m.move_focus(egui::FocusDirection::None);
            if let Some(index) = model.index(state.focused.as_ref()) {
                m.request_focus(model.ids[index]);
            }
        });
    }
    outcome
}

fn key_event<K: Clone + Eq + Hash>(
    state: &mut ListState<K>,
    model: &Model<'_, K>,
    mode: ListSelectionMode,
    event: (Key, Modifiers),
    read_only: bool,
    page: usize,
    out: &mut Outcome<K>,
) {
    let (key, modifiers) = event;
    let Some(index) = model.index(state.focused.as_ref()) else {
        return;
    };
    let current = model.enabled.binary_search(&index).unwrap_or(0);
    if let Some(next) = destination(key, current, model.enabled.len(), page) {
        let next = model.enabled[next];
        if !read_only && (!selection::additive(modifiers) || modifiers.shift) {
            out.changed |= selection::select(state, model, mode, next, modifiers, false);
        }
        state.focused = Some(model.items[next].key.clone());
        state.query.clear();
        out.reveal = true;
        return;
    }
    if read_only {
        return;
    }
    match key {
        Key::A if selection::additive(modifiers) && mode == ListSelectionMode::Multiple => {
            let before = state.selected.len();
            state
                .selected
                .extend(model.enabled.iter().map(|&i| model.items[i].key.clone()));
            out.changed |= state.selected.len() != before;
        }
        Key::Space => {
            let modifiers = if mode == ListSelectionMode::Multiple && !modifiers.shift {
                Modifiers {
                    command: true,
                    ..modifiers
                }
            } else {
                modifiers
            };
            out.changed |= selection::select(state, model, mode, index, modifiers, true);
        }
        Key::Enter => out.activated = Some(model.items[index].key.clone()),
        _ => {}
    }
}

fn destination(key: Key, current: usize, count: usize, page: usize) -> Option<usize> {
    let last = count.saturating_sub(1);
    match key {
        Key::ArrowUp => Some(current.saturating_sub(1)),
        Key::ArrowDown => Some((current + 1).min(last)),
        Key::Home => Some(0),
        Key::End => Some(last),
        Key::PageUp => Some(current.saturating_sub(page)),
        Key::PageDown => Some(current.saturating_add(page).min(last)),
        _ => None,
    }
}

fn type_ahead<K: Clone + Eq + Hash>(
    ui: &Ui,
    state: &mut ListState<K>,
    model: &Model<'_, K>,
    mode: ListSelectionMode,
    text: &str,
    read_only: bool,
    out: &mut Outcome<K>,
) {
    let text: String = text
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .to_lowercase();
    if !typing(ui, state) {
        state.query.clear();
    }
    // Whitespace continues an active prefix ("new y"); alone it starts nothing.
    if text.is_empty() || (state.query.is_empty() && text.trim().is_empty()) {
        return;
    }
    let now = ui.input(|i| i.time);
    let cycling = text.chars().count() == 1 && state.query == text;
    if !cycling {
        state.query.push_str(&text);
        state.query = state
            .query
            .chars()
            .take(geometry::TYPE_AHEAD_LIMIT)
            .collect();
    }
    state.typed_at = now;
    let current = model.index(state.focused.as_ref()).unwrap_or(0);
    let start = if cycling || state.query == text {
        current + 1
    } else {
        current
    };
    let next = (0..model.items.len())
        .map(|step| (start + step) % model.items.len())
        .find(|&i| {
            model.is_enabled(i)
                && model.items[i]
                    .label
                    .trim_start()
                    .to_lowercase()
                    .starts_with(&state.query)
        });
    if let Some(next) = next {
        if !read_only {
            out.changed |= selection::select(state, model, mode, next, Modifiers::NONE, false);
        }
        state.focused = Some(model.items[next].key.clone());
        out.reveal = true;
    }
}

/// A typed prefix is pending: it is non-empty and has not idled out.
fn typing<K>(ui: &Ui, state: &ListState<K>) -> bool {
    !state.query.is_empty() && ui.input(|i| i.time) - state.typed_at <= geometry::TYPE_AHEAD_TIMEOUT
}

fn assistive<K: Clone + Eq + Hash>(
    ui: &Ui,
    model: &Model<'_, K>,
    state: &mut ListState<K>,
    mode: ListSelectionMode,
    read_only: bool,
    out: &mut Outcome<K>,
) {
    let events = ui.input_mut(|i| {
        let mut events = Vec::new();
        i.events.retain(|e| {
            let Event::AccessKitActionRequest(request) = e else {
                return true;
            };
            let Some(index) = model
                .ids
                .iter()
                .position(|id| id.accesskit_id() == request.target_node)
            else {
                return true;
            };
            if !matches!(
                request.action,
                egui::accesskit::Action::Focus | egui::accesskit::Action::Click
            ) {
                return true;
            }
            events.push((index, request.action));
            false
        });
        events
    });
    for (index, action) in events {
        if !model.is_enabled(index) {
            continue;
        }
        if action == egui::accesskit::Action::Click && !read_only {
            out.changed |= selection::select(state, model, mode, index, Modifiers::NONE, false);
        }
        state.focused = Some(model.items[index].key.clone());
        ui.memory_mut(|m| m.request_focus(model.ids[index]));
        out.reveal = true;
    }
}
