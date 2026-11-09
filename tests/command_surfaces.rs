//! One command catalog drives menus, tools, keyboard, remapping and nested search.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/command_surfaces.rs"]
mod studio;
use dutchy_egui::{
    Theme,
    egui::{Key, Modifiers, accesskit::Role},
};
use egui_kittest::{Harness, kittest::Queryable};
fn harness() -> Harness<'static, studio::CommandState> {
    Harness::builder()
        .with_size([1000.0, 850.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(studio::show, studio::CommandState::default())
}
fn feedback(h: &Harness<'_, studio::CommandState>, text: &str) -> bool {
    common::galleys(&h.output().shapes)
        .iter()
        .any(|galley| galley.job.text.contains(text))
}
#[test]
fn save_is_shared_by_shortcut_toolbar_menu_and_remapped_binding() {
    let mut h = harness();
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    h.run();
    assert!(feedback(&h, "Saved 1 times"));
    h.get_by_role_and_label(Role::Button, "Save document")
        .click();
    h.run();
    assert!(feedback(&h, "Saved 2 times"));
    h.get_by_role_and_label(Role::MenuItem, "File").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Save document")
        .click();
    h.run_steps(4);
    assert!(feedback(&h, "Saved 3 times"));
    h.get_by_role_and_label(Role::Button, "Save shortcut")
        .click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::P);
    h.run_steps(3);
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    h.run();
    assert!(feedback(&h, "Saved 3 times"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::P);
    h.run();
    assert!(feedback(&h, "Saved 4 times"));
    h.check_accessibility();
}
#[test]
fn nested_search_and_split_primary_route_to_the_same_export() {
    let mut h = harness();
    h.get_by_label("Export PNG").click();
    h.run();
    assert!(feedback(&h, "PNG export requested"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    h.get_by_role_and_label(Role::Dialog, "Canal Studio commands");
    h.get_by_label("Search commands")
        .type_text("file export png");
    h.run_steps(3);
    h.get_by_role_and_label(Role::ListBoxOption, "PNG image");
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(feedback(&h, "PNG export requested"));
    assert!(!h.state().palette.open);
}
#[test]
fn checked_requests_follow_host_acceptance_for_keyboard_and_palette() {
    let mut h = harness();
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::G);
    h.run();
    assert!(!h.state().grid);
    h.get_by_role_and_label(Role::Switch, "Apply view changes")
        .click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::G);
    h.run();
    assert!(!h.state().grid);
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    h.get_by_label("Search commands").type_text("show grid");
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(!h.state().grid);
    assert!(feedback(&h, "View change declined"));
}
