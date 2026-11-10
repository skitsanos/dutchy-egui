//! Surface colors, wrapping, focus, and application-level interactions.
#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{Accent, Alert, AlertVariant, Card, CardPadding, CardVariant, Mode, Theme};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

fn matrix(ui: &mut egui::Ui) {
    let theme = Theme::from_context(ui.ctx());
    ui.painter()
        .rect_filled(ui.max_rect(), 0, theme.palette.background);
    for pair in CardVariant::ALL.chunks(2) {
        ui.columns(2, |columns| {
            for (column, variant) in columns.iter_mut().zip(pair) {
                Card::new()
                    .variant(*variant)
                    .padding(CardPadding::Small)
                    .show(column, |ui| {
                        Card::title(ui, variant.name());
                        Card::description(ui, "Related information, with deliberate hierarchy.");
                        ui.label("Native label inherits the surface.");
                    });
            }
        });
        ui.add_space(8.0);
    }
    for &variant in AlertVariant::ALL {
        Alert::new(
            variant.name(),
            "Your work is retained. Review the next step before continuing.",
        )
        .variant(variant)
        .dismissible(true)
        .show(ui);
        ui.add_space(8.0);
    }
    Alert::new("banner", "A new component release is available.")
        .title("Library update")
        .banner(true)
        .action("Explore")
        .dismissible(true)
        .show(ui);
    ui.add_space(12.0);
    Card::new()
        .variant(CardVariant::Accent)
        .padding(CardPadding::Small)
        .show_interactive(ui, "action-card", "Open focused card", |ui| {
            Card::title(ui, "Interactive card");
            Card::description(
                ui,
                "Keyboard focus remains visible around the whole surface.",
            );
        });
    ui.add_space(12.0);
    Card::new()
        .variant(CardVariant::Primary)
        .padding(CardPadding::Small)
        .show_interactive(ui, "hover-card", "Open hovered card", |ui| {
            Card::title(ui, "Hover surface");
            Card::description(
                ui,
                "Interactive hover uses muted fill and the normal foreground.",
            );
        });
}

#[test]
fn surfaces_render_across_themes_scales_focus_and_hover() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "surfaces-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "surfaces-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "surfaces-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([760.0, 1360.0])
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(matrix);
        h.get_by_label("Open focused card").focus();
        h.get_by_label("Open hovered card").hover();
        h.run();
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn showcase_surface_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Cards, "cards"),
        (gallery::Page::Alerts, "alerts"),
    ] {
        for (size, suffix) in [([1200.0, 1500.0], "desktop"), ([480.0, 2200.0], "narrow")] {
            let mut h = Harness::builder()
                .with_size(size)
                .with_theme(egui::Theme::Light)
                .build_eframe(|cc| {
                    common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                    gallery::Showcase {
                        page,
                        reduced_motion: true,
                        ..Default::default()
                    }
                });
            h.run();
            h.check_accessibility();
            h.render()
                .unwrap()
                .save(format!("target/visual-proof/{prefix}-{suffix}.png"))
                .unwrap();
        }
    }
}

#[test]
fn example_card_and_alert_actions_update_only_host_owned_state() {
    let mut h = Harness::builder()
        .with_size([1200.0, 2200.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Cards,
                reduced_motion: true,
                ..Default::default()
            }
        });
    h.get_by_label("Pin project").click();
    h.run();
    assert_eq!((h.state().surfaces.pins, h.state().surfaces.opens), (1, 0));
    h.get_by_label("Open Project Phoenix").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(h.state().surfaces.opens, 1);
    h.get_by_label("Alerts").click();
    h.run();
    h.get_by_label("Retry connection").click();
    h.run();
    assert_eq!(h.state().surfaces.retries, 1);
    h.get_by_label("Dismiss Connection interrupted").focus();
    h.run();
    h.key_press(egui::Key::Space);
    h.run_steps(2);
    assert!(!h.state().surfaces.alert_visible);
    assert!(h.get_by_label("Restore connection alert").is_focused());
    h.get_by_label("Restore connection alert").click();
    h.run();
    assert!(h.state().surfaces.alert_visible);
    h.get_by_label("Dismiss New in the library").click();
    h.run_steps(2);
    assert!(!h.state().surfaces.banner_visible);
    assert!(h.get_by_label("Restore banner").is_focused());
}
