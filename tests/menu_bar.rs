//! Menu bar navigation, host visibility and native menu roles.
mod common;
use dutchy_egui::{MenuBar, MenuBarItem, MenuBarState, MenuEvent, MenuItem, Theme};
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    bar: MenuBarState,
    events: Vec<MenuEvent<u8>>,
    disabled: bool,
    remove: bool,
    rename: bool,
    reverse: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([900.0, 550.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                let before = ui.button("Before bar");
                let nested = [MenuItem::new(3, "Show rulers").checked(true)];
                let file = [
                    MenuItem::new(1, "Save document"),
                    MenuItem::new(2, "View settings").submenu(&nested),
                ];
                let edit = [MenuItem::new(4, "Copy selection")];
                let mut menus = vec![
                    MenuBarItem::new("file", "File", &file),
                    MenuBarItem::new("unused", "Unavailable", &edit).enabled(false),
                    MenuBarItem::new("edit", "Edit", &edit),
                ];
                if s.remove {
                    menus.clear();
                }
                if s.rename && !menus.is_empty() {
                    menus[0].label = "Document".into();
                }
                if s.reverse {
                    menus.reverse();
                }
                let result = MenuBar::new("application", &mut s.bar, &menus)
                    .accessible_name("Editor menus")
                    .enabled(!s.disabled)
                    .focus_when_empty(before.id)
                    .show(ui);
                if let Some(event) = result.event {
                    s.events.push(event);
                }
                let _ = ui.button("After bar");
            },
            State::default(),
        )
}
#[test]
fn one_tab_stop_arrows_switch_open_menus_and_skip_disabled_entries() {
    let mut h = harness();
    h.get_by_label("Before bar").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_role_and_label(Role::MenuItem, "File").is_focused());
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    assert!(h.get_by_role_and_label(Role::MenuItem, "Edit").is_focused());
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    assert!(h.state().bar.is_open("edit"));
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Copy selection")
            .is_focused()
    );
    h.key_press(Key::ArrowRight);
    h.run_steps(5);
    assert!(h.state().bar.is_open("file"));
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Save document")
            .is_focused()
    );
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItemCheckBox, "Show rulers")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.state().bar.is_open("file"));
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.get_by_role_and_label(Role::MenuItem, "File").is_focused());
    assert!(h.state().events.is_empty());
    h.check_accessibility();
}
#[test]
fn root_tab_exit_and_host_open_state_preserve_normal_focus_order() {
    for shift in [false, true] {
        let mut h = harness();
        h.state_mut().bar.open("file");
        h.run_steps(5);
        h.key_press_modifiers(
            if shift {
                Modifiers::SHIFT
            } else {
                Modifiers::NONE
            },
            Key::Tab,
        );
        h.run_steps(4);
        assert!(!h.state().bar.is_open("file"));
        assert!(
            h.get_by_label(if shift { "Before bar" } else { "After bar" })
                .is_focused()
        );
    }
    let mut h = harness();
    h.get_by_role_and_label(Role::MenuItem, "File").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Save document")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().events, [MenuEvent::Activated(1)]);
    assert!(!h.state().bar.is_open("file"));
}

#[test]
fn tab_exits_from_nested_menu_children() {
    for shift in [false, true] {
        let mut h = harness();
        h.state_mut().bar.open("file");
        h.run_steps(4);
        h.key_press(Key::ArrowDown);
        h.run_steps(2);
        h.key_press(Key::ArrowRight);
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItemCheckBox, "Show rulers")
                .is_focused()
        );
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
            h.get_by_label(if shift { "Before bar" } else { "After bar" })
                .is_focused()
        );
    }
}

#[test]
fn tab_moves_from_nested_bar_menu_into_the_following_toolbar() {
    use dutchy_egui::{Menu, Toolbar, ToolbarItem};
    let mut h = Harness::builder()
        .with_size([900.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut MenuBarState| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let before = [
                        MenuItem::new(1, "First inline action"),
                        MenuItem::new(2, "Last inline action"),
                    ];
                    Menu::new("before", &before).show(ui);
                    let children = [MenuItem::new(3, "Nested action")];
                    let file = [MenuItem::new(4, "Export").submenu(&children)];
                    let menus = [MenuBarItem::new("file", "File", &file)];
                    MenuBar::new("bar", state, &menus).show(ui);
                    let tools = [ToolbarItem::action(5, "Following tool")];
                    Toolbar::new("after", &tools).show(ui);
                });
            },
            MenuBarState::default(),
        );
    h.state_mut().open("file");
    h.run_steps(4);
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    h.key_press(Key::Tab);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::Button, "Following tool")
            .is_focused()
    );
}
#[test]
fn removing_or_disabling_an_open_menu_repairs_focus() {
    let mut h = harness();
    h.state_mut().bar.open("file");
    h.run_steps(4);
    h.state_mut().remove = true;
    h.run_steps(3);
    assert!(h.get_by_label("Before bar").is_focused());
    h.state_mut().remove = false;
    h.state_mut().bar.open("edit");
    h.run_steps(4);
    h.state_mut().disabled = true;
    h.run_steps(3);
    assert!(!h.state().bar.is_open("edit"));
    assert!(
        h.query_by_role_and_label(Role::MenuItem, "Copy selection")
            .is_none()
    );
}

#[test]
fn repeated_switching_hover_and_label_changes_keep_focus_and_identity() {
    let mut h = harness();
    for _ in 0..3 {
        h.state_mut().bar.open("file");
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Save document")
                .is_focused()
        );
        h.key_press(Key::ArrowLeft);
        h.run_steps(4);
        assert!(h.state().bar.is_open("edit"));
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Copy selection")
                .is_focused()
        );
        h.key_press(Key::ArrowRight);
        h.run_steps(4);
    }
    let rect = h.get_by_role_and_label(Role::MenuItem, "Edit").rect();
    h.hover_at(rect.center());
    h.run_steps(4);
    assert!(h.state().bar.is_open("edit"));
    h.key_press(Key::Escape);
    h.run_steps(3);
    h.get_by_role_and_label(Role::MenuItem, "File").focus();
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "File").is_focused(),
        "before rename: {:?}",
        h.state().bar
    );
    let id = h
        .get_by_role_and_label(Role::MenuItem, "File")
        .accesskit_node()
        .id();
    h.state_mut().rename = true;
    h.state_mut().reverse = true;
    h.run_steps(3);
    let renamed = h.get_by_role_and_label(Role::MenuItem, "Document");
    assert_eq!(renamed.accesskit_node().id(), id);
    assert!(renamed.is_focused());
}

#[test]
fn arrows_keep_the_open_menu_when_it_is_the_only_enabled_entry() {
    for key in [Key::ArrowRight, Key::ArrowLeft] {
        let mut h = Harness::builder()
            .with_size([700.0, 400.0])
            .with_context_setup(|ctx| {
                common::install(ctx, Theme::default().with_reduced_motion(true))
            })
            .build_ui_state(
                |ui, bar: &mut MenuBarState| {
                    let _ = ui.button("Before bar");
                    let file = [MenuItem::new(1, "Save document"), MenuItem::new(2, "Close")];
                    let edit = [MenuItem::new(3, "Copy selection")];
                    let menus = [
                        MenuBarItem::new("file", "File", &file),
                        MenuBarItem::new("edit", "Edit", &edit).enabled(false),
                    ];
                    let _ = MenuBar::new("application", bar, &menus)
                        .accessible_name("Editor menus")
                        .show(ui);
                },
                MenuBarState::default(),
            );
        h.get_by_label("Before bar").focus();
        h.run();
        h.key_press(Key::Tab);
        h.run_steps(3);
        h.key_press(Key::ArrowDown);
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Save document")
                .is_focused()
        );
        h.key_press(key);
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Save document")
                .is_focused(),
            "{key:?} keeps the focused row of the only enabled menu"
        );
        assert!(!h.get_by_role_and_label(Role::MenuItem, "File").is_focused());
    }
}

fn clicks(h: &mut Harness<'_, State>, label: &str) -> Vec<egui::WidgetInfo> {
    let mut found = Vec::new();
    for _ in 0..4 {
        h.step();
        for event in &h.output().platform_output.events {
            if let egui::output::OutputEvent::Clicked(info) = event
                && info
                    .label
                    .as_deref()
                    .is_some_and(|l| l.eq_ignore_ascii_case(label))
            {
                found.push(info.clone());
            }
        }
    }
    found
}

#[test]
fn bar_entries_report_one_menu_item_event_per_click() {
    let mut h = harness();
    h.run();
    h.get_by_role_and_label(Role::MenuItem, "File").click();
    let clicked = clicks(&mut h, "File");
    assert_eq!(clicked.len(), 1, "{clicked:?}");
    assert_eq!(clicked[0].role, Role::MenuItem);
}

#[test]
fn escape_on_a_closed_bar_entry_releases_focus_like_other_widgets() {
    let mut h = harness();
    h.get_by_label("Before bar").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_role_and_label(Role::MenuItem, "File").is_focused());
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.get_by_role_and_label(Role::MenuItem, "File").is_focused());
}

#[test]
fn up_opens_a_bar_menu_at_its_last_entry() {
    let mut h = harness();
    h.get_by_label("Before bar").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(3);
    h.key_press(Key::ArrowUp);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "View settings")
            .is_focused()
    );
}
