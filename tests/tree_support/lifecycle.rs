use super::support::*;
use dutchy_egui::{ListSelectionMode, Theme, TreeNode, TreeState, TreeView};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn removing_all_nodes_leaves_a_focusable_empty_tree() {
    let mut h = harness(Page::default());
    focus(&mut h, "Workspace");
    h.state_mut().empty = true;
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Tree, "Workspace tree")
            .is_focused()
    );
    key(&mut h, Key::Tab);
    assert!(h.get_by_label("After tree").is_focused());
    h.check_accessibility();
}

#[test]
fn duplicate_subtrees_are_ignored_and_stale_expansion_is_pruned() {
    let mut state = TreeState::default();
    state.expanded.insert(1);
    state.rows.selected.insert(9);
    let mut h = Harness::builder()
        .with_size([400.0, 240.0])
        .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut TreeState<u32>| {
                let hidden = [TreeNode::new(9, "Ignored child")];
                let nodes = [
                    TreeNode::new(1, "First"),
                    TreeNode::new(1, "Duplicate").children(&hidden),
                ];
                TreeView::new("duplicates", state, &nodes).show(ui);
            },
            state,
        );
    h.run_steps(3);
    assert!(h.state().expanded.is_empty());
    assert!(h.state().rows.selected.is_empty());
    assert!(
        h.query_by_role_and_label(Role::TreeItem, "Duplicate")
            .is_none()
    );
    h.check_accessibility();
}

#[test]
fn collapsing_a_focused_descendant_repairs_focus_and_retains_selection_and_nested_expansion() {
    let mut p = Page::default();
    p.tree.expanded.extend([1, 2]);
    p.tree.rows.selected.insert(3);
    let mut h = harness(p);
    focus(&mut h, "Brief");
    let id = h
        .get_by_role_and_label(Role::TreeItem, "Brief")
        .accesskit_node()
        .id();
    h.state_mut().tree.expanded.remove(&1);
    h.run_steps(4);
    assert_eq!(h.state().tree.rows.focused, Some(1));
    assert_eq!(selected(&h), [3]);
    assert!(
        h.get_by_role_and_label(Role::TreeItem, "Workspace")
            .is_focused()
    );
    assert!(h.state().tree.expanded.contains(&2));
    h.state_mut().tree.expanded.insert(1);
    h.run_steps(4);
    assert_eq!(
        h.get_by_role_and_label(Role::TreeItem, "Brief")
            .accesskit_node()
            .id(),
        id
    );
    h.check_accessibility();
}
#[test]
fn removing_a_subtree_prunes_keys_and_disabled_branches_disable_descendants() {
    let mut p = Page::default();
    p.tree.expanded.extend([1, 2]);
    p.tree.rows.selected.insert(3);
    let mut h = harness(p);
    focus(&mut h, "Brief");
    h.state_mut().disabled_branch = true;
    h.run_steps(3);
    assert!(selected(&h).is_empty());
    assert_eq!(h.state().tree.rows.focused, Some(6));
    h.state_mut().removed = true;
    h.run_steps(3);
    assert!(h.state().tree.expanded.is_empty());
    h.check_accessibility();
}
#[test]
fn read_only_allows_navigation_inside_open_branches_and_disabled_blocks_edits() {
    let mut p = Page::default();
    p.tree.expanded.insert(1);
    p.read_only = true;
    let mut h = harness(p);
    focus(&mut h, "Workspace");
    key(&mut h, Key::ArrowLeft);
    assert!(h.state().tree.expanded.contains(&1));
    key(&mut h, Key::ArrowRight);
    assert_eq!(h.state().tree.rows.focused, Some(2));
    key(&mut h, Key::ArrowRight);
    assert!(!h.state().tree.expanded.contains(&2));
    assert!(selected(&h).is_empty());
    key(&mut h, Key::Enter);
    assert!(h.state().activated.is_empty());
    h.state_mut().enabled = false;
    h.run_steps(3);
    key(&mut h, Key::ArrowLeft);
    assert_eq!(h.state().tree.rows.focused, Some(2));
    h.check_accessibility();
}
#[test]
fn ten_thousand_children_virtualize_and_materialize_offscreen_ancestors() {
    let children: Vec<_> = (1..=10000u32)
        .map(|i| TreeNode::new(i, format!("Document {i:05}")))
        .collect();
    let mut state = TreeState::default();
    state.expanded.insert(0);
    let mut h = Harness::builder()
        .with_size([440.0, 300.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            move |ui, state: &mut TreeState<u32>| {
                let roots = [TreeNode::new(0, "Documents").children(&children)];
                let r = TreeView::new("large", state, &roots)
                    .max_height(192.0)
                    .show(ui);
                assert!(r.list.rows.len() < 10);
            },
            state,
        );
    h.run_steps(3);
    h.get_by_role_and_label(Role::TreeItem, "Documents").focus();
    h.run_steps(3);
    h.key_press(Key::End);
    h.run_steps(4);
    assert_eq!(h.state().rows.focused, Some(10000));
    let last = h.get_by_role_and_label(Role::TreeItem, "Document 10000");
    assert!(last.is_focused());
    assert_eq!(
        ancestor(last.accesskit_node(), Role::Group).size_of_set(),
        Some(10000)
    );
    assert_eq!(
        ancestor(ancestor(last.accesskit_node(), Role::Group), Role::TreeItem)
            .label()
            .as_deref(),
        Some("Documents")
    );
    h.check_accessibility();
}

#[test]
fn manual_scrolling_keeps_focus_and_offscreen_selection_metadata_updates_in_the_click_frame() {
    let children: Vec<_> = (1..=10000u32)
        .map(|i| TreeNode::new(i, format!("Document {i:05}")))
        .collect();
    let mut state = TreeState::default();
    state.expanded.insert(0);
    state.rows.selected.insert(0);
    let mut h = Harness::builder()
        .with_size([440.0, 300.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            move |ui, state: &mut TreeState<u32>| {
                let roots = [TreeNode::new(0, "Documents").children(&children)];
                TreeView::new("scroll", state, &roots)
                    .max_height(192.0)
                    .show(ui);
            },
            state,
        );
    h.run_steps(3);
    h.get_by_role_and_label(Role::TreeItem, "Document 00001")
        .focus();
    h.run_steps(3);
    let at = h.get_by_role_and_label(Role::Tree, "Tree").rect().center();
    h.hover_at(at);
    h.run_steps(2);
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: egui::vec2(0.0, -2000.0),
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(80);
    assert!(
        h.get_by_role_and_label(Role::TreeItem, "Document 00001")
            .is_focused()
    );
    let target = h.get_by_role_and_label(Role::TreeItem, "Document 00043");
    target.click();
    h.run_steps(1);
    assert_eq!(
        h.state().rows.selected.iter().copied().collect::<Vec<_>>(),
        [43]
    );
    assert_eq!(
        h.get_by_role_and_label(Role::TreeItem, "Documents")
            .accesskit_node()
            .is_selected(),
        Some(false)
    );
    h.check_accessibility();
}
#[test]
fn none_mode_keeps_tree_semantics_and_explicit_activation() {
    let p = Page {
        mode: ListSelectionMode::None,
        ..Default::default()
    };
    let mut h = harness(p);
    focus(&mut h, "Workspace");
    key(&mut h, Key::ArrowRight);
    key(&mut h, Key::ArrowRight);
    key(&mut h, Key::Space);
    assert!(selected(&h).is_empty());
    key(&mut h, Key::Enter);
    assert_eq!(h.state().activated, [2]);
    h.check_accessibility();
}
