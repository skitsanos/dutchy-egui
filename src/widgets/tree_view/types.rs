use super::super::list::{ListResponse, ListState};
use std::{borrow::Cow, collections::HashSet};

/// Declarative keyed node. Children borrow host-owned node descriptors.
#[derive(Clone, Debug)]
pub struct TreeNode<'a, K> {
    /// Unique, stable identity throughout the tree, including hidden nodes.
    pub key: K,
    /// Accessible and type-ahead label; the default visible text.
    pub label: Cow<'a, str>,
    pub(super) children: &'a [TreeNode<'a, K>],
    pub(super) enabled: bool,
}
impl<'a, K> TreeNode<'a, K> {
    /// Define a leaf. Use children to turn it into a branch.
    pub fn new(key: K, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            label: label.into(),
            children: &[],
            enabled: true,
        }
    }
    /// Borrow this branch's children in display order. An empty slice is a leaf.
    pub fn children(mut self, children: &'a [Self]) -> Self {
        self.children = children;
        self
    }
    /// Disable this node and its complete subtree without erasing expansion.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Application-owned row state and keyed branch expansion.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct TreeState<K> {
    /// List's selection, roving focus, range anchor and typed prefix.
    pub rows: ListState<K>,
    /// Expanded branch keys. Descendant expansion survives ancestor collapse.
    pub expanded: HashSet<K>,
}
impl<K> Default for TreeState<K> {
    fn default() -> Self {
        Self {
            rows: ListState::default(),
            expanded: HashSet::new(),
        }
    }
}

/// Passive content context for one visible tree row.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct TreeRow {
    /// Zero-based index in the flattened visible view (not the complete tree).
    pub index: usize,
    /// Zero-based hierarchy depth.
    pub depth: usize,
    /// Whether the key is selected, independently of focus.
    pub selected: bool,
    /// Whether this row owns keyboard focus.
    pub focused: bool,
    /// Whether the node has children.
    pub branch: bool,
    /// Whether the branch is open; false for leaves.
    pub expanded: bool,
}

/// Selection/focus/activation from List plus independent expansion changes.
#[non_exhaustive]
pub struct TreeViewResponse<K> {
    /// Reused row-engine result. Its response.changed() reports selection edits.
    pub list: ListResponse<K>,
    /// Whether expansion changed, including pruning removed branch keys.
    pub expansion_changed: bool,
}
