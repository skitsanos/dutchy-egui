use egui::{Rect, Response};
use std::{borrow::Cow, collections::HashSet, ops::Range};

/// How a list updates its host-owned selected keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListSelectionMode {
    /// Roving focus and activation without selection; List/ListItem semantics.
    None,
    /// At most one selected key. Selection may initially be empty.
    #[default]
    Single,
    /// Modifier-based range and discontiguous selection.
    Multiple,
}

/// A stable row key and its accessible, type-ahead label.
#[derive(Clone, Debug)]
pub struct ListItem<'a, K> {
    /// Host identity. Keys must be unique within the current view.
    pub key: K,
    /// Accessible name and default visible row content.
    pub label: Cow<'a, str>,
    pub(super) enabled: bool,
}
impl<'a, K> ListItem<'a, K> {
    /// Define one keyed row. Labels need not be unique.
    pub fn new(key: K, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            label: label.into(),
            enabled: true,
        }
    }
    /// Skip this row during selection, navigation and activation.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Host-owned selection, roving row, range anchor and transient typed prefix.
/// Removing or disabling a key prunes it when the list is next shown.
#[derive(Clone, Debug)]
pub struct ListState<K> {
    /// Selected keys, independent of their current display order.
    pub selected: HashSet<K>,
    /// Roving row. Setting this reveals it without stealing external focus.
    pub focused: Option<K>,
    pub(super) anchor: Option<K>,
    pub(super) query: String,
    pub(super) typed_at: f64,
}
impl<K> Default for ListState<K> {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            focused: None,
            anchor: None,
            query: String::new(),
            typed_at: f64::NEG_INFINITY,
        }
    }
}

/// Logical row state supplied to passive custom content.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct ListRow {
    /// Zero-based index in the current view.
    pub index: usize,
    /// Whether the host selection contains this key.
    pub selected: bool,
    /// Whether this row owns keyboard focus.
    pub focused: bool,
}

/// One rendered row. Offscreen focus proxies do not appear here.
#[non_exhaustive]
pub struct ListRowResponse<K> {
    /// Stable host key.
    pub key: K,
    /// Zero-based index in the current view.
    pub index: usize,
    /// Row click, hover and focus response.
    pub response: Response,
}

/// Visible row responses and an explicit activation request.
#[non_exhaustive]
pub struct ListResponse<K> {
    /// List surface; changed only when selection changes, including pruning.
    pub response: Response,
    /// Rendered rows in source order, including egui's one-row overscan.
    pub rows: Vec<ListRowResponse<K>>,
    /// Half-open current-view range passed to the row renderer.
    pub visible_range: Range<usize>,
    /// Scroll viewport, excluding the scrollbar.
    pub viewport: Rect,
    /// Enter or double-click request. The host performs the action.
    pub activated: Option<K>,
}

impl<K> ListState<K> {
    pub(in crate::widgets) fn clear_typeahead(&mut self) {
        self.query.clear();
        self.typed_at = f64::NEG_INFINITY;
    }
}
