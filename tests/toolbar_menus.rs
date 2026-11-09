//! Toolbar menu buttons preserve the toolbar's host event and focus contract.
mod common;
use dutchy_egui::{Icon, MenuItem, Theme, Toolbar, ToolbarEvent, ToolbarItem, ToolbarOrientation};
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[derive(Default)]
struct State {
    events: Vec<ToolbarEvent<u8>>,
    remove: bool,
    vertical: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([900.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                let before = ui.button("Before toolbar");
                let formats = [MenuItem::new(4, "SVG")];
                let commands = [
                    MenuItem::new(2, "Wrap lines").checked(false),
                    MenuItem::new(3, "Export drawing").submenu(&formats),
                ];
                let mut items = vec![
                    ToolbarItem::action(0, "Save").icon(Icon::Save),
                    ToolbarItem::action(1, "More tools")
                        .icon(Icon::More)
                        .menu(&commands),
                    ToolbarItem::action(5, "Undo").icon(Icon::Undo),
                ];
                if s.remove {
                    items.clear();
                }
                let result = Toolbar::new("editor", &items)
                    .accessible_name("Editor tools")
                    .focus_when_empty(before.id)
                    .orientation(if s.vertical {
                        ToolbarOrientation::Vertical
                    } else {
                        ToolbarOrientation::Horizontal
                    })
                    .show(ui);
                if let Some(event) = result.event {
                    s.events.push(event);
                }
                let _ = ui.button("After toolbar");
            },
            State::default(),
        )
}
#[test]
fn menu_leafs_use_the_toolbar_event_channel_without_executing_the_trigger() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "More tools").click();
    h.run_steps(4);
    assert!(h.state().events.is_empty());
    h.key_press(Key::Space);
    h.run_steps(3);
    assert_eq!(
        h.state().events,
        [ToolbarEvent::Toggled {
            value: 2,
            pressed: true
        }]
    );
    assert!(
        h.get_by_role_and_label(Role::Button, "More tools")
            .is_focused()
    );
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().events.last(), Some(&ToolbarEvent::Activated(4)));
    assert!(
        h.get_by_role_and_label(Role::Button, "More tools")
            .is_focused()
    );
    h.check_accessibility();
}
#[test]
fn tab_exits_the_group_and_removal_repairs_child_focus() {
    for shift in [false, true] {
        let mut h = harness();
        h.get_by_role_and_label(Role::Button, "More tools").click();
        h.run_steps(4);
        h.key_press_modifiers(
            if shift {
                Modifiers::SHIFT
            } else {
                Modifiers::NONE
            },
            Key::Tab,
        );
        h.run_steps(4);
        assert!(
            h.get_by_label(if shift {
                "Before toolbar"
            } else {
                "After toolbar"
            })
            .is_focused()
        );
        assert!(
            h.query_by_role_and_label(Role::MenuItemCheckBox, "Wrap lines")
                .is_none()
        );
    }
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "More tools").click();
    h.run_steps(4);
    h.state_mut().remove = true;
    h.run_steps(3);
    assert!(h.get_by_label("Before toolbar").is_focused());
}
#[test]
fn vertical_arrows_navigate_tools_and_right_opens_the_menu() {
    let mut h = harness();
    h.state_mut().vertical = true;
    h.run();
    h.get_by_label("Save").focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Button, "More tools")
            .is_focused()
    );
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItemCheckBox, "Wrap lines")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Button, "More tools")
            .is_focused()
    );
}

#[test]
fn escape_treats_menu_buttons_like_the_other_tools() {
    for name in ["Save", "More tools"] {
        let mut h = harness();
        h.get_by_role_and_label(Role::Button, name).focus();
        h.run_steps(2);
        assert!(h.get_by_role_and_label(Role::Button, name).is_focused());
        h.key_press(Key::Escape);
        h.run_steps(3);
        assert!(
            !h.get_by_role_and_label(Role::Button, name).is_focused(),
            "{name}: Escape on a closed tool follows egui's native focus release"
        );
    }
}

#[test]
fn up_opens_a_toolbar_menu_at_its_last_entry() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "More tools").focus();
    h.run_steps(2);
    h.key_press(Key::ArrowUp);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Export drawing")
            .is_focused()
    );
}

#[test]
fn icon_only_menu_buttons_keep_the_square_tool_target() {
    let mut h = harness();
    h.run_steps(2);
    let save = h.get_by_role_and_label(Role::Button, "Save").rect();
    let menu = h.get_by_role_and_label(Role::Button, "More tools").rect();
    assert!((menu.width() - menu.height()).abs() < 0.5, "{menu:?}");
    assert!(
        (menu.size() - save.size()).length() < 0.5,
        "{menu:?} vs {save:?}"
    );
}
