//! Token geometry and command controls in light/dark at 1x, 1.5x and 2x.
#![cfg(feature = "bundled-fonts")]
mod common;
use dutchy_egui::{
    Accent, ButtonVariant, ControlSize, KeyRecorder, MenuItem, Mode, SplitButton, Text, Theme,
    egui::{self, Key, KeyboardShortcut, Modifiers},
};
use egui_kittest::{Harness, SnapshotResults};
#[test]
fn command_controls_render_across_themes_scales_and_disabled_states() {
    let mut snapshots = SnapshotResults::new();
    for (mode, scale, name) in [
        (Mode::Light, 1.0, "light-1x"),
        (Mode::Dark, 1.5, "dark-1_5x"),
        (Mode::Light, 2.0, "light-2x"),
    ] {
        let theme = Theme::new(Accent::Orange, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_os(egui::os::OperatingSystem::Mac)
            .with_size([640.0, 620.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| {
                common::install(ctx, theme);
            })
            .build_ui_state(
                |ui, state: &mut (bool, Option<KeyboardShortcut>)| {
                    ui.painter().rect_filled(
                        ui.ctx().content_rect(),
                        0,
                        Theme::from_context(ui.ctx()).palette.background,
                    );
                    ui.add(Text::heading("Commands in your hands."));
                    let alternatives = [
                        MenuItem::new(1, "PNG image"),
                        MenuItem::new(2, "SVG vector"),
                    ];
                    for (index, (variant, size)) in [
                        (ButtonVariant::Primary, ControlSize::Small),
                        (ButtonVariant::Outline, ControlSize::Medium),
                        (ButtonVariant::Secondary, ControlSize::Large),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        SplitButton::new(index, "Export artwork", &mut state.0, &alternatives)
                            .variant(variant)
                            .size(size)
                            .show(ui);
                    }
                    SplitButton::new("disabled", "Disabled export", &mut state.0, &alternatives)
                        .enabled(false)
                        .show(ui);
                    SplitButton::new("busy", "Preparing export", &mut state.0, &alternatives)
                        .loading(true)
                        .show(ui);
                    KeyRecorder::new("binding", &mut state.1)
                        .accessible_name("Record Save shortcut")
                        .show(ui);
                },
                (
                    false,
                    Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::S)),
                ),
            );
        h.run_steps(4);
        h.check_accessibility();
        h.snapshot(format!("command-controls-{name}"));
        snapshots.extend_harness(&mut h);
    }
    snapshots.unwrap();
}
