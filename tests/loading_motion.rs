//! Loading animation changes pixels only while visible and enabled by preferences.

mod common;
use dutchy_egui::{Progress, Skeleton, Theme};
use egui::{Context, RawInput, Rect, ViewportId, pos2, vec2};
use std::time::Duration;

#[derive(Clone, Copy)]
enum Indicator {
    Determinate,
    Unknown,
    Skeleton,
    EmptySkeleton,
    ZeroWidth,
}

fn draw(
    ctx: &Context,
    indicator: Indicator,
    time: f64,
    animate: bool,
    hidden: bool,
) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 200.0))),
            time: Some(time),
            ..Default::default()
        },
        |ui| {
            if hidden {
                ui.add_space(400.0);
            }
            match indicator {
                Indicator::Determinate => {
                    ui.add(Progress::new(50.0));
                }
                Indicator::Unknown => {
                    ui.add(Progress::indeterminate().animate(animate));
                }
                Indicator::Skeleton => {
                    ui.add(Skeleton::new().animate(animate));
                }
                Indicator::EmptySkeleton => {
                    ui.add(Skeleton::new().count(0));
                }
                Indicator::ZeroWidth => {
                    ui.add(Progress::indeterminate().width(0.0));
                }
            }
        },
    );
    output.textures_delta.clear();
    output
}

fn delay(indicator: Indicator, reduced: bool, animate: bool, hidden: bool) -> Duration {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(reduced));
    let mut delay = Duration::ZERO;
    for frame in 0..5 {
        delay = draw(&ctx, indicator, f64::from(frame) / 60.0, animate, hidden).viewport_output
            [&ViewportId::ROOT]
            .repaint_delay;
    }
    delay
}

#[test]
fn only_visible_animated_loading_indicators_schedule_frames() {
    for indicator in [Indicator::Unknown, Indicator::Skeleton] {
        assert!(delay(indicator, false, true, false) <= Duration::from_millis(16));
        assert!(delay(indicator, true, true, false) > Duration::from_millis(100));
        assert!(delay(indicator, false, false, false) > Duration::from_millis(100));
        assert!(delay(indicator, false, true, true) > Duration::from_millis(100));
    }
    for indicator in [
        Indicator::Determinate,
        Indicator::EmptySkeleton,
        Indicator::ZeroWidth,
    ] {
        assert!(delay(indicator, false, true, false) > Duration::from_millis(100));
    }
}

#[test]
fn loading_paint_changes_with_time_and_pausing_keeps_it_static() {
    for indicator in [Indicator::Unknown, Indicator::Skeleton] {
        let ctx = Context::default();
        common::install(&ctx, Theme::default());
        let first = draw(&ctx, indicator, 0.0, true, false).shapes;
        let later = draw(&ctx, indicator, 0.75, true, false).shapes;
        assert_ne!(first, later);
        let paused = draw(&ctx, indicator, 1.0, false, false).shapes;
        assert_eq!(paused, draw(&ctx, indicator, 2.0, false, false).shapes);
    }
}
