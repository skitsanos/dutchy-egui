//! Sharp drawing surfaces and fixed ink across themes, DPI and constrained widths.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/sketch/sample.rs"]
mod sample;
use dutchy_egui::{Accent, Mode, Scribble, ScribbleDrawing, Theme};
use egui::accesskit::Role;
use egui_kittest::{
    Harness, SnapshotResults,
    kittest::{By, Queryable},
};
fn harness(
    theme: Theme,
    scale: f32,
    width: f32,
) -> Harness<'static, (ScribbleDrawing, ScribbleDrawing)> {
    Harness::builder()
        .with_size([width, 1700.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(if theme.mode == Mode::Dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        })
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(
            |ui, (empty, drawing)| {
                Scribble::new("empty", empty)
                    .label("Empty drawing")
                    .show(ui);
                ui.add_space(24.0);
                Scribble::new("sample", drawing)
                    .label("Sample drawing")
                    .show(ui);
                ui.add_space(24.0);
                Scribble::new("read-only", drawing)
                    .label("Read-only drawing")
                    .read_only(true)
                    .show(ui);
                ui.add_space(24.0);
                Scribble::new("disabled", drawing)
                    .label("Disabled drawing")
                    .enabled(false)
                    .controls(false)
                    .show(ui);
            },
            (ScribbleDrawing::default(), sample::drawing()),
        )
}
#[test]
fn canvas_ink_focus_and_states_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "purple-2x"),
    ] {
        let mut h = harness(Theme::new(accent, mode), scale, 740.0);
        h.run_steps(4);
        h.get_by_role_and_label(Role::Canvas, "Sample drawing")
            .focus();
        h.run_steps(2);
        h.check_accessibility();
        let feature = if cfg!(feature = "png-export") {
            "-png"
        } else {
            ""
        };
        h.snapshot(format!("scribble-{name}{feature}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
#[test]
fn narrow_canvases_and_wrapped_controls_stay_inside_the_column() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for width in [280.0, 160.0] {
        let mut h = harness(Theme::default(), 1.0, width);
        h.run_steps(4);
        for role in [Role::Canvas, Role::Button] {
            for node in h.query_all(By::new().role(role)) {
                assert!(
                    node.rect().left() >= 7.5 && node.rect().right() <= width - 7.5,
                    "{width}: {:?}",
                    node.rect()
                );
            }
        }
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/scribble-{width}.png"))
            .unwrap();
    }
}
