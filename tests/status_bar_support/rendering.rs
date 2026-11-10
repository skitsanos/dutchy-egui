use Segment::{Focused, Hovered, Rest};
use dutchy_egui::{
    Accent::{Blue, Orange},
    Icon,
    Mode::{Dark, Light},
    StatusAlign, StatusItem, StatusTone, Theme,
};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

#[derive(Clone, Copy)]
enum Segment {
    Rest,
    Hovered,
    Focused,
}

fn items() -> Vec<StatusItem<'static, u32>> {
    vec![
        StatusItem::labeled(1, "Unread", "42"),
        StatusItem::text(2, "Synced")
            .icon(Icon::Check)
            .tone(StatusTone::Success),
        StatusItem::labeled(3, "Upload", "3 of 8").progress(37.5),
        StatusItem::labeled(4, "Sync", "Retry")
            .tone(StatusTone::Destructive)
            .clickable(true),
        StatusItem::text(5, "Paused").clickable(true).enabled(false),
        StatusItem::hint(6, "Ctrl+K", "Commands").align(StatusAlign::End),
    ]
}

#[test]
fn status_bars_render_rest_hover_and_focus_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, segment, name) in [
        (Orange, Light, 1.0, Rest, "light-1x"),
        (Orange, Dark, 1.5, Hovered, "dark-1_5x"),
        (Blue, Light, 2.0, Focused, "blue-2x"),
    ] {
        let mut h = Harness::builder()
            .with_size([760.0, 96.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_context_setup(move |ctx| {
                super::common::install(ctx, Theme::new(accent, mode).with_reduced_motion(true))
            })
            .build_ui_state(
                super::support::draw,
                super::support::Page {
                    width: 740.0,
                    items: items(),
                    ..Default::default()
                },
            );
        h.run_steps(3);
        let sync = h.get_by_role_and_label(Role::Button, "Sync");
        match segment {
            Rest => {}
            Hovered => {
                let center = sync.rect().center();
                h.hover_at(center);
            }
            Focused => sync.focus(),
        }
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(format!("status-bar-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
