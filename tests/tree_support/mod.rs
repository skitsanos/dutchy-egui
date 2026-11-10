use dutchy_egui::{ListSelectionMode, Theme, TreeNode, TreeState, TreeView};
use egui::{Key, Modifiers, Rect, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};
pub struct Page {
    pub tree: TreeState<u32>,
    pub mode: ListSelectionMode,
    pub enabled: bool,
    pub read_only: bool,
    pub removed: bool,
    pub empty: bool,
    pub custom: bool,
    pub discard: bool,
    pub disabled_branch: bool,
    pub changes: usize,
    pub expansions: usize,
    pub activated: Vec<u32>,
    pub range: std::ops::Range<usize>,
    pub viewport: Rect,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            tree: TreeState::default(),
            mode: ListSelectionMode::Single,
            enabled: true,
            read_only: false,
            removed: false,
            empty: false,
            custom: false,
            discard: false,
            disabled_branch: false,
            changes: 0,
            expansions: 0,
            activated: Vec::new(),
            range: 0..0,
            viewport: Rect::NOTHING,
        }
    }
}
pub fn show(ui: &mut egui::Ui, p: &mut Page) {
    let _ = ui.button("Before tree");
    let files = [TreeNode::new(3, "Brief"), TreeNode::new(4, "Research")];
    let folders = [
        TreeNode::new(2, "Projects").children(&files),
        TreeNode::new(5, "Archive"),
    ];
    let roots = [
        TreeNode::new(1, "Workspace")
            .children(&folders)
            .enabled(!p.disabled_branch),
        TreeNode::new(6, "Trash"),
    ];
    let nodes = if p.empty {
        &roots[..0]
    } else if p.removed {
        &roots[1..]
    } else {
        &roots[..]
    };
    let tree = TreeView::new("workspace", &mut p.tree, nodes)
        .accessible_name("Workspace tree")
        .selection_mode(p.mode)
        .enabled(p.enabled)
        .read_only(p.read_only)
        .max_height(192.0);
    let out = if p.custom {
        tree.show_with(ui, |ui, node, _| {
            ui.label(node.label.as_ref());
        })
    } else {
        tree.show(ui)
    };
    if p.discard && ui.ctx().current_pass_index() == 0 {
        ui.ctx().request_discard("tree interaction regression");
    }
    p.changes += usize::from(out.list.response.changed());
    p.expansions += usize::from(out.expansion_changed);
    p.activated.extend(out.list.activated);
    p.range = out.list.visible_range;
    p.viewport = out.list.viewport;
    let _ = ui.button("After tree");
}
pub fn harness(p: Page) -> Harness<'static, Page> {
    let mut h = Harness::builder()
        .with_size([440.0, 340.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(show, p);
    h.run_steps(3);
    h
}
pub fn focus(h: &mut Harness<'_, Page>, name: &str) {
    h.get_by_role_and_label(Role::TreeItem, name).focus();
    h.run_steps(3);
}
pub fn key(h: &mut Harness<'_, Page>, key: Key) {
    h.key_press(key);
    h.run_steps(3);
}
pub fn modified(h: &mut Harness<'_, Page>, key: Key, modifiers: Modifiers) {
    h.key_press_modifiers(modifiers, key);
    h.run_steps(3);
}
pub fn selected(h: &Harness<'_, Page>) -> Vec<u32> {
    let mut v: Vec<_> = h.state().tree.rows.selected.iter().copied().collect();
    v.sort();
    v
}

pub fn ancestor<'a>(
    mut node: egui_kittest::kittest::AccessKitNode<'a>,
    role: Role,
) -> egui_kittest::kittest::AccessKitNode<'a> {
    loop {
        node = node.parent().expect("ancestor exists");
        if node.role() == role {
            return node;
        }
    }
}
