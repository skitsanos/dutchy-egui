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
                let mut hash = DefaultHasher::new();
                item.key.hash(&mut hash);
                let id = root.with(("row", hash.finish()));
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
        let before = state.selected.len();
        state.selected.retain(|key| self.index(Some(key)).is_some());
        let mut changed = state.selected.len() != before;
        match mode {
            ListSelectionMode::None => {
                changed |= !state.selected.is_empty();
                state.selected.clear();
            }
            ListSelectionMode::Single if state.selected.len() > 1 => {
                changed = true;
                let first = self
                    .enabled
                    .iter()
                    .find(|&&i| state.selected.contains(&self.items[i].key))
                    .copied();
                state.selected.clear();
                if let Some(index) = first {
                    state.selected.insert(self.items[index].key.clone());
                }
            }
            _ => {}
        }
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
        changed
    }
}
