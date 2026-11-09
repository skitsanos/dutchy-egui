//! Controlled checks and nested command navigation use one shared menu tree.
mod common;
use dutchy_egui::{Button, Dropdown, Input, MenuEvent, MenuItem, Theme};
use egui::{
    Key, Modifiers,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    open: bool,
    checked: bool,
    accept: bool,
    remove: bool,
    disable: bool,
    reverse: bool,
    outside: String,
    events: Vec<MenuEvent<u8>>,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([1000.0, 650.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                let _ = ui.button("Before menu");
                let formats = [
                    MenuItem::new(3, "PNG"),
                    MenuItem::new(4, "Unavailable format").enabled(false),
                ];
                let export = [MenuItem::new(2, "Image").submenu(&formats)];
                let mut items = vec![
                    MenuItem::new(1, "Show grid").checked(s.checked),
                    MenuItem::new(9, "Empty branch").submenu(&[]),
                ];
                if !s.remove {
                    items.push(
                        MenuItem::new(2, "Export")
                            .submenu(&export)
                            .enabled(!s.disable),
                    );
                }
                if s.reverse {
                    items.reverse();
                }
                let result = Dropdown::new("nested", &mut s.open, &items)
                    .show(ui, |ui| ui.add(Button::new("Commands")));
                if let Some(event) = result.menu.and_then(|m| m.event) {
                    if s.accept
                        && let MenuEvent::Checked { checked, .. } = event
                    {
                        s.checked = checked;
                    }
                    s.events.push(event);
                }
                ui.add(Input::new("outside", &mut s.outside).accessible_name("Outside input"));
            },
            State::default(),
        )
}
fn enter_png(h: &mut Harness<'_, State>) {
    h.get_by_label("Commands").focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Export")
            .is_focused()
    );
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Image")
            .is_focused()
    );
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    assert!(h.get_by_role_and_label(Role::MenuItem, "PNG").is_focused());
}
#[test]
fn checks_report_requests_without_mutating_host_values() {
    let mut h = harness();
    for accept in [false, true] {
        h.state_mut().accept = accept;
        h.get_by_label("Commands").click();
        h.run_steps(4);
        let item = h.get_by_role_and_label(Role::MenuItemCheckBox, "Show grid");
        assert_eq!(item.accesskit_node().data().toggled(), Some(Toggled::False));
        item.click();
        h.run_steps(3);
        assert_eq!(h.state().checked, accept);
        assert!(!h.state().open);
        assert_eq!(
            h.state().events.last(),
            Some(&MenuEvent::Checked {
                value: 1,
                checked: true
            })
        );
        assert!(h.get_by_label("Commands").is_focused());
    }
    h.get_by_label("Commands").click();
    h.run_steps(4);
    assert_eq!(
        h.get_by_role_and_label(Role::MenuItemCheckBox, "Show grid")
            .accesskit_node()
            .data()
            .toggled(),
        Some(Toggled::True)
    );
    h.check_accessibility();
}
#[test]
fn escape_and_left_close_one_level_then_leaf_activation_closes_the_tree() {
    let mut h = harness();
    enter_png(&mut h);
    h.get_by_role_and_label(Role::Menu, "Image")
        .get_by_role_and_label(Role::MenuItem, "PNG");
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(
        h.state().open
            && h.get_by_role_and_label(Role::MenuItem, "Image")
                .is_focused()
    );
    assert!(h.query_by_role_and_label(Role::MenuItem, "PNG").is_none());
    h.key_press(Key::ArrowLeft);
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Export")
            .is_focused()
    );
    h.key_press(Key::Enter);
    h.run_steps(4);
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(h.state().events.is_empty());
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().events, [MenuEvent::Activated(3)]);
    assert!(!h.state().open && h.get_by_label("Commands").is_focused());
}
#[test]
fn tab_and_outside_click_exit_from_nested_children() {
    for shift in [false, true] {
        let mut h = harness();
        enter_png(&mut h);
        h.key_press_modifiers(
            if shift {
                Modifiers::SHIFT
            } else {
                Modifiers::NONE
            },
            Key::Tab,
        );
        h.run_steps(4);
        assert!(!h.state().open && h.state().events.is_empty());
        assert!(
            h.get_by_label(if shift {
                "Before menu"
            } else {
                "Outside input"
            })
            .is_focused()
        );
    }
    let mut h = harness();
    enter_png(&mut h);
    h.get_by_label("Outside input").click();
    h.run_steps(4);
    assert!(!h.state().open && h.get_by_label("Outside input").is_focused());
}
#[test]
fn pointer_children_checks_metadata_and_repair_survive_reorder_and_removal() {
    let mut h = harness();
    enter_png(&mut h);
    let id = h
        .get_by_role_and_label(Role::MenuItem, "PNG")
        .accesskit_node()
        .id();
    h.state_mut().reverse = true;
    h.run_steps(3);
    assert_eq!(
        id,
        h.get_by_role_and_label(Role::MenuItem, "PNG")
            .accesskit_node()
            .id()
    );
    assert_eq!(
        h.get_by_role_and_label(Role::MenuItem, "Image")
            .accesskit_node()
            .data()
            .is_expanded(),
        Some(true)
    );
    h.state_mut().disable = true;
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::MenuItem, "PNG").is_none());
    assert!(
        h.get_by_role_and_label(Role::MenuItemCheckBox, "Show grid")
            .is_focused()
    );
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Empty branch")
            .accesskit_node()
            .is_disabled()
    );
    h.state_mut().disable = false;
    h.run();
    let rect = h.get_by_role_and_label(Role::MenuItem, "Export").rect();
    h.hover_at(rect.center());
    h.run_steps(4);
    let rect = h.get_by_role_and_label(Role::MenuItem, "Image").rect();
    h.hover_at(rect.center());
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "PNG").click();
    h.run_steps(3);
    assert_eq!(h.state().events, [MenuEvent::Activated(3)]);
}
