use super::{TreeNode, TreeState};
use crate::widgets::list::{ListSelectionMode, model::normalize_selection};
use std::{collections::HashMap, hash::Hash};

pub(super) struct Entry<'a, K> {
    pub node: &'a TreeNode<'a, K>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub enabled: bool,
    pub depth: usize,
    pub position: usize,
}
pub(super) struct Model<'a, K> {
    pub entries: Vec<Entry<'a, K>>,
    pub roots: Vec<usize>,
    keys: HashMap<&'a K, usize>,
}
impl<'a, K: Clone + Eq + Hash> Model<'a, K> {
    pub fn new(nodes: &'a [TreeNode<'a, K>]) -> Self {
        let mut result = Self {
            entries: Vec::new(),
            roots: Vec::new(),
            keys: HashMap::new(),
        };
        let mut stack: Vec<_> = nodes.iter().rev().map(|n| (n, None, 0)).collect();
        while let Some((node, parent, depth)) = stack.pop() {
            if result.keys.contains_key(&node.key) {
                continue;
            }
            let index = result.entries.len();
            let siblings = parent.map_or(&mut result.roots, |p: usize| {
                &mut result.entries[p].children
            });
            let position = siblings.len();
            siblings.push(index);
            let enabled = node.enabled && parent.is_none_or(|p| result.entries[p].enabled);
            result.keys.insert(&node.key, index);
            result.entries.push(Entry {
                node,
                parent,
                children: Vec::new(),
                enabled,
                depth,
                position,
            });
            stack.extend(
                node.children
                    .iter()
                    .rev()
                    .map(|child| (child, Some(index), depth + 1)),
            );
        }
        result
    }
    pub fn index(&self, key: Option<&K>) -> Option<usize> {
        key.and_then(|k| self.keys.get(k).copied())
    }
    pub fn visible(&self, state: &TreeState<K>) -> Vec<usize> {
        let mut visible = vec![false; self.entries.len()];
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                visible[i] = e.parent.is_none_or(|p| {
                    visible[p] && state.expanded.contains(&self.entries[p].node.key)
                });
                visible[i].then_some(i)
            })
            .collect()
    }
    pub fn normalize(&self, state: &mut TreeState<K>, mode: ListSelectionMode) -> (bool, bool) {
        let before = state.expanded.len();
        state.expanded.retain(|k| {
            self.index(Some(k))
                .is_some_and(|i| !self.entries[i].children.is_empty())
        });
        let changed = normalize_selection(
            &mut state.rows,
            mode,
            |k| self.index(Some(k)).is_some_and(|i| self.entries[i].enabled),
            self.entries
                .iter()
                .filter(|e| e.enabled)
                .map(|e| &e.node.key),
        );
        let visible = self.visible(state);
        self.repair_focus(state, &visible);
        (changed, before != state.expanded.len())
    }
    pub fn repair_focus(&self, state: &mut TreeState<K>, visible: &[usize]) {
        let candidate = self
            .index(state.rows.focused.as_ref())
            .filter(|&i| self.entries[i].enabled)
            .or_else(|| {
                self.entries
                    .iter()
                    .position(|e| e.enabled && state.rows.selected.contains(&e.node.key))
            })
            .or_else(|| visible.iter().copied().find(|&i| self.entries[i].enabled));
        let mut candidate = candidate;
        while let Some(index) = candidate {
            if visible.binary_search(&index).is_ok() {
                break;
            }
            candidate = self.entries[index].parent;
        }
        state.rows.focused = candidate.map(|i| self.entries[i].node.key.clone());
    }
}
