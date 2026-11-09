//! Native command controls and interaction regressions.
mod common;
use dutchy_egui::{
    Command, CommandRegistry, KeyRecorder, Theme,
    egui::{self, Key, KeyboardShortcut, Modifiers, accesskit::Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
struct State {
    value: Option<KeyboardShortcut>,
    registry: CommandRegistry<u8>,
    actions: usize,
    recording: bool,
    conflict: bool,
    read_only: bool,
    enabled: bool,
    discard: bool,
    navigation: bool,
}
fn chord(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND, key)
}
fn harness() -> Harness<'static, State> {
    harness_on(egui::os::OperatingSystem::Nix)
}
fn harness_on(os: egui::os::OperatingSystem) -> Harness<'static, State> {
    Harness::builder()
        .with_os(os)
        .with_size([700.0, 450.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, s: &mut State| {
                s.actions += s.registry.dispatch(ui.ctx()).len();
                let out = KeyRecorder::new("save", &mut s.value)
                    .accessible_name("Record Save shortcut")
                    .reserved(&[chord(Key::K)])
                    .read_only(s.read_only)
                    .enabled(s.enabled)
                    .allow_navigation_keys(s.navigation)
                    .show(ui);
                s.recording = out.recording;
                s.conflict = out.conflict;
                let _ = ui.button("After recorder");
                if s.discard && ui.ctx().current_pass_index() == 0 {
                    ui.ctx().request_discard("record once");
                }
            },
            State {
                value: Some(chord(Key::S)),
                registry: CommandRegistry::new(
                    "commands",
                    [Command::new(1, "Test binding").shortcut(chord(Key::P))],
                ),
                actions: 0,
                recording: false,
                conflict: false,
                read_only: false,
                enabled: true,
                discard: false,
                navigation: false,
            },
        )
}
#[test]
fn capture_changes_host_binding_and_suppresses_registry_dispatch() {
    let mut h = harness();
    h.get_by_label("Record Save shortcut").click();
    h.run();
    assert!(h.state().recording);
    h.key_press_modifiers(Modifiers::COMMAND, Key::P);
    h.run_steps(3);
    assert_eq!(h.state().value, Some(chord(Key::P)));
    assert!(!h.state().recording);
    assert_eq!(h.state().actions, 0);
    h.get_by_label("After recorder").focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::P);
    h.run();
    assert_eq!(h.state().actions, 1);
    h.check_accessibility();
}
#[test]
fn reserved_chords_are_rejected_and_escape_preserves_original_value() {
    let mut h = harness();
    h.get_by_label("Record Save shortcut").click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(3);
    assert!(h.state().conflict && h.state().recording);
    assert_eq!(h.state().value, Some(chord(Key::S)));
    h.get_by_role_and_label(Role::Alert, "Error: Shortcut already in use");
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.state().recording);
    assert_eq!(h.state().value, Some(chord(Key::S)));
}
#[test]
fn tab_cancels_and_clear_is_explicit_and_read_only_never_edits() {
    let mut h = harness();
    h.get_by_label("Record Save shortcut").click();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(!h.state().recording);
    h.get_by_label("Clear Record Save shortcut").click();
    h.run();
    assert!(h.state().value.is_none());
    h.state_mut().value = Some(chord(Key::S));
    h.state_mut().read_only = true;
    h.run();
    h.get_by_label("Record Save shortcut").click();
    h.run();
    assert!(!h.state().recording);
    assert_eq!(h.state().value, Some(chord(Key::S)));
    h.state_mut().read_only = false;
    h.get_by_label("Record Save shortcut").click();
    h.run();
    h.state_mut().enabled = false;
    h.run();
    assert!(!h.state().recording);
}

#[test]
fn captured_enter_does_not_restart_recording_on_a_discarded_pass() {
    let mut h = harness();
    // Bare Enter needs the host's explicit opt-in.
    h.state_mut().navigation = true;
    h.get_by_label("Record Save shortcut").click();
    h.run();
    h.state_mut().discard = true;
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(!h.state().recording);
    assert_eq!(
        h.state().value,
        Some(KeyboardShortcut::new(Modifiers::NONE, Key::Enter))
    );
    assert_eq!(h.state().actions, 0);
}
#[test]
fn command_control_chords_keep_control_on_mac_and_fold_elsewhere() {
    let mut h = harness_on(egui::os::OperatingSystem::Mac);
    h.get_by_label("Record Save shortcut").click();
    h.run();
    let pressed = Modifiers {
        mac_cmd: true,
        command: true,
        ctrl: true,
        ..Default::default()
    };
    h.key_press_modifiers(pressed, Key::N);
    h.run_steps(3);
    let both = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::CTRL, Key::N);
    assert_eq!(h.state().value, Some(both));
    let description = h
        .get_by_label("Record Save shortcut")
        .accesskit_node()
        .description();
    assert_eq!(description.as_deref(), Some("Current shortcut: ⌃⌘N"));
    let mut h = harness_on(egui::os::OperatingSystem::Windows);
    h.get_by_label("Record Save shortcut").click();
    h.run();
    h.key_press_modifiers(Modifiers::CTRL | Modifiers::COMMAND, Key::N);
    h.run_steps(3);
    assert_eq!(h.state().value, Some(chord(Key::N)));
}
#[test]
fn unmodified_activation_and_navigation_keys_need_a_modifier() {
    let mut h = harness();
    h.get_by_label("Record Save shortcut").click();
    h.run();
    for (modifiers, key) in [
        (Modifiers::NONE, Key::Enter),
        (Modifiers::NONE, Key::Space),
        (Modifiers::NONE, Key::ArrowDown),
        (Modifiers::NONE, Key::PageUp),
        (Modifiers::SHIFT, Key::Home),
    ] {
        h.key_press_modifiers(modifiers, key);
        h.run_steps(3);
        assert!(h.state().recording && h.state().conflict, "{key:?}");
        assert_eq!(h.state().value, Some(chord(Key::S)), "{key:?}");
    }
    h.get_by_role_and_label(Role::Alert, "Error: Key needs a modifier");
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    h.run_steps(3);
    assert!(!h.state().recording && !h.state().conflict);
    assert_eq!(
        h.state().value,
        Some(KeyboardShortcut::new(Modifiers::ALT, Key::ArrowDown))
    );
}
