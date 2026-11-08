//! State, geometry, inverse surfaces and host-painted icons across display scales.
#![cfg(feature = "bundled-fonts")]
mod common;
use dutchy_egui::{
    Accent, ButtonVariant, Card, CardVariant, ControlSize, Icon, IconButton, Mode, Text, TextSize,
    Theme, ToggleButton, Toolbar, ToolbarItem,
};
use egui::{Color32, Painter, Rect, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

fn fixture(ui: &mut egui::Ui) {
    let p = Theme::from_context(ui.ctx()).palette;
    ui.painter().rect_filled(ui.max_rect(), 0, p.background);
    ui.add(Text::heading("Tools with purpose.").size(TextSize::Xl3));
    ui.horizontal_wrapped(|ui| {
        for icon in Icon::ALL {
            ui.add(IconButton::new(icon.name(), icon.name(), icon).tooltip(""));
        }
    });
    for variant in ButtonVariant::ALL {
        ui.horizontal(|ui| {
            ui.add(
                IconButton::new((variant.name(), "idle"), variant.name(), Icon::Plus)
                    .variant(variant),
            );
            ui.add(
                IconButton::new((variant.name(), "disabled"), "Unavailable", Icon::Trash)
                    .variant(variant)
                    .enabled(false),
            );
            ui.add(
                IconButton::new((variant.name(), "busy"), "Working", Icon::Save)
                    .variant(variant)
                    .loading(true),
            );
            ui.add(Text::label(variant.name()));
        });
    }
    ui.horizontal_wrapped(|ui| {
        for (i, size) in [ControlSize::Small, ControlSize::Medium, ControlSize::Large]
            .into_iter()
            .enumerate()
        {
            let mut on = true;
            let mut off = false;
            ui.add(
                ToggleButton::new((i, "on"), &mut on, "Bold")
                    .icon(Icon::Bold)
                    .size(size),
            );
            ui.add(
                ToggleButton::new((i, "off"), &mut off, "Italic")
                    .icon(Icon::Italic)
                    .size(size),
            );
        }
    });
    Card::new().variant(CardVariant::Dark).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add(
                IconButton::new("inverse-icon", "Export on inverse", Icon::Download).tooltip(""),
            );
            let mut off = false;
            ui.add(
                ToggleButton::new("inverse-toggle", &mut off, "Preview on inverse")
                    .icon(Icon::Play),
            );
        });
    });
    let items = [
        ToolbarItem::action(0u8, "Save document")
            .icon(Icon::Save)
            .tooltip(""),
        ToolbarItem::toggle(1, "Bold text", true)
            .icon(Icon::Bold)
            .separator_before(true)
            .tooltip(""),
        ToolbarItem::toggle(2, "Italic text", false)
            .icon(Icon::Italic)
            .tooltip(""),
        ToolbarItem::action(3, "Delete document")
            .icon(Icon::Trash)
            .enabled(false),
    ];
    Toolbar::new("format", &items)
        .accessible_name("Formatting")
        .show(ui);
}

#[test]
fn actions_render_across_themes_and_display_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "actions-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "actions-dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, "actions-blue-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([660.0, 810.0])
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(fixture);
        h.run();
        h.get_by_role_and_label(Role::Button, "Save document")
            .focus();
        h.get_by_role_and_label(Role::Button, "Italic text").hover();
        h.run();
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

fn oversized_icon(painter: &Painter, rect: Rect, ink: Color32) {
    painter.rect_filled(rect.expand(100.0), 0, ink);
}

#[test]
fn custom_icons_are_clipped_and_transparent_actions_inherit_inverse_ink() {
    let mut h = Harness::builder()
        .with_size([420.0, 240.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            Card::new().variant(CardVariant::Dark).show(ui, |ui| {
                ui.add(
                    IconButton::new("custom", "Custom tool", Icon::Custom(oversized_icon))
                        .tooltip(""),
                );
            });
        });
    h.run();
    let button = h.get_by_label("Custom tool").rect();
    let ink = Theme::default().palette.background;
    let custom = h
        .output()
        .shapes
        .iter()
        .find(|clipped| {
            matches!(&clipped.shape, egui::Shape::Rect(r)
                if r.fill == ink && r.rect.width() > 100.0 && r.rect.center() == button.center())
        })
        .expect("custom painter receives inverse ink");
    assert!(button.contains_rect(custom.clip_rect));
    assert_eq!(
        custom.clip_rect.width(),
        dutchy_egui::tokens::actions::ICON_MEDIUM
    );
}

#[test]
fn hover_changes_fill_and_disabled_toggles_keep_their_pressed_surface() {
    let mut h = Harness::builder()
        .with_size([400.0, 220.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            ui.add(IconButton::new("add", "Add document", Icon::Plus).tooltip(""));
            let mut on = true;
            let mut off = false;
            ui.add(ToggleButton::new("on", &mut on, "Selected").enabled(false));
            ui.add(ToggleButton::new("off", &mut off, "Unselected").enabled(false));
        });
    h.run();
    let fill = |h: &Harness<'_, ()>, label: &str| {
        let target = h.get_by_label(label).rect();
        h.output()
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Rect(r) if r.rect == target => Some(r.fill),
                _ => None,
            })
            .unwrap()
    };
    let before = fill(&h, "Add document");
    h.get_by_label("Add document").hover();
    h.run();
    assert_ne!(fill(&h, "Add document"), before);
    assert_ne!(fill(&h, "Selected"), fill(&h, "Unselected"));
}

fn surface(h: &Harness<'_, ()>, label: &str) -> egui::epaint::RectShape {
    let target = h.get_by_label(label).rect();
    h.output()
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Rect(r) if r.rect == target => Some(r.clone()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn toggle_hover_keeps_pressed_and_unpressed_surfaces_distinct() {
    let mut h = Harness::builder()
        .with_size([400.0, 220.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            let mut on = true;
            let mut off = false;
            ui.add(ToggleButton::new("on", &mut on, "Pressed"));
            ui.add(ToggleButton::new("off", &mut off, "Unpressed"));
        });
    h.run();
    let p = Theme::default().palette;
    let pressed_rest = surface(&h, "Pressed").fill;
    h.get_by_label("Unpressed").hover();
    h.run();
    let hovered_off = surface(&h, "Unpressed");
    assert_ne!(
        hovered_off.fill, pressed_rest,
        "a hovered off-state must not paint the pressed surface"
    );
    assert_eq!(hovered_off.fill, p.muted);
    assert_eq!(
        hovered_off.stroke.color, p.foreground,
        "outline toggles keep their stroke while hovered"
    );
    h.get_by_label("Pressed").hover();
    h.run();
    let hovered_on = surface(&h, "Pressed").fill;
    assert_ne!(
        hovered_on, p.primary,
        "hovering a pressed toggle must not switch to the accent"
    );
    assert_eq!(hovered_on, p.foreground.gamma_multiply(0.9));
}
