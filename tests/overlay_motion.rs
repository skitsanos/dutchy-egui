//! Drawer entrance scheduling and reduced motion; closed overlays stop rendering.
mod common;
use dutchy_egui::{Drawer, DrawerSide, Theme};
use egui::{Context, RawInput, Rect, ViewportId, pos2, vec2};
use std::time::Duration;
fn frame(ctx: &Context, time: f64, open: bool, side: DrawerSide) -> (Option<Rect>, Duration) {
    let mut rect = None;
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))),
            time: Some(time),
            ..Default::default()
        },
        |ui| {
            let mut open = open;
            rect = Drawer::new("moving", &mut open, "Details")
                .side(side)
                .show(ui.ctx(), |ui| {
                    ui.label("Drawer content");
                })
                .map(|r| r.response.rect);
        },
    );
    output.textures_delta.clear();
    (
        rect,
        output.viewport_output[&ViewportId::ROOT].repaint_delay,
    )
}
#[test]
fn entrance_moves_toward_each_edge_then_stops_scheduling() {
    for side in [DrawerSide::Left, DrawerSide::Right] {
        let ctx = Context::default();
        common::install(&ctx, Theme::default());
        frame(&ctx, 0.0, false, side);
        let first = frame(&ctx, 0.01, true, side).0.unwrap();
        let middle = frame(&ctx, 0.11, true, side).0.unwrap();
        let end = frame(&ctx, 0.6, true, side).0.unwrap();
        assert_ne!(first.left(), middle.left());
        assert_ne!(middle.left(), end.left());
        let expected = if side == DrawerSide::Left { 0.0 } else { 480.0 };
        assert!((end.left() - expected).abs() < 0.5, "{side:?}: {end:?}");
        let settled = frame(&ctx, 1.0, true, side);
        assert!(settled.1 > Duration::from_millis(100));
        frame(&ctx, 1.1, false, side);
        frame(&ctx, 1.2, false, side);
        for n in 0..5 {
            frame(&ctx, 1.3 + f64::from(n) / 60.0, false, side);
        }
        let closed = frame(&ctx, 1.5, false, side);
        assert!(closed.0.is_none());
        assert!(
            closed.1 > Duration::from_millis(100),
            "{side:?}: {:?}",
            closed.1
        );
    }
}
#[test]
fn reduced_motion_places_the_drawer_immediately() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(true));
    let first = frame(&ctx, 0.0, true, DrawerSide::Right).0.unwrap();
    let later = frame(&ctx, 0.2, true, DrawerSide::Right).0.unwrap();
    assert!((first.left() - 480.0).abs() < 0.5, "{first:?}");
    assert_eq!(first, later);
    for time in [0.3, 0.4, 0.5, 0.6] {
        frame(&ctx, time, true, DrawerSide::Right);
    }
    assert!(frame(&ctx, 0.8, true, DrawerSide::Right).1 > Duration::from_millis(100));
}
