pub(super) mod accessibility;
pub(super) mod hierarchy;
mod keys;
pub(super) mod model;
pub(super) mod paint;
mod render;
pub(super) mod selection;
mod types;

use crate::{Text, TextSize, tokens::list as geometry};
use egui::{IdSalt, Ui};
use std::{borrow::Cow, hash::Hash};
pub use types::{ListItem, ListResponse, ListRow, ListRowResponse, ListSelectionMode, ListState};

/// Keyed list with fixed-height virtualization and one roving Tab stop.
/// The host owns the current view, selected keys and focused key. Row labels
/// power accessibility and Unicode lowercase prefix search, even offscreen.
#[must_use = "Call show or show_with to render the list"]
pub struct List<'a, K> {
    id: IdSalt,
    state: &'a mut ListState<K>,
    items: &'a [ListItem<'a, K>],
    name: Cow<'a, str>,
    empty: Cow<'a, str>,
    mode: ListSelectionMode,
    row_height: f32,
    height: f32,
    enabled: bool,
    read_only: bool,
    hierarchy: Option<&'a [hierarchy::HierarchyRow]>,
}
impl<'a, K: Clone + Eq + Hash> List<'a, K> {
    /// Bind a stable list ID, application-owned state and current-view rows.
    /// Duplicate keys are invalid: only their first occurrence is interactive.
    pub fn new(
        id: impl egui::AsIdSalt,
        state: &'a mut ListState<K>,
        items: &'a [ListItem<'a, K>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            state,
            items,
            name: Cow::Borrowed("List"),
            empty: Cow::Borrowed("No items"),
            mode: ListSelectionMode::Single,
            row_height: geometry::ROW_HEIGHT,
            height: geometry::HEIGHT,
            enabled: true,
            read_only: false,
            hierarchy: None,
        }
    }
    pub(in crate::widgets) fn hierarchy(mut self, rows: &'a [hierarchy::HierarchyRow]) -> Self {
        self.hierarchy = Some(rows);
        self
    }
    /// Accessible collection name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Muted text shown in place of rows when the current view is empty.
    /// Defaults to "No items"; set it to localize or explain an empty filter.
    pub fn empty_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.empty = text.into();
        self
    }
    /// None, single (default), or multiple selection.
    pub fn selection_mode(mut self, mode: ListSelectionMode) -> Self {
        self.mode = mode;
        self
    }
    /// Fixed row height, clamped to 32–256 points. Nonfinite uses 48.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = finite(
            height,
            geometry::ROW_HEIGHT,
            geometry::MIN_ROW_HEIGHT,
            geometry::MAX_ROW_HEIGHT,
        );
        self
    }
    /// Viewport height cap, clamped to 96–4096 points. Nonfinite uses 384.
    pub fn max_height(mut self, height: f32) -> Self {
        self.height = finite(
            height,
            geometry::HEIGHT,
            geometry::MIN_HEIGHT,
            geometry::MAX_HEIGHT,
        );
        self
    }
    /// Disable row interaction and scrolling, respecting disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Keep navigation and scrolling but suppress selection edits and activation.
    /// Stale keys are still normalized against the current view.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Render vertically centered single-line labels, elided with "…" when too long.
    pub fn show(self, ui: &mut Ui) -> ListResponse<K> {
        self.show_with(ui, |ui, item, _| {
            let theme = crate::Theme::from_context(ui.ctx());
            let mut job = Text::new(item.label.replace(['\n', '\r'], " "))
                .size(TextSize::Sm)
                .layout_job(&theme, f32::INFINITY);
            // Long labels end in an ellipsis inside the row padding, like Button.
            job.wrap = egui::text::TextWrapping::truncate_at_width(ui.available_width());
            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), galley.size().y),
                egui::Sense::hover(),
            );
            ui.painter()
                .galley(rect.min, galley, theme.palette.foreground);
        })
    }
    /// Render passive custom content only for visible rows, once per egui pass.
    /// Content is clipped to the fixed row and cannot change its height. Use
    /// text, icons and badges; embedded buttons/editors require a different
    /// composite (for example DataTable), not list box option semantics.
    pub fn show_with(
        self,
        ui: &mut Ui,
        content: impl FnMut(&mut Ui, &ListItem<'a, K>, ListRow),
    ) -> ListResponse<K> {
        render::show(self, ui, content)
    }
}
fn finite(value: f32, default: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        default
    }
}
