use Divider::{Focused, Hovered, Rest};
use dutchy_egui::{
    Accent::{Blue, Orange},
    Mode::{Dark, Light},
    Theme,
};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

#[derive(Clone, Copy)]
enum Divider {
    Rest,
    Hovered,
    Focused,
}

#[test]
fn split_panes_render_rest_hover_and_focus_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, vertical, divider, name) in [
        (Orange, Light, 1.0, false, Rest, "light-1x"),
        (Orange, Dark, 1.5, false, Hovered, "dark-1_5x"),
        (Blue, Light, 2.0, true, Focused, "blue-2x"),
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
        let splitter = h.get_by_role_and_label(Role::Splitter, "Folders");
        match divider {
            Divider::Rest => {}
            Divider::Hovered => {
                let center = splitter.rect().center();
                h.hover_at(center);
            }
            Divider::Focused => splitter.focus(),
        }
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(format!("split-pane-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
