use super::support::*;
use dutchy_egui::StatusItem;
use egui::{
    Key,
    accesskit::{Action, ActionRequest, Role},
};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn clickable_segments_activate_by_pointer_keyboard_and_assistive_click() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Sync").click();
    h.run_steps(2);
    assert_eq!(h.state().clicked, [3]);
    h.get_by_role_and_label(Role::Button, "Sync").focus();
    h.run_steps(2);
    key(&mut h, Key::Enter);
    key(&mut h, Key::Space);
    assert_eq!(h.state().clicked, [3, 3, 3]);
    let (target_node, target_tree) = h
        .get_by_role_and_label(Role::Button, "Sync")
        .accesskit_node()
        .locate();
    h.event(egui::Event::AccessKitActionRequest(ActionRequest {
        target_node,
        target_tree,
        action: Action::Click,
        data: None,
    }));
    h.run_steps(2);
    assert_eq!(h.state().clicked, [3, 3, 3, 3]);
    h.check_accessibility();
}
#[test]
fn tab_reaches_only_clickable_segments_and_labels_ignore_clicks() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Before status")
        .focus();
    h.run_steps(2);
    key(&mut h, Key::Tab);
    assert!(h.get_by_role_and_label(Role::Button, "Sync").is_focused());
    key(&mut h, Key::Tab);
    assert!(
        h.get_by_role_and_label(Role::Button, "After status")
            .is_focused()
    );
    h.get_by_role_and_label(Role::Label, "Unread: 42").click();
    h.run_steps(2);
    assert!(h.state().clicked.is_empty());
}
#[test]
fn disabled_segments_and_bars_neither_focus_nor_activate() {
    for disable_bar in [false, true] {
        let mut items = mail_items();
        if !disable_bar {
            items[2] = StatusItem::labeled(3, "Sync", "Paused")
                .clickable(true)
                .enabled(false);
        }
        let mut h = harness(Page {
            items,
            enabled: !disable_bar,
            ..Default::default()
        });
        h.run_steps(3);
        let sync = h.get_by_role_and_label(Role::Button, "Sync");
        assert!(sync.accesskit_node().is_disabled());
        sync.click();
        h.run_steps(2);
        h.get_by_role_and_label(Role::Button, "Before status")
            .focus();
        h.run_steps(2);
        key(&mut h, Key::Tab);
        assert!(
            h.get_by_role_and_label(Role::Button, "After status")
                .is_focused()
        );
        assert!(h.state().clicked.is_empty());
    }
}
#[test]
fn focus_follows_the_key_when_segments_reorder() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Sync").focus();
    h.run_steps(2);
    h.state_mut().items.swap(0, 2);
    h.run_steps(3);
    assert!(h.get_by_role_and_label(Role::Button, "Sync").is_focused());
    key(&mut h, Key::Enter);
    assert_eq!(h.state().clicked, [3]);
}
