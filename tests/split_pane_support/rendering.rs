use dutchy_egui::{Accent, Mode, Theme};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
#[test]
fn split_panes_render_both_axes_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, vertical, name) in [
        (Accent::Orange, Mode::Light, 1.0, false, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, false, "dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, true, "blue-2x"),
    ] {
        let mut h = Harness::builder()
            .with_size([600.0, 420.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_context_setup(move |ctx| {
                super::common::install(ctx, Theme::new(accent, mode).with_reduced_motion(true))
            })
            .build_ui_state(
                super::support::draw,
                super::support::Page {
                    vertical,
                    first: 120.0,
                    ..Default::default()
                },
            );
        h.run_steps(3);
        h.get_by_role_and_label(Role::Splitter, "Folders").focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(format!("split-pane-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
