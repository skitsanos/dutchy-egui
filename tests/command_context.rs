//! Editing and overlays keep their keys unless a host explicitly opts in.
mod common;
use dutchy_egui::{
    Command, CommandRegistry, Input, Theme,
    egui::{Key, KeyboardShortcut, Modifiers},
};
use egui_kittest::{Harness, kittest::Queryable};
struct State {
    registry: CommandRegistry<u8>,
    text: String,
    requests: Vec<u8>,
    modal: bool,
    popup: bool,
    clicks: usize,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([700.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, s: &mut State| {
                s.requests
                    .extend(s.registry.dispatch(ui.ctx()).into_iter().map(|r| r.value));
                ui.add(Input::new("edit", &mut s.text).accessible_name("Editor"));
                s.clicks += usize::from(ui.button("Outside").clicked());
                dutchy_egui::Dropdown::new(
                    "popup",
                    &mut s.popup,
                    &[dutchy_egui::MenuItem::new(9, "Popup action")],
                )
                .show(ui, |ui| ui.button("Popup"));
                if s.modal {
                    egui::Modal::new(egui::Id::unique("modal")).show(ui.ctx(), |ui| {
                        let _ = ui.button("Modal action");
                    });
                }
            },
            State {
                registry: CommandRegistry::new(
                    "app",
                    [Command::new(1, "New")
                        .shortcut(KeyboardShortcut::new(Modifiers::NONE, Key::N))],
                ),
                text: String::new(),
                requests: Vec::new(),
                modal: false,
                popup: false,
                clicks: 0,
            },
        )
}
#[test]
fn editors_keep_bare_keys_and_explicit_opt_in_can_bind_modified_commands() {
    let mut h = harness();
    h.get_by_label("Editor").focus();
    h.run();
    h.key_press(Key::N);
    h.run();
    assert!(h.state().requests.is_empty());
    h.state_mut().registry.get_mut(&1).unwrap().shortcut =
        Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::S));
    h.state_mut().registry.get_mut(&1).unwrap().allow_in_text = true;
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    h.run();
    assert_eq!(h.state().requests, [1]);
    assert!(h.get_by_label("Editor").is_focused());
}
#[test]
fn modal_blocks_application_bindings_until_explicitly_allowed() {
    let mut h = harness();
    h.state_mut().modal = true;
    h.run_steps(3);
    h.key_press(Key::N);
    h.run();
    assert!(h.state().requests.is_empty());
    h.state_mut()
        .registry
        .get_mut(&1)
        .unwrap()
        .allow_in_overlays = true;
    h.key_press(Key::N);
    h.run();
    assert_eq!(h.state().requests, [1]);
}

#[test]
fn host_owned_popup_flags_also_protect_keyboard_input() {
    let mut h = harness();
    h.state_mut().popup = true;
    h.run_steps(4);
    h.key_press(Key::N);
    h.run();
    assert!(h.state().requests.is_empty());
    h.state_mut()
        .registry
        .get_mut(&1)
        .unwrap()
        .allow_in_overlays = true;
    h.key_press(Key::N);
    h.run();
    assert_eq!(h.state().requests, [1]);
    h.state_mut().popup = false;
    h.run_steps(3);
    h.key_press(Key::N);
    h.run();
    assert_eq!(h.state().requests, [1, 1]);
}
#[test]
fn focused_controls_keep_unmodified_activation_and_navigation_keys() {
    let mut h = harness();
    let bind = |modifiers, key| Some(KeyboardShortcut::new(modifiers, key));
    h.state_mut().registry.get_mut(&1).unwrap().shortcut = bind(Modifiers::NONE, Key::Enter);
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(
        h.state().requests,
        [1],
        "without focus the host binding applies"
    );
    h.get_by_label("Outside").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().clicks, 1, "the focused button keeps Enter");
    h.state_mut().registry.get_mut(&1).unwrap().shortcut = bind(Modifiers::NONE, Key::ArrowDown);
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(h.state().requests, [1], "focused navigation keeps arrows");
    h.get_by_label("Outside").focus();
    h.state_mut().registry.get_mut(&1).unwrap().shortcut = bind(Modifiers::COMMAND, Key::Enter);
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    h.run();
    assert_eq!(
        h.state().requests,
        [1, 1],
        "modified chords still reach the host"
    );
}
