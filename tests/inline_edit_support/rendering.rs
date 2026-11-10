use super::support::*;
use dutchy_egui::{Accent, Mode, Theme};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

#[test]
fn hover_editing_and_compact_read_only_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, editing, compact, name) in [
        (Accent::Orange, Mode::Light, 1.0, false, false, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, true, false, "dark-1_5x"),
        (
            Accent::Blue,
            Mode::Light,
            2.0,
            true,
            true,
            "blue-2x-compact",
        ),
    ] {
        let mut page = Page {
            width: if compact { 120.0 } else { 400.0 },
            read_only: compact,
            ..Default::default()
        };
        if editing {
            page.edit.begin("Design notes.md");
        }
        let mut h = Harness::builder()
            .with_size([560.0, 320.0])
            .with_render_cursor(false)
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| {
                super::common::install(ctx, Theme::new(accent, mode).with_reduced_motion(true))
            })
            .build_ui_state(
                |ui, p| {
                    ui.painter().rect_filled(
                        ui.max_rect(),
                        0,
                        Theme::from_context(ui.ctx()).palette.background,
                    );
                    draw(ui, p);
                },
                page,
            );
        h.run_steps(3);
        if editing {
            h.get_by_label(NAME).focus();
        } else {
            h.get_by_role_and_label(Role::Button, "Rename Document name")
                .hover();
        }
        h.run_steps(2);
        h.check_accessibility();
        h.snapshot(format!("inline-edit-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
