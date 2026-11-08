//! Native icon/toggle behavior, state metadata and stable identity.
mod common;
use dutchy_egui::{ControlSize, Icon, IconButton, Theme, ToggleButton};
use egui::{
    Key,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    on: bool,
    enabled: bool,
    parent: bool,
    busy: bool,
    prefix: bool,
    size: ControlSize,
    actions: usize,
    changes: usize,
}
fn harness() -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([640.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                if s.prefix {
                    ui.label("Inserted sibling");
                }
                ui.add_enabled_ui(s.parent, |ui| {
                    if ui
                        .add(
                            IconButton::new("copy", "Copy selection", Icon::Copy)
                                .size(s.size)
                                .enabled(s.enabled)
                                .loading(s.busy),
                        )
                        .clicked()
                    {
                        s.actions += 1;
                    }
                    let response = ui.add(
                        ToggleButton::new("bold", &mut s.on, "Bold")
                            .icon(Icon::Bold)
                            .size(s.size)
                            .enabled(s.enabled)
                            .loading(s.busy),
                    );
                    s.changes += usize::from(response.changed());
                    ui.add(IconButton::new("unicode", "Inspect 🦀", Icon::Search).tooltip(""));
                });
            },
            State {
                on: false,
                enabled: true,
                parent: true,
                busy: false,
                prefix: false,
                size: ControlSize::Medium,
                actions: 0,
                changes: 0,
            },
        );
    h.run();
    h
}

#[test]
fn named_icons_activate_by_pointer_enter_and_space_and_keep_names_as_written() {
    let mut h = harness();
    h.check_accessibility();
    h.get_by_role_and_label(Role::Button, "Copy selection")
        .click();
    h.run();
    assert_eq!(h.state().actions, 1);
    h.key_press(Key::Enter);
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().actions, 3);
    assert!(
        h.get_by_role_and_label(Role::Button, "Copy selection")
            .is_focused()
    );
    h.get_by_role_and_label(Role::Button, "Inspect 🦀");
}

#[test]
fn toggles_bind_host_state_and_report_pressed_without_changing_their_names() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "Bold").click();
    h.run();
    assert!(h.state().on);
    assert_eq!(h.state().changes, 1);
    assert_eq!(
        h.get_by_role_and_label(Role::Button, "Bold")
            .accesskit_node()
            .data()
            .toggled(),
        Some(Toggled::True)
    );
    h.key_press(Key::Space);
    h.run();
    assert!(!h.state().on);
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().on);
    assert_eq!(h.state().changes, 3);
    h.state_mut().on = false;
    h.run();
    assert_eq!(h.state().changes, 3);
    assert_eq!(
        h.get_by_role_and_label(Role::Button, "Bold")
            .accesskit_node()
            .data()
            .toggled(),
        Some(Toggled::False)
    );
}

#[test]
fn busy_disabled_and_disabled_parents_never_activate_or_toggle() {
    let mut h = harness();
    for (enabled, parent, busy) in [
        (true, true, true),
        (false, true, false),
        (true, false, false),
    ] {
        h.state_mut().enabled = enabled;
        h.state_mut().parent = parent;
        h.state_mut().busy = busy;
        h.run();
        for label in ["Copy selection", "Bold"] {
            let button = h.get_by_role_and_label(Role::Button, label);
            assert!(button.accesskit_node().is_disabled());
            assert_eq!(button.accesskit_node().data().is_busy(), busy);
            button.click();
            h.run();
        }
    }
    assert_eq!(h.state().actions, 0);
    assert_eq!(h.state().changes, 0);
    assert!(!h.state().on);
}

#[test]
fn explicit_ids_preserve_focus_and_icon_targets_follow_all_shared_sizes() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "Bold").focus();
    h.run();
    let id = h.ctx.memory(|m| m.focused());
    h.state_mut().prefix = true;
    h.run();
    assert_eq!(h.ctx.memory(|m| m.focused()), id);
    for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
        h.state_mut().size = size;
        h.run();
        let r = h
            .get_by_role_and_label(Role::Button, "Copy selection")
            .rect();
        assert_eq!(r.width(), size.height());
        assert_eq!(r.height(), size.height());
    }
}
