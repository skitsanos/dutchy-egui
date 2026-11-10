use super::support::*;
use egui::{
    Key, Modifiers,
    accesskit::{Action, ActionData, Role},
};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn keyboard_steps_home_end_and_resize_do_not_shift_following_content() {
    let mut h = harness(Page::default());
    focus(&mut h);
    let below = h.state().below;
    key(&mut h, Key::ArrowRight, Modifiers::NONE);
    assert_eq!(h.state().first, 168.0);
    key(&mut h, Key::ArrowRight, Modifiers::SHIFT);
    assert_eq!(h.state().first, 200.0);
    key(&mut h, Key::Home, Modifiers::NONE);
    assert_eq!(h.state().first, 80.0);
    key(&mut h, Key::End, Modifiers::NONE);
    assert_eq!(h.state().rects[2].width(), 100.0);
    assert_eq!(below, h.state().below);
    assert!(
        h.get_by_role_and_label(Role::Splitter, "Folders")
            .is_focused()
    );
    h.check_accessibility();
}
#[test]
fn vertical_resize_uses_up_down_and_leaves_host_shortcuts_alone() {
    let mut h = harness(Page {
        vertical: true,
        first: 100.0,
        ..Default::default()
    });
    focus(&mut h);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert_eq!(h.state().first, 108.0);
    key(&mut h, Key::ArrowUp, Modifiers::SHIFT);
    assert_eq!(h.state().first, 80.0);
    key(&mut h, Key::ArrowDown, Modifiers::COMMAND);
    assert_eq!(h.state().first, 80.0);
    h.event(egui::Event::Key {
        key: Key::ArrowLeft,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(1);
    assert!(h.state().remaining_keys.contains(&Key::ArrowLeft));
}
#[test]
fn cross_axis_arrows_neither_resize_nor_move_focus_off_the_divider() {
    for vertical in [false, true] {
        let mut h = harness(Page {
            vertical,
            ..Default::default()
        });
        focus(&mut h);
        let size = h.state().first;
        let cross = if vertical {
            [Key::ArrowLeft, Key::ArrowRight]
        } else {
            [Key::ArrowUp, Key::ArrowDown]
        };
        for arrow in cross {
            key(&mut h, arrow, Modifiers::NONE);
            assert_eq!(h.state().first, size, "{arrow:?}");
            assert!(
                h.get_by_role_and_label(Role::Splitter, "Folders")
                    .is_focused(),
                "{arrow:?} on a vertical={vertical} divider moved focus"
            );
        }
    }
}
#[test]
fn divider_is_one_tab_stop_between_native_child_controls() {
    let mut h = harness(Page::default());
    h.get_by_label("Draft").focus();
    h.run_steps(3);
    key(&mut h, Key::Tab, Modifiers::NONE);
    assert!(
        h.get_by_role_and_label(Role::Splitter, "Folders")
            .is_focused()
    );
    key(&mut h, Key::Tab, Modifiers::NONE);
    assert!(h.get_by_label("Second pane action").is_focused());
    key(&mut h, Key::Tab, Modifiers::SHIFT);
    assert!(
        h.get_by_role_and_label(Role::Splitter, "Folders")
            .is_focused()
    );
    key(&mut h, Key::Tab, Modifiers::SHIFT);
    assert!(h.get_by_label("Draft").is_focused());
    h.check_accessibility();
}
#[test]
fn dragging_keeps_the_grab_offset_applies_release_position_and_clamps() {
    for discard in [false, true] {
        let mut h = harness(Page {
            discard,
            ..Default::default()
        });
        h.run_steps(3);
        let start = h.state().rects[1].center();
        h.hover_at(start);
        h.run_steps(2);
        button(&mut h, start, true);
        let next = start + egui::vec2(50.0, 0.0);
        h.event(egui::Event::PointerMoved(next));
        h.run_steps(3);
        assert_eq!(h.state().first, 210.0);
        button(&mut h, next + egui::vec2(20.0, 0.0), false);
        assert_eq!(h.state().first, 230.0);
        let start = h.state().rects[1].center();
        h.hover_at(start);
        h.run_steps(2);
        button(&mut h, start, true);
        let next = start - egui::vec2(1000.0, 0.0);
        h.event(egui::Event::PointerMoved(next));
        h.run_steps(3);
        assert_eq!(h.state().first, 80.0);
        assert_eq!(
            h.output().platform_output.cursor_icon,
            egui::CursorIcon::ResizeHorizontal
        );
        button(&mut h, next, false);
    }
}
#[test]
fn accessible_splitter_reports_orientation_range_value_and_controlled_pane() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    let split = h.get_by_role_and_label(Role::Splitter, "Folders");
    let n = split.accesskit_node();
    assert_eq!(n.numeric_value(), Some(160.0));
    assert_eq!(n.min_numeric_value(), Some(80.0));
    assert_eq!(
        n.orientation(),
        Some(egui::accesskit::Orientation::Vertical)
    );
    assert_eq!(n.data().controls(), &[h.state().first_id.accesskit_id()]);
    let (target_node, target_tree) = n.locate();
    h.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            target_node,
            target_tree,
            action: Action::SetValue,
            data: Some(ActionData::NumericValue(260.0)),
        },
    ));
    h.run_steps(3);
    assert_eq!(h.state().first, 260.0);
    h.event(egui::Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            target_node,
            target_tree,
            action: Action::Increment,
            data: None,
        },
    ));
    h.run_steps(3);
    assert_eq!(h.state().first, 268.0);
    h.check_accessibility();
}
#[test]
fn read_only_keeps_child_controls_usable_while_disabled_blocks_all_interaction() {
    let mut h = harness(Page {
        read_only: true,
        ..Default::default()
    });
    focus(&mut h);
    key(&mut h, Key::ArrowRight, Modifiers::NONE);
    assert_eq!(h.state().first, 160.0);
    h.get_by_label("Draft").focus();
    h.run_steps(2);
    h.get_by_label("Draft").type_text("Host draft");
    h.run_steps(3);
    assert_eq!(h.state().text, "Host draft");
    h.state_mut().read_only = false;
    h.state_mut().enabled = false;
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Splitter, "Folders")
            .accesskit_node()
            .is_disabled()
    );
    assert!(h.get_by_label("Draft").accesskit_node().is_disabled());
    h.state_mut().enabled = true;
    h.state_mut().parent_disabled = true;
    h.run_steps(3);
    assert!(
        h.get_by_label("Second pane action")
            .accesskit_node()
            .is_disabled()
    );
    h.check_accessibility();
}
#[test]
fn native_child_focus_and_draft_survive_resize_and_inserted_siblings() {
    let mut h = harness(Page::default());
    h.get_by_label("Draft").focus();
    h.run_steps(3);
    let id = h.get_by_label("Draft").accesskit_node().id();
    h.state_mut().first = 240.0;
    h.state_mut().inserted = true;
    h.run_steps(3);
    assert_eq!(h.get_by_label("Draft").accesskit_node().id(), id);
    assert!(h.get_by_label("Draft").is_focused());
    h.get_by_label("Draft").type_text("Still editing");
    h.run_steps(3);
    assert_eq!(h.state().text, "Still editing");
    h.check_accessibility();
}
