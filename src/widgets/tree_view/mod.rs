mod input;
mod model;
mod types;

use super::list::{
    List, ListItem, ListSelectionMode,
    hierarchy::{self, HierarchyRow},
};
use crate::{Text, TextSize, Theme, tokens::list as geometry};
use egui::{IdSalt, Ui};
use std::{borrow::Cow, hash::Hash};
pub use types::{TreeNode, TreeRow, TreeState, TreeViewResponse};

/// Keyed, virtualized tree over List's row/selection engine.
/// Expansion and row state live in the host; collapsing never erases selection.
#[must_use = "Call show or show_with to render the tree"]
pub struct TreeView<'a, K> {
    id: IdSalt,
    state: &'a mut TreeState<K>,
    nodes: &'a [TreeNode<'a, K>],
    name: Cow<'a, str>,
    empty: Cow<'a, str>,
    mode: ListSelectionMode,
    height: f32,
    row_height: f32,
    enabled: bool,
    read_only: bool,
}
impl<'a, K: Clone + Eq + Hash> TreeView<'a, K> {
    /// Bind a stable tree ID, host state and root nodes in display order.
    /// Keys must be unique across all nodes; later duplicate subtrees are ignored.
    pub fn new(
        id: impl egui::AsIdSalt,
        state: &'a mut TreeState<K>,
        nodes: &'a [TreeNode<'a, K>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            state,
            nodes,
            name: Cow::Borrowed("Tree"),
            empty: Cow::Borrowed("No items"),
            mode: ListSelectionMode::Single,
            height: geometry::HEIGHT,
            row_height: geometry::ROW_HEIGHT,
            enabled: true,
            read_only: false,
        }
    }
    /// Accessible name of the tree.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Short muted empty-state text.
    pub fn empty_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.empty = text.into();
        self
    }
    /// Reuse List's none, single (default), or multiple selection model.
    pub fn selection_mode(mut self, mode: ListSelectionMode) -> Self {
        self.mode = mode;
        self
    }
    /// Fixed row height; uses the same finite/clamped limits as List.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }
    /// Viewport height cap; uses the same finite/clamped limits as List.
    pub fn max_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    /// Disable interaction and scrolling, respecting disabled parent UIs.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Keep navigation through open branches but prevent user edits/activation.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Render single-line labels with ellipsis after the disclosure gutter.
    pub fn show(self, ui: &mut Ui) -> TreeViewResponse<K> {
        self.show_with(ui, |ui, node, _| {
            let theme = Theme::from_context(ui.ctx());
            let mut job = Text::new(node.label.replace(['\n', '\r'], " "))
                .size(TextSize::Sm)
                .layout_job(&theme, f32::INFINITY);
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
    /// Compose passive text, icons and badges after the indented disclosure slot.
    /// As in List, embedded interactive controls and variable heights are unsupported.
    pub fn show_with(
        self,
        ui: &mut Ui,
        mut content: impl FnMut(&mut Ui, &TreeNode<'a, K>, TreeRow),
    ) -> TreeViewResponse<K> {
        let model = model::Model::new(self.nodes);
        let (normalized, pruned) = model.normalize(self.state, self.mode);
        let salt = IdSalt::new(("dutchy-tree", self.id));
        let root = hierarchy::root(ui, salt);
        let input = if self.enabled {
            input::read(ui, root, &model, self.state, self.read_only)
        } else {
            input::Outcome {
                expansion_changed: false,
                destination: None,
            }
        };
        let mut selection_changed = normalized;
        if !self.read_only
            && let Some(i) = input.destination
        {
            selection_changed |= super::list::selection::select_focused(
                &mut self.state.rows,
                self.mode,
                &model.entries[i].node.key,
            );
        }
        let visible = model.visible(self.state);
        model.repair_focus(self.state, &visible);
        let items: Vec<_> = visible
            .iter()
            .map(|&i| {
                let e = &model.entries[i];
                ListItem::new(e.node.key.clone(), e.node.label.as_ref()).enabled(e.enabled)
            })
            .collect();
        let metas: Vec<_> = visible
            .iter()
            .map(|&i| {
                let e = &model.entries[i];
                HierarchyRow {
                    depth: e.depth,
                    parent: e
                        .parent
                        .map(|p| visible.binary_search(&p).expect("visible ancestors")),
                    children: e.children.len(),
                    position: e.position,
                    expanded: (!e.children.is_empty())
                        .then(|| self.state.expanded.contains(&e.node.key)),
                }
            })
            .collect();
        let mut list = List::new(("dutchy-tree", self.id), &mut self.state.rows, &items)
            .accessible_name(self.name)
            .empty_text(self.empty)
            .selection_mode(self.mode)
            .enabled(self.enabled)
            .read_only(self.read_only)
            .row_height(self.row_height)
            .max_height(self.height)
            .hierarchy(&metas)
            .show_with(ui, |ui, _, row| {
                let entry = &model.entries[visible[row.index]];
                let meta = metas[row.index];
                content(
                    ui,
                    entry.node,
                    TreeRow {
                        index: row.index,
                        depth: entry.depth,
                        selected: row.selected,
                        focused: row.focused,
                        branch: meta.expanded.is_some(),
                        expanded: meta.expanded.unwrap_or(false),
                    },
                );
            });
        if selection_changed {
            list.response.mark_changed();
        }
        ui.ctx().accesskit_node_builder(list.response.id, |node| {
            node.set_role(egui::accesskit::Role::Tree);
            node.set_size_of_set(model.roots.len());
        });
        TreeViewResponse {
            list,
            expansion_changed: pruned || input.expansion_changed,
        }
    }
}
