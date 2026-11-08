//! Color editing, host ownership and popup lifecycle through the public API.
mod common;

use dutchy_egui::{Button, ColorPicker, ColorPreset, ControlSize, Theme};
use egui::{Color32, Key, Modifiers, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    color: Color32,
    enabled: bool,
    parent_enabled: bool,
    read_only: bool,
    alpha: bool,
    prefix: bool,
    covered: bool,
    changed: bool,
    actions: usize,
}

fn harness(alpha: bool) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([560.0, 700.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.prefix {
                    ui.label("Inserted sibling");
                }
                ui.add_enabled_ui(state.parent_enabled, |ui| {
                    let presets = [
                        ColorPreset::new("Ocean", Color32::from_rgb(20, 80, 160)),
                        ColorPreset::new("Paper", Color32::WHITE),
                    ];
                    state.changed = ui
                        .add(
                            ColorPicker::new("color", &mut state.color)
                                .alpha(state.alpha)
                                .enabled(state.enabled)
                                .read_only(state.read_only)
                                .palette(&presets)
                                .accessible_name("Accent"),
                        )
                        .changed();
                });
                ui.add_space(500.0);
                if ui
                    .add(Button::new("After").size(ControlSize::Small))
                    .clicked()
                {
                    state.actions += 1;
                }
                if state.covered {
                    egui::Modal::new(egui::Id::unique("cover-color")).show(ui.ctx(), |ui| {
                        ui.label("Covering modal");
                    });
                }
            },
            State {
                color: Color32::from_rgb(30, 60, 90),
                enabled: true,
                parent_enabled: true,
                read_only: false,
                alpha,
                prefix: false,
                covered: false,
                changed: false,
                actions: 0,
            },
        );
    h.run();
    h
}

fn open(h: &mut Harness<'_, State>) {
    h.get_by_role_and_label(Role::ColorWell, "Accent").click();
    h.run_steps(3);
    h.get_by_label("Hex color");
}

fn replace(h: &mut Harness<'_, State>, label: &str, text: &str) {
    h.get_by_label(label).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_label(label).type_text(text);
    h.run_steps(1);
}

#[test]
fn named_color_well_exposes_host_value_and_opens_with_keyboard() {
    let mut h = harness(false);
    let well = h.get_by_role_and_label(Role::ColorWell, "Accent");
    let node = well.accesskit_node();
    assert_eq!(
        node.data().color_value(),
        Some(egui::accesskit::Color {
            red: 30,
            green: 60,
            blue: 90,
            alpha: 255
        })
    );
    assert_eq!(node.value().as_deref(), Some("#1E3C5A"));
    well.focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(h.get_by_label("Hex color").is_focused());
    assert!(!h.state().changed);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label("Hex color").is_none());
    assert!(
        h.get_by_role_and_label(Role::ColorWell, "Accent")
            .is_focused()
    );
}

#[test]
fn hex_commits_valid_input_and_invalid_drafts_revert_without_a_host_change() {
    let mut h = harness(false);
    open(&mut h);
    replace(&mut h, "Hex color", "#f81");
    assert_eq!(h.state().color, Color32::from_rgb(255, 136, 17));
    assert!(h.state().changed);
    replace(&mut h, "Hex color", "#gé1234");
    assert_eq!(h.state().color, Color32::from_rgb(255, 136, 17));
    assert!(!h.state().changed);
    assert!(
        h.get_by_label("Hex color")
            .accesskit_node()
            .data()
            .invalid()
            .is_some()
    );
    h.get_by_label("Done").click();
    h.run_steps(2);
    open(&mut h);
    assert_eq!(
        h.get_by_label("Hex color")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("#FF8811")
    );
}

#[test]
fn palette_slider_and_step_buttons_all_commit_to_the_same_host_value() {
    let mut h = harness(false);
    open(&mut h);
    h.get_by_role_and_label(Role::Button, "Ocean").click();
    h.run_steps(1);
    assert_eq!(h.state().color, Color32::from_rgb(20, 80, 160));
    assert!(h.state().changed);
    h.get_by_role_and_label(Role::Slider, "Red").focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run_steps(1);
    assert_eq!(h.state().color.r(), 21);
    h.get_by_label("Increase Red value").click();
    h.run_steps(1);
    assert_eq!(h.state().color.r(), 22);
    h.run_steps(1);
    assert_eq!(
        h.get_by_label("Hex color")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("#1650A0")
    );
}

#[test]
fn alpha_zero_keeps_editor_rgb_until_opacity_is_restored() {
    let mut h = harness(true);
    open(&mut h);
    replace(&mut h, "Hex color", "#FF000000");
    assert_eq!(h.state().color, Color32::TRANSPARENT);
    h.get_by_label("Done").click();
    h.run_steps(2);
    open(&mut h);
    assert_eq!(
        h.get_by_label("Hex color")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("#FF000000")
    );
    h.get_by_label("Increase Alpha value").click();
    h.run_steps(1);
    assert_eq!(
        h.state().color,
        Color32::from_rgba_unmultiplied(255, 0, 0, 1)
    );
}

#[test]
fn rgb_mode_preserves_alpha_and_rgba_mode_accepts_alpha_last() {
    let mut h = harness(false);
    h.state_mut().color = Color32::from_rgba_unmultiplied(30, 60, 90, 80);
    h.run();
    open(&mut h);
    replace(&mut h, "Hex color", "#ABCDEF");
    assert_eq!(h.state().color.a(), 80);
    assert_eq!(
        h.state().color,
        Color32::from_rgba_unmultiplied(171, 205, 239, 80)
    );
    h.state_mut().alpha = true;
    h.run();
    replace(&mut h, "Hex color", "#1234");
    assert_eq!(
        h.state().color,
        Color32::from_rgba_unmultiplied(17, 34, 51, 68)
    );
    replace(&mut h, "Hex color", "#112233");
    assert_eq!(h.state().color, Color32::from_rgb(17, 34, 51));
}

#[test]
fn disabled_picker_closes_and_cannot_edit_during_focus_loss() {
    for parent in [false, true] {
        let mut h = harness(true);
        open(&mut h);
        h.get_by_role_and_label(Role::Slider, "Red").focus();
        h.run();
        if parent {
            h.state_mut().parent_enabled = false;
        } else {
            h.state_mut().enabled = false;
        }
        h.key_press(Key::ArrowRight);
        h.run_steps(2);
        assert_eq!(h.state().color, Color32::from_rgb(30, 60, 90));
        assert!(h.query_by_label("Hex color").is_none());
        assert!(
            h.get_by_role_and_label(Role::ColorWell, "Accent")
                .accesskit_node()
                .is_disabled()
        );
    }
}

#[test]
fn read_only_allows_inspection_but_rejects_edits() {
    let mut h = harness(true);
    h.state_mut().read_only = true;
    h.run();
    open(&mut h);
    replace(&mut h, "Hex color", "#FFFFFF00");
    assert_eq!(h.state().color, Color32::from_rgb(30, 60, 90));
    assert!(h.get_by_label("Hex color").accesskit_node().is_read_only());
    assert!(
        h.get_by_role_and_label(Role::Button, "Ocean")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        h.get_by_role_and_label(Role::Slider, "Red")
            .accesskit_node()
            .is_disabled()
    );
    assert!(!h.get_by_label("Copy hex").accesskit_node().is_disabled());
}

#[test]
fn focus_survives_inserted_siblings_and_external_host_updates_win() {
    let mut h = harness(true);
    open(&mut h);
    h.state_mut().prefix = true;
    h.run();
    assert!(h.get_by_label("Hex color").is_focused());
    h.state_mut().color = Color32::from_rgb(12, 34, 56);
    h.run();
    assert_eq!(
        h.get_by_label("Hex color")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("#0C2238FF")
    );
    h.get_by_role_and_label(Role::Slider, "Green").focus();
    h.run();
    h.state_mut().prefix = false;
    h.run();
    assert!(h.get_by_role_and_label(Role::Slider, "Green").is_focused());
}

#[test]
fn outside_click_runs_the_host_action_once_and_modals_block_the_editor() {
    let mut h = harness(false);
    open(&mut h);
    h.get_by_label("After").click();
    h.run_steps(3);
    assert_eq!(h.state().actions, 1);
    assert!(h.query_by_label("Hex color").is_none());
    open(&mut h);
    h.state_mut().covered = true;
    h.run_steps(2);
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(h.state().color, Color32::from_rgb(30, 60, 90));
    assert!(h.query_by_label("Hex color").is_none());
}

#[test]
fn copy_uses_the_committed_color_instead_of_an_invalid_draft() {
    let mut h = harness(true);
    open(&mut h);
    replace(&mut h, "Hex color", "not a color");
    h.get_by_label("Copy hex").click();
    h.run_steps(1);
    assert!(h.output().platform_output.commands.iter().any(|command| {
        matches!(command, egui::OutputCommand::CopyText(text) if text == "#1E3C5AFF")
    }));
    assert_eq!(h.state().color, Color32::from_rgb(30, 60, 90));
}

#[test]
fn inspection_does_not_reencode_a_premultiplied_host_color() {
    let mut h = harness(true);
    let supplied = Color32::from_rgba_premultiplied(10, 20, 30, 0);
    h.state_mut().color = supplied;
    h.state_mut().read_only = true;
    h.run_steps(2);
    open(&mut h);
    assert_eq!(h.state().color, supplied);
    assert!(!h.state().changed);
}
