//! Tooltips and layout composition across display scales and viewport widths.
#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{
    Accent, Button, ButtonVariant, Card, CardPadding, CardVariant, ColumnWidth, Container,
    ContainerWidth, Field, Flex, Input, Mode, ResponsiveGrid, Spacing, Text, TextColor, TextSize,
    Theme, Tooltip,
};
use egui::accesskit::Role;
use egui_kittest::{
    Harness, SnapshotResults,
    kittest::{NodeT, Queryable},
};

fn matrix(ui: &mut egui::Ui, name: &mut String) {
    let theme = Theme::from_context(ui.ctx());
    ui.painter()
        .rect_filled(ui.max_rect(), 0, theme.palette.background);
    ui.add(Text::heading("Space to think.").size(TextSize::Xl4));
    ui.add(Text::new("Token gaps / native layout / stable controls").color(TextColor::Muted));
    ui.add_space(16.0);
    Flex::row("toolbar")
        .gap(Spacing::Lg)
        .wrap(true)
        .show(ui, |ui| {
            for label in ["Projects", "People", "Activity"] {
                let _ = ui.add(Button::new(label).variant(ButtonVariant::Outline));
            }
        });
    ui.add_space(16.0);
    Container::new("settings")
        .max_width(ContainerWidth::Lg)
        .show(ui, |ui| {
            Card::new()
                .variant(CardVariant::Bordered)
                .padding(CardPadding::Small)
                .show(ui, |ui| {
                    Card::title(ui, "Centered settings");
                    Field::new("Workspace name")
                        .help("A visible instruction.")
                        .show(ui, |ui| ui.add(Input::new("name", name)));
                });
        });
    ui.add_space(24.0);
    ResponsiveGrid::new("tiles")
        .min_column_width(ColumnWidth::Compact)
        .gap(Spacing::Lg)
        .show(
            ui,
            &["Plan", "Build", "Ship"],
            |key| *key,
            |ui, key| {
                Card::new()
                    .variant(CardVariant::Muted)
                    .padding(CardPadding::Small)
                    .show(ui, |ui| {
                        Card::title(ui, key);
                        Card::description(ui, "Native column flow.");
                    });
            },
        );
    ui.add_space(64.0);
    let response = ui.add(Button::new("Keyboard help"));
    Tooltip::new("Clear guidance on hover or focus.").show(&response);
}

#[test]
fn layout_and_focused_tooltip_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "layout-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "layout-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "layout-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([760.0, 720.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(matrix, "Studio workspace".into());
        h.get_by_label("Keyboard help").focus();
        h.run_steps(3);
        h.check_accessibility();
        assert!(h.query_by_role(Role::Tooltip).is_some());
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn new_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Tooltips, "tooltips"),
        (gallery::Page::Layout, "layout"),
    ] {
        for (size, suffix) in [([1200.0, 1600.0], "desktop"), ([480.0, 2400.0], "narrow")] {
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
fn workspace_example_preserves_focus_on_reflow_and_saves_host_state() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1700.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Layout,
                reduced_motion: true,
                ..Default::default()
            }
        });
    h.get_by_label("Workspace name").focus();
    h.run_steps(2);
    let id = h.get_by_label("Workspace name").accesskit_node().id();
    h.state_mut().layout.compact = true;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Workspace name").accesskit_node().id(), id);
    assert!(h.get_by_label("Workspace name").is_focused());
    h.get_by_label("Save settings").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().layout.saves, 1);
    assert!(h.query_by_role_and_label(Role::Status, "Success").is_some());
    h.get_by_label("Tooltips").scroll_to_me();
    h.run_steps(3);
    h.get_by_label("Tooltips").click();
    h.run();
    h.get_by_label("Above").focus();
    h.run_steps(2);
    assert!(h.query_by_role(Role::Tooltip).is_some());
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().help.actions, 1);
}
