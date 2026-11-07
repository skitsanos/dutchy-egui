//! Focused rendering baselines and showcase smoke checks.

#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{
    Accent, Badge, BadgeVariant, Button, ButtonVariant, ControlSize, Mode, Text, TextColor,
    TextSize, Theme,
};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

fn state_matrix(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Sharp. Clear.").size(TextSize::Xl5));
    ui.add(Text::new("Inter / Body 16 / Warm neutrals / Zero radius"));
    ui.add_space(16.0);
    egui::Grid::new("button-states")
        .spacing([12.0, 12.0])
        .show(ui, |ui| {
            for variant in ButtonVariant::ALL {
                ui.add(
                    Button::new(variant.name())
                        .variant(variant)
                        .size(ControlSize::Small),
                );
                ui.add(
                    Button::new("Disabled")
                        .variant(variant)
                        .size(ControlSize::Small)
                        .enabled(false),
                );
                ui.add(
                    Button::new("Working")
                        .variant(variant)
                        .size(ControlSize::Small)
                        .loading(true),
                );
                ui.end_row();
            }
        });
    ui.add_space(16.0);
    ui.horizontal_wrapped(|ui| {
        for variant in BadgeVariant::ALL {
            ui.add(Badge::new(variant.name()).variant(variant));
        }
    });
    ui.add_space(16.0);
    ui.add(Text::label("Tab / Enter / Space").color(TextColor::Primary));
}

#[test]
fn states_render_across_accents_modes_and_display_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "orange-light-1x"),
        (Accent::Purple, Mode::Light, 1.5, "purple-light-1_5x"),
        (Accent::Crimson, Mode::Light, 2.0, "crimson-light-2x"),
        (Accent::Orange, Mode::Dark, 1.0, "orange-dark-1x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut harness = Harness::builder()
            .with_size([640.0, 600.0])
            .with_render_cursor(false)
            .with_theme(match mode {
                Mode::Light => egui::Theme::Light,
                Mode::Dark => egui::Theme::Dark,
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(state_matrix);
        harness.run();
        harness.snapshot(name);
        results.extend_harness(&mut harness);
    }
    results.unwrap();
}

#[test]
fn keyboard_focus_and_hover_render() {
    let mut harness = Harness::builder()
        .with_size([640.0, 600.0])
        .with_render_cursor(false)
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(state_matrix);
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Primary")
        .focus();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Outline")
        .hover();
    harness.run();
    harness.snapshot("focus-and-hover");
}

#[test]
fn showcase_is_usable_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [
        ([1200.0, 940.0], "showcase-desktop"),
        ([480.0, 900.0], "showcase-narrow"),
    ] {
        let mut harness = Harness::builder()
            .with_size(size)
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| {
                common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                gallery::Showcase {
                    reduced_motion: true,
                    ..Default::default()
                }
            });
        harness.run();
        harness.check_accessibility();
        harness
            .render()
            .unwrap()
            .save(format!("target/visual-proof/{name}.png"))
            .unwrap();
        assert!(harness.get_by_label("D. DUTCHY / EGUI").rect().width() < size[0]);
        assert!(
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Destructive")
                .rect()
                .right()
                <= size[0]
        );
    }
}

#[test]
fn showcase_navigation_and_theme_switches_update_the_host_state() {
    let mut harness = Harness::builder()
        .with_size([1200.0, 940.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                reduced_motion: true,
                ..Default::default()
            }
        });
    harness.get_by_label("Purple").click();
    harness.run();
    assert_eq!(harness.state().accent, Accent::Purple);
    harness.get_by_label("Dark mode").click();
    harness.run();
    assert_eq!(harness.state().mode, Mode::Dark);
    assert!(harness.ctx.global_style().visuals.dark_mode);
    // The component list now overflows the sidebar at this viewport height.
    harness.get_by_label("Badges").scroll_to_me();
    harness.run_steps(3);
    harness.get_by_label("Badges").click();
    harness.run();
    harness.get_by_label("Dismiss Removable tag").click();
    harness.run();
    assert!(!harness.state().badge_visible);
    harness.get_by_label("Restore tag").click();
    harness.run();
    assert!(harness.state().badge_visible);
}
