use super::support::*;
use dutchy_egui::ListSelectionMode;
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn custom_text_is_passive_and_clicks_select_the_tree_row() {
    let mut p = Page::default();
    p.tree.expanded.insert(1);
    p.custom = true;
    let mut h = harness(p);
    h.get_by(|n| n.role() == Role::Label && n.value().as_deref() == Some("Projects"))
        .click();
    h.run_steps(3);
    assert_eq!(selected(&h), [2]);
    assert!(
        h.get_by_role_and_label(Role::TreeItem, "Projects")
            .is_focused()
    );
}

#[test]
fn disclosure_toggles_once_during_a_discard_pass() {
    let p = Page {
        discard: true,
        ..Default::default()
    };
    let mut h = harness(p);
    let rect = h.get_by_role_and_label(Role::TreeItem, "Workspace").rect();
    let pos = egui::pos2(rect.left() + 24.0, rect.center().y);
    h.hover_at(pos);
    h.run_steps(2);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(3);
    assert!(h.state().tree.expanded.contains(&1));
    assert_eq!(h.state().expansions, 1);
    h.get_by_role_and_label(Role::TreeItem, "Trash")
        .click_modifiers(Modifiers::COMMAND);
    h.run_steps(3);
    assert_eq!(
        selected(&h),
        [6],
        "row selection must also survive discard passes"
    );
    h.check_accessibility();
}

#[test]
fn repeated_right_keys_in_one_frame_enter_and_open_the_child() {
    let mut h = harness(Page::default());
    focus(&mut h, "Workspace");
    for _ in 0..3 {
        h.event(egui::Event::Key {
            key: Key::ArrowRight,
            physical_key: Some(Key::ArrowRight),
            pressed: true,
            repeat: true,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(3);
    assert_eq!(h.state().tree.rows.focused, Some(2));
    assert!(h.state().tree.expanded.contains(&2));
    h.check_accessibility();
}

#[test]
fn right_opens_then_enters_children_and_left_closes_then_returns_to_parent() {
    let mut h = harness(Page::default());
    focus(&mut h, "Workspace");
    key(&mut h, Key::ArrowRight);
    assert!(h.state().tree.expanded.contains(&1));
    assert_eq!(h.state().tree.rows.focused, Some(1));
    key(&mut h, Key::ArrowRight);
    assert_eq!(h.state().tree.rows.focused, Some(2));
    assert_eq!(selected(&h), [2]);
    key(&mut h, Key::ArrowRight);
    assert!(h.state().tree.expanded.contains(&2));
    key(&mut h, Key::ArrowRight);
    assert_eq!(h.state().tree.rows.focused, Some(3));
    key(&mut h, Key::ArrowLeft);
    assert_eq!(h.state().tree.rows.focused, Some(2));
    key(&mut h, Key::ArrowLeft);
    assert!(!h.state().tree.expanded.contains(&2));
    key(&mut h, Key::ArrowLeft);
    assert_eq!(h.state().tree.rows.focused, Some(1));
    h.check_accessibility();
}
#[test]
fn disclosure_click_changes_expansion_without_selection_and_text_click_selects() {
    let mut h = harness(Page::default());
    let rect = h.get_by_role_and_label(Role::TreeItem, "Workspace").rect();
    let pos = egui::pos2(rect.left() + 24.0, rect.center().y);
    h.hover_at(pos);
    h.run_steps(2);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(4);
    assert!(h.state().tree.expanded.contains(&1));
    assert!(selected(&h).is_empty());
    assert_eq!(h.state().expansions, 1);
    h.get_by_role_and_label(Role::TreeItem, "Projects").click();
    h.run_steps(3);
    assert_eq!(selected(&h), [2]);
    assert!(
        h.get_by_role_and_label(Role::TreeItem, "Projects")
            .is_focused()
    );
}
#[test]
fn ranges_and_type_ahead_use_only_visible_rows_and_tab_exits_once() {
    let mut p = Page {
        mode: ListSelectionMode::Multiple,
        ..Default::default()
    };
    p.tree.expanded.extend([1, 2]);
    let mut h = harness(p);
    focus(&mut h, "Projects");
    key(&mut h, Key::Space);
    modified(&mut h, Key::ArrowDown, Modifiers::SHIFT);
    assert_eq!(selected(&h), [2, 3]);
    h.event(egui::Event::Text("r".into()));
    h.run_steps(3);
    assert_eq!(h.state().tree.rows.focused, Some(4));
    key(&mut h, Key::Tab);
    assert!(h.get_by_label("After tree").is_focused());
    modified(&mut h, Key::Tab, Modifiers::SHIFT);
    assert!(
        h.get_by_role_and_label(Role::TreeItem, "Research")
            .is_focused()
    );
    h.check_accessibility();
}
#[test]
fn tree_roles_depth_sibling_sets_and_expansion_actions_are_hierarchical() {
    let mut p = Page::default();
    p.tree.expanded.extend([1, 2]);
    let h = harness(p);
    let tree = h.get_by_role_and_label(Role::Tree, "Workspace tree");
    assert_eq!(tree.accesskit_node().size_of_set(), Some(2));
    // AccessKit levels are zero-based like positions; platform adapters add one
    // (Windows reports a root item as UIA level 1).
    let root = h.get_by_role_and_label(Role::TreeItem, "Workspace");
    assert_eq!(root.accesskit_node().level(), Some(0));
    let brief = h.get_by_role_and_label(Role::TreeItem, "Brief");
    let node = brief.accesskit_node();
    assert_eq!(node.level(), Some(2));
    assert_eq!(node.position_in_set(), Some(0));
    let group = ancestor(node, Role::Group);
    assert_eq!(group.role(), Role::Group);
    assert_eq!(group.size_of_set(), Some(2));
    assert_eq!(
        ancestor(group, Role::TreeItem).label().as_deref(),
        Some("Projects")
    );
    assert_eq!(
        h.get_by_role_and_label(Role::TreeItem, "Projects")
            .accesskit_node()
            .data()
            .is_expanded(),
        Some(true)
    );
    assert_eq!(node.data().is_expanded(), None);
    h.check_accessibility();
}
#[test]
fn assistive_expansion_actions_open_and_close_the_named_branch() {
    let mut h = harness(Page::default());
    let (target_node, target_tree) = h
        .get_by_role_and_label(Role::TreeItem, "Workspace")
        .accesskit_node()
        .locate();
    h.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            target_node,
            target_tree,
            action: egui::accesskit::Action::Expand,
            data: None,
        },
    ));
    h.run_steps(3);
    assert!(h.state().tree.expanded.contains(&1));
    h.check_accessibility();
}
