use super::{TreeState, model::Model};
use crate::widgets::list::model::row_id;
use egui::{Event, Id, Key, Rect, Ui, accesskit::Action};
use std::hash::Hash;

pub(super) struct Outcome {
    pub expansion_changed: bool,
    pub destination: Option<usize>,
}
pub(super) fn read<K: Clone + Eq + Hash>(
    ui: &Ui,
    root: Id,
    model: &Model<'_, K>,
    state: &mut TreeState<K>,
    read_only: bool,
) -> Outcome {
    let mut out = Outcome {
        expansion_changed: false,
        destination: None,
    };
    if !ui.is_enabled() || !ui.is_visible() {
        return out;
    }
    let focused = ui.memory(|m| m.focused());
    let current = model.index(state.rows.focused.as_ref());
    let owns_focus =
        current.is_some_and(|i| focused == Some(row_id(root, &model.entries[i].node.key)));
    let events = ui.input_mut(|input| {
        let mut events = Vec::new();
        input.events.retain(|event| {
            let take = match event {
                Event::Key {
                    key: Key::ArrowLeft | Key::ArrowRight,
                    pressed: true,
                    modifiers,
                    ..
                } => owns_focus && modifiers.is_none(),
                Event::AccessKitActionRequest(r) => {
                    matches!(r.action, Action::Expand | Action::Collapse)
                        && model
                            .entries
                            .iter()
                            .any(|e| row_id(root, &e.node.key).accesskit_id() == r.target_node)
                }
                _ => false,
            };
            if take {
                events.push(event.clone());
            }
            !take
        });
        events
    });
    for event in events {
        match event {
            Event::Key { key, .. } => {
                state.rows.clear_typeahead();
                if let Some(i) = model.index(state.rows.focused.as_ref()) {
                    arrow(model, state, i, key, read_only, &mut out);
                    if let Some(next) = out.destination {
                        state.rows.focused = Some(model.entries[next].node.key.clone());
                    }
                }
            }
            Event::AccessKitActionRequest(r) => {
                if let Some(i) = model
                    .entries
                    .iter()
                    .position(|e| row_id(root, &e.node.key).accesskit_id() == r.target_node)
                    && !read_only
                    && model.entries[i].enabled
                {
                    set(model, state, i, r.action == Action::Expand, &mut out);
                }
            }
            _ => {}
        }
    }
    if !read_only {
        pointer(ui, root, model, state, &mut out);
    }
    if let Some(i) = out.destination {
        state.rows.focused = Some(model.entries[i].node.key.clone());
        ui.memory_mut(|m| {
            m.move_focus(egui::FocusDirection::None);
            m.request_focus(row_id(root, &model.entries[i].node.key));
        });
    }
    if out.expansion_changed {
        ui.ctx().request_repaint();
    }
    out
}
fn arrow<K: Clone + Eq + Hash>(
    model: &Model<'_, K>,
    state: &mut TreeState<K>,
    index: usize,
    key: Key,
    read_only: bool,
    out: &mut Outcome,
) {
    let entry = &model.entries[index];
    let open = state.expanded.contains(&entry.node.key);
    match key {
        Key::ArrowRight if !entry.children.is_empty() && !open => {
            if !read_only {
                set(model, state, index, true, out);
            }
        }
        Key::ArrowRight if open => {
            out.destination = entry
                .children
                .iter()
                .copied()
                .find(|&i| model.entries[i].enabled)
        }
        Key::ArrowLeft if open => {
            if !read_only {
                set(model, state, index, false, out);
            }
        }
        Key::ArrowLeft => out.destination = entry.parent,
        _ => {}
    }
}
fn set<K: Clone + Eq + Hash>(
    model: &Model<'_, K>,
    state: &mut TreeState<K>,
    index: usize,
    expanded: bool,
    out: &mut Outcome,
) {
    let entry = &model.entries[index];
    if entry.children.is_empty() {
        return;
    }
    out.expansion_changed |= if expanded {
        state.expanded.insert(entry.node.key.clone())
    } else {
        state.expanded.remove(&entry.node.key)
    };
}
fn pointer<K: Clone + Eq + Hash>(
    ui: &Ui,
    root: Id,
    model: &Model<'_, K>,
    state: &mut TreeState<K>,
    out: &mut Outcome,
) {
    let frame = ui.ctx().cumulative_frame_nr();
    if ui
        .ctx()
        .data(|d| d.get_temp::<u64>(root.with("tree-pointer-frame")))
        == Some(frame)
    {
        return;
    }
    let hits = ui
        .ctx()
        .data(|d| d.get_temp::<Vec<(Id, Rect)>>(root.with("tree-disclosures")))
        .unwrap_or_default();
    for (id, rect) in hits {
        let Some(response) = ui.ctx().read_response(id) else {
            continue;
        };
        if !response.clicked()
            || !response.enabled()
            || !response
                .interact_pointer_pos()
                .is_some_and(|p| rect.contains(p))
        {
            continue;
        }
        if let Some(index) = model
            .entries
            .iter()
            .position(|e| e.enabled && row_id(root, &e.node.key) == id)
        {
            ui.ctx()
                .data_mut(|d| d.insert_temp(root.with("tree-pointer-frame"), frame));
            state.rows.clear_typeahead();
            set(
                model,
                state,
                index,
                !state.expanded.contains(&model.entries[index].node.key),
                out,
            );
        }
    }
}
