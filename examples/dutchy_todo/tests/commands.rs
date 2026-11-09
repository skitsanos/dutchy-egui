//! Registry-backed palette, remapping and text-editing integration.
use super::harness;
use crate::commands::Action;
use egui::{Key, KeyboardShortcut, Modifiers, accesskit::Role};
use egui_kittest::kittest::Queryable;
#[test]
fn palette_and_shortcuts_reach_the_same_new_task_action() {
    let mut h = harness();
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    h.get_by_role_and_label(Role::Dialog, "Command palette");
    h.get_by_label("Search commands").type_text("new task");
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(5);
    assert!(!h.state().commands.palette.open && h.state().editor.open);
    h.get_by_role_and_label(Role::Dialog, "New task");
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.key_press(Key::N);
    h.run_steps(4);
    assert!(h.state().editor.open);
    h.check_accessibility();
}
#[test]
fn remapping_replaces_the_old_binding_and_text_editing_keeps_unmodified_keys() {
    let mut h = harness();
    h.state_mut()
        .commands
        .registry
        .get_mut(&Action::New)
        .unwrap()
        .shortcut = Some(KeyboardShortcut::new(
        Modifiers::COMMAND | Modifiers::SHIFT,
        Key::N,
    ));
    h.run();
    h.key_press(Key::N);
    h.run();
    assert!(!h.state().editor.open);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::N);
    h.run_steps(4);
    assert!(h.state().editor.open);
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.key_press_modifiers(Modifiers::COMMAND, Key::F);
    h.run_steps(3);
    assert!(h.get_by_label("Search tasks").is_focused());
    h.get_by_label("Search tasks").type_text("N");
    h.run();
    assert!(!h.state().editor.open);
    assert_eq!(h.state().query, "N");
}
#[test]
fn shortcut_settings_record_a_new_chord_without_opening_other_actions() {
    let mut h = harness();
    h.get_by_label("Shortcuts").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::Button, "New task shortcut")
        .click();
    h.run();
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(
        h.state().commands.settings,
        "recorder Escape must not close its drawer"
    );
    h.get_by_role_and_label(Role::Button, "New task shortcut")
        .click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::P);
    h.run_steps(3);
    assert_eq!(
        h.state()
            .commands
            .registry
            .get(&Action::New)
            .unwrap()
            .shortcut,
        Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::P))
    );
    assert!(!h.state().editor.open && !h.state().commands.palette.open);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().commands.settings);
    h.key_press_modifiers(Modifiers::COMMAND, Key::P);
    h.run_steps(4);
    assert!(h.state().editor.open);
    h.check_accessibility();
}
