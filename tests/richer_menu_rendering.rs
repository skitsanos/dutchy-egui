//! Whole command surfaces, nested placement and sharp geometry at three scales.
#![cfg(feature = "bundled-fonts")]
#[path = "../examples/showcase/gallery/command_surfaces.rs"]
mod command_surfaces;
mod common;
use dutchy_egui::{Accent, Button, ControlSize, Dropdown, MenuItem, MenuWidth, Mode, Text, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

#[test]
fn showcase_shares_checked_values_between_view_menu_and_toolbar() {
    let mut h = Harness::builder()
        .with_size([1000.0, 650.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            command_surfaces::show,
            command_surfaces::CommandState::default(),
        );
    h.run();
    h.get_by_role_and_label(Role::MenuItem, "View").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItemCheckBox, "Show grid")
        .click();
    h.run_steps(3);
    assert!(!h.state().grid);
    h.get_by_role_and_label(Role::Button, "Show grid").click();
    h.run_steps(3);
    assert!(h.state().grid);
    h.get_by_role_and_label(Role::Switch, "Apply view changes")
        .click();
    h.run_steps(3);
    h.get_by_role_and_label(Role::MenuItem, "View").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItemCheckBox, "Show grid")
        .click();
    h.run_steps(3);
    assert!(h.state().grid);
    assert!(
        common::galleys(&h.output().shapes)
            .iter()
            .any(|g| g.job.text.contains("View change declined"))
    );
    h.check_accessibility();
}

#[test]
fn application_menus_and_nested_exports_render_at_three_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, "blue-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([1100.0, 660.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(
                |ui, state: &mut command_surfaces::CommandState| {
                    ui.painter().rect_filled(
                        ui.max_rect(),
                        0,
                        Theme::from_context(ui.ctx()).palette.background,
                    );
                    ui.add(Text::heading("Commands with depth."));
                    command_surfaces::show(ui, state);
                },
                command_surfaces::CommandState::default(),
            );
        h.run();
        h.get_by_role_and_label(Role::MenuItem, "File").click();
        h.run_steps(4);
        h.check_accessibility();
        h.snapshot(format!("rich-menus-{name}-root"));
        h.get_by_role_and_label(Role::MenuItem, "Export artwork")
            .focus();
        h.run();
        h.key_press(Key::ArrowRight);
        h.run_steps(4);
        h.key_press(Key::ArrowRight);
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "PNG image")
                .is_focused()
        );
        h.check_accessibility();
        h.snapshot(format!("rich-menus-{name}-nested"));
        h.get_by_role_and_label(Role::MenuItem, "View").focus();
        h.run();
        h.key_press(Key::ArrowDown);
        h.run_steps(4);
        h.check_accessibility();
        h.snapshot(format!("rich-menus-{name}-checks"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn submenus_flip_and_stay_inside_a_narrow_viewport() {
    let mut h = Harness::builder()
        .with_size([240.0, 220.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            let children = [MenuItem::new(
                2,
                "A wrapped child command with a long accessible name",
            )];
            let items = [MenuItem::new(1, "Formats").submenu(&children)];
            let mut open = true;
            Dropdown::new("edge", &mut open, &items)
                .width(MenuWidth::Large)
                .accessible_name("Edge menu")
                .show(ui, |ui| {
                    ui.add(Button::new("Edge").size(ControlSize::Small))
                });
        });
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Formats").focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    for name in ["Edge menu", "Formats"] {
        let rect = h.get_by_role_and_label(Role::Menu, name).rect();
        assert!(
            rect.left() >= -1.0
                && rect.top() >= -1.0
                && rect.right() <= 241.0
                && rect.bottom() <= 221.0,
            "{name}: {rect:?}"
        );
    }
    h.check_accessibility();
}
