//! Animation scheduling respects clipping and reduced motion.

mod common;

use dutchy_egui::{Spinner, Theme};
use egui::{Context, RawInput, Rect, ViewportId, pos2, vec2};

fn repaint_delay(reduced: bool, hidden: bool) -> std::time::Duration {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(reduced));
    let mut delay = std::time::Duration::ZERO;
    for frame in 0..5 {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 200.0))),
                time: Some(f64::from(frame) / 60.0),
                ..Default::default()
            },
            |ui| {
                if hidden {
                    ui.add_space(400.0);
                }
                ui.add(Spinner::new());
            },
        );
        delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
        output.textures_delta.clear();
    }
    delay
}

#[test]
fn only_visible_animated_spinners_schedule_frames() {
    assert!(repaint_delay(false, false) <= std::time::Duration::from_millis(16));
    assert!(repaint_delay(true, false) > std::time::Duration::from_millis(100));
    assert!(repaint_delay(false, true) > std::time::Duration::from_millis(100));
}
