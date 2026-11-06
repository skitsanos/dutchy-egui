//! Menu grouping, native activation, stable identities, and disabled actions.
mod common;
use dutchy_egui::{Menu, MenuItem, MenuItemVariant, MenuWidth, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn items() -> Vec<MenuItem<'static, u8>> {
    vec![
        MenuItem::new(1, "Profile")
            .group("Account")
            .shortcut("Ctrl+P"),
        MenuItem::new(2, "Unavailable")
            .group("Account")
            .enabled(false),
        MenuItem::new(3, "Sign out")
            .group("Account")
            .variant(MenuItemVariant::Destructive)
            .separator_before(true),
        MenuItem::new(4, "Help").group("Support"),
    ]
}
#[derive(Default)]
struct State {
    actions: Vec<u8>,
    reversed: bool,
    disabled: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([600.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let style = ui.style().clone();
                let mut entries = items();
                if state.reversed {
                    ui.label("Inserted context");
                    entries.reverse();
                }
                let result = Menu::new("account", &entries)
                    .accessible_name("Account actions")
                    .enabled(!state.disabled)
                    .show(ui);
                if let Some(action) = result.action {
                    state.actions.push(action);
                }
                assert_eq!(style.as_ref(), ui.style().as_ref());
            },
            State::default(),
        )
}
#[test]
fn groups_shortcuts_and_actions_are_named_without_binding_application_shortcuts() {
    let mut h = harness();
    assert!(
        h.query_by_role_and_label(Role::Menu, "Account actions")
            .is_some()
    );
    assert!(h.query_by_role_and_label(Role::Group, "Account").is_some());
    assert_eq!(
        h.get_by_role_and_label(Role::MenuItem, "Profile")
            .accesskit_node()
            .description()
            .as_deref(),
        Some("Shortcut: Ctrl+P")
    );
    h.get_by_role_and_label(Role::MenuItem, "Profile").click();
    h.run();
    assert_eq!(h.state().actions, [1]);
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Sign out")
            .is_focused()
    );
    h.key_press(Key::Enter);
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().actions, [1, 3, 3]);
    h.key_press(Key::End);
    h.run();
    assert!(h.get_by_role_and_label(Role::MenuItem, "Help").is_focused());
    h.key_press(Key::Home);
    h.run();
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Profile")
            .is_focused()
    );
    h.check_accessibility();
}
#[test]
fn identities_survive_group_reordering_and_disabled_menus_cannot_activate() {
    let mut h = harness();
    h.get_by_role_and_label(Role::MenuItem, "Profile").focus();
    h.run();
    let id = h
        .get_by_role_and_label(Role::MenuItem, "Profile")
        .accesskit_node()
        .id();
    h.state_mut().reversed = true;
    h.run_steps(2);
    assert_eq!(
        h.get_by_role_and_label(Role::MenuItem, "Profile")
            .accesskit_node()
            .id(),
        id
    );
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Profile")
            .is_focused()
    );
    h.get_by_role_and_label(Role::MenuItem, "Unavailable")
        .click();
    h.run();
    h.state_mut().disabled = true;
    h.run();
    h.get_by_role_and_label(Role::MenuItem, "Sign out").click();
    h.run();
    assert!(h.state().actions.is_empty());
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Profile")
            .accesskit_node()
            .is_disabled()
    );
}
#[test]
fn narrow_menu_labels_wrap_and_empty_menus_are_safe() {
    let mut h = Harness::builder()
        .with_size([240.0, 700.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.available_rect_before_wrap();
            let entries = [MenuItem::new(
                1,
                "A very long menu action that wraps inside its available width",
            )
            .shortcut("Ctrl+Shift+P")];
            let result = Menu::new("long", &entries).width(MenuWidth::Large).show(ui);
            assert!(result.response.rect.right() <= before.right() + 1.0);
            let empty = Menu::new("empty", &[] as &[MenuItem<'_, u8>]).show(ui);
            assert!(empty.items.is_empty() && empty.action.is_none());
        });
    h.run();
}
