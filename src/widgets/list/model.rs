use super::{ListItem, ListSelectionMode, ListState};
use egui::Id;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

pub(super) struct Model<'a, K> {
    pub items: &'a [ListItem<'a, K>],
    pub ids: Vec<Id>,
    pub enabled: Vec<usize>,
    indices: HashMap<&'a K, usize>,
}
impl<'a, K: Eq + Hash> Model<'a, K> {
    pub fn new(root: Id, items: &'a [ListItem<'a, K>]) -> Self {
        let mut indices = HashMap::with_capacity(items.len());
        let mut enabled = Vec::new();
        let ids = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let id = row_id(root, &item.key);
                if indices.contains_key(&item.key) {
                    return id.with(("duplicate", index));
                }
                indices.insert(&item.key, index);
                if item.enabled {
                    enabled.push(index);
                }
                id
            })
            .collect();
        Self {
            items,
            ids,
            enabled,
            indices,
        }
    }
    pub fn index(&self, key: Option<&K>) -> Option<usize> {
        key.and_then(|key| self.indices.get(key).copied())
            .filter(|index| self.is_enabled(*index))
    }
    pub fn is_enabled(&self, index: usize) -> bool {
        self.enabled.binary_search(&index).is_ok()
    }
}
impl<K: Clone + Eq + Hash> Model<'_, K> {
    /// Prune stale keys and repair focus; report whether the selection changed.
    pub fn normalize(&self, state: &mut ListState<K>, mode: ListSelectionMode) -> bool {
        let changed = normalize_selection(
            state,
            mode,
            |key| self.index(Some(key)).is_some(),
            self.enabled.iter().map(|&i| &self.items[i].key),
        );
        self.normalize_focus(state);
        changed
    }
    pub(in crate::widgets) fn normalize_focus(&self, state: &mut ListState<K>) {
        if self.index(state.focused.as_ref()).is_none() {
            state.focused = self
                .enabled
                .iter()
                .find(|&&i| state.selected.contains(&self.items[i].key))
                .or_else(|| self.enabled.first())
                .map(|&i| self.items[i].key.clone());
        }
        if self.index(state.anchor.as_ref()).is_none() {
            state.anchor = None;
        }
    }
}

pub(in crate::widgets) fn row_id<K: Hash>(root: Id, key: &K) -> Id {
    let mut hash = DefaultHasher::new();
    key.hash(&mut hash);
    root.with(("row", hash.finish()))
}
pub(in crate::widgets) fn normalize_selection<'a, K: Clone + Eq + Hash + 'a>(
    state: &mut ListState<K>,
    mode: ListSelectionMode,
    valid: impl Fn(&K) -> bool,
    mut ordered: impl Iterator<Item = &'a K>,
) -> bool {
    let before = state.selected.len();
    state.selected.retain(|key| valid(key));
    let mut changed = state.selected.len() != before;
    match mode {
        ListSelectionMode::None => {
            changed |= !state.selected.is_empty();
            state.selected.clear();
        }
        ListSelectionMode::Single if state.selected.len() > 1 => {
            changed = true;
            let first = ordered.find(|key| state.selected.contains(*key)).cloned();
            state.selected.clear();
            if let Some(key) = first {
                state.selected.insert(key);
            }
        }
        _ => {}
    }
    changed
}
