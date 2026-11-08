//! Native flat-color controls across themes, scales and constrained layouts.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{Accent, ColorPicker, ControlSize, Field, Mode, Text, TextSize, Theme};
use egui::{Color32, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

fn fixture(ui: &mut egui::Ui, value: &mut Color32) {
    let p = Theme::from_context(ui.ctx()).palette;
    ui.painter().rect_filled(ui.max_rect(), 0, p.background);
    ui.add(Text::heading("Color as data.").size(TextSize::Xl3));
    Field::new("Accent").show(ui, |ui| {
        ui.add(
            ColorPicker::new("accent", value)
                .alpha(true)
                .accessible_name("Accent"),
        )
    });
}

#[test]
fn picker_renders_flat_surfaces_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "color-picker-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "color-picker-dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, "color-picker-blue-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([480.0, 720.0])
            .with_theme(match mode {
                Mode::Light => egui::Theme::Light,
                Mode::Dark => egui::Theme::Dark,
            })
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(fixture, Color32::from_rgba_unmultiplied(249, 116, 21, 128));
        h.run();
        h.get_by_role_and_label(Role::ColorWell, "Accent").click();
        h.run_steps(3);
        h.check_accessibility();
        for clipped in &h.output().shapes {
            if let egui::Shape::Rect(rect) = &clipped.shape {
                assert_eq!(rect.corner_radius, egui::CornerRadius::ZERO);
                assert_eq!(rect.blur_width, 0.0);
            }
        }
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn narrow_popup_stays_in_the_viewport_and_reveals_keyboard_actions() {
    let mut h = Harness::builder()
        .with_size([240.0, 420.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(fixture, Color32::from_rgb(20, 80, 160));
    h.run();
    h.get_by_role_and_label(Role::ColorWell, "Accent").click();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Dialog, "Edit Accent");
    let popup = h
        .output()
        .shapes
        .iter()
        .find_map(|shape| {
            if let egui::Shape::Rect(rect) = &shape.shape
                && rect.stroke.width == dutchy_egui::tokens::border::STANDARD
                && rect.rect.height() > ControlSize::Large.height()
            {
                Some(rect.rect)
            } else {
                None
            }
        })
        .expect("the popup paints its sharp frame");
    assert!(
        popup.left() >= 0.0
            && popup.right() <= 240.0
            && popup.top() >= 0.0
            && popup.bottom() <= 420.0,
        "{popup:?}"
    );
    h.get_by_label("Done").focus();
    h.run_steps(3);
    let done = h.get_by_label("Done").rect();
    assert!(
        done.top() >= popup.top() && done.bottom() <= popup.bottom(),
        "{done:?}, {popup:?}"
    );
    std::fs::create_dir_all("target/color-picker-proof").unwrap();
    h.render()
        .unwrap()
        .save("target/color-picker-proof/narrow.png")
        .unwrap();
    h.get_by_label("Done").click();
    h.run_steps(2);
    assert!(h.query_by_label("Hex color").is_none());
    assert!(
        h.get_by_role_and_label(Role::ColorWell, "Accent")
            .is_focused()
    );
}

#[test]
fn showcase_and_all_trigger_sizes_render_with_accessible_names() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1400.0])
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Colors,
                reduced_motion: true,
                ..Default::default()
            }
        });
    h.run();
    h.check_accessibility();
    std::fs::create_dir_all("target/color-picker-proof").unwrap();
    h.render()
        .unwrap()
        .save("target/color-picker-proof/showcase.png")
        .unwrap();
    for (name, size) in [
        ("Small", ControlSize::Small),
        ("Medium", ControlSize::Medium),
        ("Large", ControlSize::Large),
    ] {
        let rect = h.get_by_role_and_label(Role::ColorWell, name).rect();
        assert_eq!(rect.height(), size.height());
        let color = Theme::default().palette.primary;
        let preview = h
            .output()
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Rect(shape) = &shape.shape
                    && shape.fill == color
                    && rect.contains_rect(shape.rect)
                {
                    Some(shape.rect)
                } else {
                    None
                }
            })
            .expect("the color preview is painted inside its trigger");
        assert!((preview.center().y - rect.center().y).abs() < 0.1);
    }
}
