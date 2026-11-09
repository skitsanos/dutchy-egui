//! Typed dispatch, exact event modifiers, ambiguity, scopes and shared metadata.
use dutchy_egui::{
    Command, CommandRegistry, CommandScope,
    egui::{self, Context, Event, Id, Key, KeyboardShortcut, Modifiers, RawInput},
};
fn context() -> Context {
    let ctx = Context::default();
    let mut output = ctx.run_ui(RawInput::default(), |_| {});
    output.textures_delta.clear();
    ctx
}
fn chord(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND, key)
}
fn key(key: Key, modifiers: Modifiers, repeat: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat,
        modifiers,
    }
}
fn run(
    ctx: &Context,
    registry: &CommandRegistry<u8>,
    mut events: Vec<Event>,
) -> Vec<dutchy_egui::CommandRequest<u8>> {
    let releases: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Key {
                key,
                physical_key,
                modifiers,
                pressed: true,
                ..
            } => Some(Event::Key {
                key: *key,
                physical_key: *physical_key,
                modifiers: *modifiers,
                pressed: false,
                repeat: false,
            }),
            _ => None,
        })
        .collect();
    events.extend(releases);
    let mut requests = Vec::new();
    let mut output = ctx.run_ui(
        RawInput {
            events,
            ..Default::default()
        },
        |ui| {
            let ctx = ui.ctx();
            requests.extend(registry.dispatch(ctx));
        },
    );
    output.textures_delta.clear();
    requests
}
#[test]
fn matching_uses_event_modifiers_and_delivers_once_per_frame() {
    let ctx = context();
    let registry = CommandRegistry::new("app", [Command::new(1, "Save").shortcut(chord(Key::S))]);
    let mut calls = Vec::new();
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![
                key(Key::S, Modifiers::COMMAND, false),
                key(Key::S, Modifiers::COMMAND, false),
            ],
            ..Default::default()
        },
        |ui| {
            let ctx = ui.ctx();
            calls.extend(registry.dispatch(ctx));
            calls.extend(registry.clone().dispatch(ctx));
        },
    );
    output.textures_delta.clear();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].value, 1);
    assert!(run(&ctx, &registry, vec![key(Key::S, Modifiers::COMMAND, true)]).is_empty());
    assert!(
        run(
            &ctx,
            &registry,
            vec![key(Key::S, Modifiers::COMMAND | Modifiers::SHIFT, false)]
        )
        .is_empty()
    );
    assert_eq!(
        run(
            &ctx,
            &registry,
            vec![key(Key::S, Modifiers::COMMAND, false)]
        )
        .len(),
        1
    );
}
#[test]
fn ambiguity_is_reported_and_suppressed_until_the_host_resolves_it() {
    let ctx = context();
    let mut registry = CommandRegistry::new(
        "conflicts",
        [
            Command::new(1, "First").shortcut(chord(Key::S)),
            Command::new(2, "Second").shortcut(chord(Key::S)),
        ],
    );
    assert_eq!(registry.conflicts(&ctx).len(), 1);
    assert!(
        run(
            &ctx,
            &registry,
            vec![key(Key::S, Modifiers::COMMAND, false)]
        )
        .is_empty()
    );
    registry.get_mut(&2).unwrap().enabled = false;
    assert!(registry.conflicts(&ctx).is_empty());
    assert_eq!(
        run(
            &ctx,
            &registry,
            vec![key(Key::S, Modifiers::COMMAND, false)]
        )[0]
        .value,
        1
    );
}
#[test]
fn focused_scope_shadows_application_and_other_targets_can_reuse_the_chord() {
    let ctx = context();
    let target = Id::unique("target");
    let registry = CommandRegistry::new(
        "scopes",
        [
            Command::new(1, "Global").shortcut(chord(Key::S)),
            Command::new(2, "Focused")
                .shortcut(chord(Key::S))
                .scope(CommandScope::Focused(target)),
            Command::new(3, "Other")
                .shortcut(chord(Key::S))
                .scope(CommandScope::Focused(Id::unique("other"))),
        ],
    );
    assert!(registry.conflicts(&ctx).is_empty());
    let mut requests = Vec::new();
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![key(Key::S, Modifiers::COMMAND, false)],
            ..Default::default()
        },
        |ui| {
            let ctx = ui.ctx();
            ctx.memory_mut(|m| m.request_focus(target));
            requests = registry.dispatch(ctx);
        },
    );
    output.textures_delta.clear();
    assert_eq!(requests[0].value, 2);
}
#[test]
fn surface_metadata_tracks_renaming_rebinding_and_rejected_checks() {
    let ctx = context();
    let mut registry = CommandRegistry::new(
        "metadata",
        [Command::new(1, "Show grid")
            .checked(true)
            .shortcut(chord(Key::G))],
    );
    let item = registry.menu_item(&ctx, &1).unwrap();
    let tool = registry.toolbar_item(&ctx, &1).unwrap();
    assert_eq!(item.label, tool.label);
    assert_eq!(item.checked, tool.pressed);
    assert_eq!(
        item.shortcut.as_deref(),
        registry.shortcut_label(&ctx, &1).as_deref()
    );
    assert_eq!(registry.request(&1).unwrap().checked, Some(false));
    assert_eq!(
        registry.request(&1).unwrap().checked,
        Some(false),
        "a rejected request never changes host state"
    );
    registry.insert(Command::new(1, "Grid overlay").shortcut(chord(Key::H)));
    assert_eq!(registry.commands().len(), 1);
    assert_eq!(registry.menu_item(&ctx, &1).unwrap().label, "Grid overlay");
    assert!(
        run(
            &ctx,
            &registry,
            vec![key(Key::G, Modifiers::COMMAND, false)]
        )
        .is_empty()
    );
    assert_eq!(
        run(
            &ctx,
            &registry,
            vec![key(Key::H, Modifiers::COMMAND, false)]
        )[0]
        .value,
        1
    );
    registry.get_mut(&1).unwrap().enabled = false;
    assert!(registry.request(&1).is_none());
    registry.remove(&1);
    assert!(registry.menu_item(&ctx, &1).is_none());
}
#[test]
fn ime_input_is_not_dispatched_and_focus_loss_releases_composition() {
    let ctx = context();
    let registry = CommandRegistry::new(
        "ime",
        [Command::new(1, "New").shortcut(KeyboardShortcut::new(Modifiers::NONE, Key::N))],
    );
    assert!(
        run(
            &ctx,
            &registry,
            vec![
                Event::Ime(egui::ImeEvent::Preedit {
                    text: "候補".into(),
                    active_range_chars: None
                }),
                key(Key::N, Modifiers::NONE, false)
            ]
        )
        .is_empty()
    );
    assert_eq!(
        run(&ctx, &registry, vec![key(Key::N, Modifiers::NONE, false)])[0].value,
        1
    );
}

#[test]
fn platform_aliases_conflict_on_control_platforms_but_not_mac_command() {
    let ctx = context();
    let registry = CommandRegistry::new(
        "aliases",
        [
            Command::new(1, "Portable").shortcut(chord(Key::S)),
            Command::new(2, "Control").shortcut(KeyboardShortcut::new(Modifiers::CTRL, Key::S)),
        ],
    );
    ctx.set_os(egui::os::OperatingSystem::Windows);
    assert_eq!(registry.conflicts(&ctx).len(), 1);
    ctx.set_os(egui::os::OperatingSystem::Mac);
    assert!(registry.conflicts(&ctx).is_empty());
    assert!(!dutchy_egui::commands::same_shortcut(
        &ctx,
        chord(Key::S),
        KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::S)
    ));
}
#[test]
fn discarded_passes_do_not_repeat_requests_or_leak_consumed_chords() {
    let ctx = context();
    let registry =
        CommandRegistry::new("discard", [Command::new(1, "Save").shortcut(chord(Key::S))]);
    let mut requests = Vec::new();
    let mut leaked = false;
    let mut passes = 0;
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![key(Key::S, Modifiers::COMMAND, false)],
            ..Default::default()
        },
        |ui| {
            passes += 1;
            requests.extend(registry.dispatch(ui.ctx()));
            leaked |= ui.input(|input| input.key_pressed(Key::S));
            if ui.ctx().current_pass_index() == 0 {
                ui.ctx().request_discard("verify command event ownership");
            }
        },
    );
    output.textures_delta.clear();
    assert!(passes >= 2);
    assert_eq!(requests.len(), 1);
    assert!(!leaked);
}
#[test]
fn mac_control_is_part_of_the_chord_in_both_directions() {
    let ctx = context();
    ctx.set_os(egui::os::OperatingSystem::Mac);
    let registry = CommandRegistry::new(
        "mac-control",
        [
            Command::new(1, "Save").shortcut(chord(Key::N)),
            Command::new(2, "Window").shortcut(KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::CTRL,
                Key::N,
            )),
        ],
    );
    assert!(registry.conflicts(&ctx).is_empty());
    let pressed = |ctrl| Modifiers {
        mac_cmd: true,
        command: true,
        ctrl,
        ..Default::default()
    };
    let plain = run(&ctx, &registry, vec![key(Key::N, pressed(false), false)]);
    assert_eq!(plain.iter().map(|r| r.value).collect::<Vec<_>>(), [1]);
    let both = run(&ctx, &registry, vec![key(Key::N, pressed(true), false)]);
    assert_eq!(both.iter().map(|r| r.value).collect::<Vec<_>>(), [2]);
}
