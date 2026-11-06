//! Pending images visibly animate, remain accessible, and stop scheduling on completion.
mod common;
#[path = "common/media.rs"]
mod media;
#[path = "../examples/showcase/gallery/media/pending.rs"]
mod pending;
use dutchy_egui::{Image, ImageAspect, Theme};
use egui::{Context, RawInput, Rect, ViewportId, accesskit::Role, pos2, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
fn draw(ctx: &Context, time: f64, hidden: bool) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            time: Some(time),
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(320.0, 200.0))),
            ..Default::default()
        },
        |ui| {
            if hidden {
                ui.add_space(400.0);
            }
            Image::new("slow", "showcase://pending", "Slow photograph")
                .width(240.0)
                .height(140.0)
                .expandable(true)
                .show(ui);
        },
    );
    output.textures_delta.clear();
    output
}
fn ctx(reduced: bool) -> Context {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(reduced));
    ctx.add_texture_loader(Arc::new(pending::Pending));
    ctx
}
#[test]
fn visible_pending_images_animate_and_reduced_motion_or_clipping_stops_frames() {
    for (reduced, hidden, animate) in [
        (false, false, true),
        (true, false, false),
        (false, true, false),
    ] {
        let ctx = ctx(reduced);
        for frame in 0..5 {
            draw(&ctx, f64::from(frame) / 60.0, hidden);
        }
        let first = draw(&ctx, 0.25, hidden);
        let later = draw(&ctx, 0.75, hidden);
        let delay = later.viewport_output[&ViewportId::ROOT].repaint_delay;
        if animate {
            assert_ne!(first.shapes, later.shapes);
            assert!(delay <= Duration::from_millis(16));
        } else {
            assert_eq!(first.shapes, later.shapes);
            assert!(delay > Duration::from_millis(100));
        }
    }
}
#[test]
fn named_busy_spinner_disappears_when_image_finishes_without_resizing_the_box() {
    let probe = Arc::new(Mutex::new(media::Probe {
        outcome: media::Outcome::Pending,
        ..Default::default()
    }));
    let fixture = probe.clone();
    let mut h = Harness::builder()
        .with_size([400.0, 300.0])
        .with_context_setup(move |ctx| {
            common::install(ctx, Theme::default().with_reduced_motion(true));
            media::install(ctx, fixture);
        })
        .build_ui(|ui| {
            Image::new("pending", "fixture://photo", "Remote photograph")
                .width(300.0)
                .aspect(ImageAspect::Video)
                .expandable(true)
                .show(ui);
        });
    h.run_steps(4);
    let name = "Loading Remote photograph";
    let indicator = h.get_by_role_and_label(Role::ProgressIndicator, name);
    assert!(indicator.accesskit_node().data().is_busy());
    let before = h
        .get_by_role_and_label(Role::Image, "Remote photograph")
        .rect();
    assert!(before.contains_rect(indicator.rect()));
    h.check_accessibility();
    probe.lock().unwrap().outcome = media::Outcome::Ready;
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::ProgressIndicator, name)
            .is_none()
    );
    assert_eq!(
        before,
        h.get_by_role_and_label(Role::Image, "Remote photograph")
            .rect()
    );
    assert!(
        !h.get_by_role_and_label(Role::Image, "Remote photograph")
            .accesskit_node()
            .data()
            .is_busy()
    );
    probe.lock().unwrap().outcome = media::Outcome::Failed;
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::ProgressIndicator, name)
            .is_none()
    );
    assert_eq!(
        before,
        h.get_by_role_and_label(Role::Image, "Remote photograph")
            .rect()
    );
}
#[test]
fn tiny_placeholders_clip_the_loading_paint() {
    let mut h = Harness::builder()
        .with_size([160.0, 160.0])
        .with_context_setup(|ctx| {
            common::install(ctx, Theme::default());
            ctx.add_texture_loader(Arc::new(pending::Pending));
        })
        .build_ui(|ui| {
            Image::new("tiny", "showcase://pending", "Tiny image")
                .width(24.0)
                .height(24.0)
                .show(ui);
        });
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::ProgressIndicator, "Loading Tiny image")
            .is_none()
    );
    assert_eq!(
        h.get_by_role_and_label(Role::Image, "Tiny image")
            .rect()
            .size(),
        vec2(24.0, 24.0)
    );
    h.check_accessibility();
}
