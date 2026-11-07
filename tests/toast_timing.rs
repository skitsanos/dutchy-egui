//! Visible lifetime, paused intervals, modal suspension and bounded scheduling.
mod common;
use dutchy_egui::{Theme, Toast, ToastDismissal, ToastEvent, ToastQueue, Toasts};
use egui::{Context, RawInput, Rect, ViewportId, pos2, vec2};
use std::time::Duration;
struct Options {
    paused: bool,
    focused: bool,
    modal: bool,
    pointer: Option<egui::Pos2>,
    size: egui::Vec2,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            paused: false,
            focused: true,
            modal: false,
            pointer: None,
            size: vec2(800.0, 700.0),
        }
    }
}
#[derive(Default)]
struct Read {
    rect: Option<Rect>,
    close: Option<egui::Id>,
    events: Vec<ToastEvent<()>>,
    delay: Duration,
    shown: bool,
    shapes: Vec<egui::epaint::ClippedShape>,
}
fn frame(ctx: &Context, queue: &mut ToastQueue, time: f64, options: &Options) -> Read {
    let mut read = Read::default();
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), options.size)),
            time: Some(time),
            focused: options.focused,
            events: options
                .pointer
                .map(|pos| vec![egui::Event::PointerMoved(pos)])
                .unwrap_or_default(),
            ..Default::default()
        },
        |ui| {
            ui.label("Host content");
            if options.modal {
                egui::Modal::new(egui::Id::unique("blocking")).show(ui.ctx(), |ui| {
                    ui.label("Modal task");
                });
            }
            let result = Toasts::new("clock", queue)
                .max_visible(1)
                .paused(options.paused)
                .show(ui.ctx());
            read.shown = result.response.is_some();
            read.rect = result.visible.first().map(|row| row.response.rect);
            read.close = result
                .visible
                .first()
                .and_then(|row| row.close.as_ref().map(|r| r.id));
            read.events.extend(result.events);
        },
    );
    output.textures_delta.clear();
    read.delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
    read.shapes = output.shapes;
    read
}
fn settle(ctx: &Context, queue: &mut ToastQueue, time: f64, options: &Options) -> Read {
    for i in 0..4 {
        frame(ctx, queue, time + f64::from(i) * 0.01, options);
    }
    frame(ctx, queue, time + 0.05, options)
}
#[test]
fn expiry_removes_once_and_pending_entries_receive_their_own_visible_lifetime() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(true));
    let mut queue = ToastQueue::new();
    let a = queue.push(Toast::new("First"));
    let b = queue.push(Toast::new("Second"));
    let options = Options::default();
    settle(&ctx, &mut queue, 0.0, &options);
    let expired = frame(&ctx, &mut queue, 5.2, &options);
    assert_eq!(
        expired.events,
        [ToastEvent::Dismissed {
            id: a,
            reason: ToastDismissal::Expired
        }]
    );
    assert_eq!(queue.len(), 1);
    assert!(frame(&ctx, &mut queue, 100.0, &options).events.is_empty());
    assert_eq!(queue.len(), 1);
    let expired = frame(&ctx, &mut queue, 105.2, &options);
    assert_eq!(
        expired.events,
        [ToastEvent::Dismissed {
            id: b,
            reason: ToastDismissal::Expired
        }]
    );
    assert!(queue.is_empty());
    assert!(frame(&ctx, &mut queue, 110.0, &options).events.is_empty());
}
#[test]
fn explicit_pause_hover_and_keyboard_focus_preserve_remaining_time() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(true));
    let mut queue = ToastQueue::new();
    queue.push(Toast::new("Read this"));
    let mut options = Options::default();
    let initial = settle(&ctx, &mut queue, 0.0, &options);
    options.pointer = Some(initial.rect.unwrap().center());
    frame(&ctx, &mut queue, 1.0, &options);
    frame(&ctx, &mut queue, 100.0, &options);
    assert_eq!(queue.len(), 1);
    options.pointer = Some(pos2(-100.0, -100.0));
    options.paused = true;
    frame(&ctx, &mut queue, 101.0, &options);
    frame(&ctx, &mut queue, 200.0, &options);
    assert_eq!(queue.len(), 1);
    options.paused = false;
    ctx.memory_mut(|m| m.request_focus(initial.close.unwrap()));
    frame(&ctx, &mut queue, 201.0, &options);
    frame(&ctx, &mut queue, 300.0, &options);
    assert_eq!(queue.len(), 1);
    ctx.memory_mut(|m| m.surrender_focus(initial.close.unwrap()));
    frame(&ctx, &mut queue, 301.0, &options);
    assert!(frame(&ctx, &mut queue, 303.0, &options).events.is_empty());
    assert_eq!(frame(&ctx, &mut queue, 307.0, &options).events.len(), 1);
}
#[test]
fn inactive_and_modal_windows_suspend_rendering_without_consuming_lifetime() {
    for modal in [false, true] {
        let ctx = Context::default();
        common::install(&ctx, Theme::default().with_reduced_motion(true));
        let mut queue = ToastQueue::new();
        queue.push(Toast::new("Wait for attention"));
        let mut options = Options::default();
        settle(&ctx, &mut queue, 0.0, &options);
        if modal {
            options.modal = true;
        } else {
            options.focused = false;
        }
        settle(&ctx, &mut queue, 0.1, &options);
        assert!(!frame(&ctx, &mut queue, 100.0, &options).shown);
        assert_eq!(queue.len(), 1);
        options.modal = false;
        options.focused = true;
        settle(&ctx, &mut queue, 101.0, &options);
        assert!(frame(&ctx, &mut queue, 103.0, &options).events.is_empty());
        assert_eq!(frame(&ctx, &mut queue, 107.0, &options).events.len(), 1);
    }
}
#[test]
fn clipped_notifications_pause_and_persistent_notifications_do_not_schedule_expiry() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(true));
    let mut queue = ToastQueue::new();
    queue.push(Toast::new("A long readable message. ".repeat(40)));
    let options = Options {
        size: vec2(280.0, 300.0),
        ..Default::default()
    };
    settle(&ctx, &mut queue, 0.0, &options);
    frame(&ctx, &mut queue, 100.0, &options);
    assert_eq!(queue.len(), 1);
    queue.clear();
    queue.push(Toast::new("Persistent").timeout(None));
    let options = Options::default();
    settle(&ctx, &mut queue, 101.0, &options);
    assert!(frame(&ctx, &mut queue, 105.0, &options).delay > Duration::from_millis(100));
    queue.clear();
    queue.push(Toast::new("Maximum duration").timeout(Some(Duration::MAX)));
    settle(&ctx, &mut queue, 106.0, &options);
    assert!(frame(&ctx, &mut queue, 110.0, &options).events.is_empty());
}
#[test]
fn entrance_changes_paint_then_settles_and_reduced_motion_is_immediate() {
    for reduced in [false, true] {
        let ctx = Context::default();
        common::install(&ctx, Theme::default().with_reduced_motion(reduced));
        let mut queue = ToastQueue::new();
        queue.push(Toast::new("Arriving").timeout(None));
        let options = Options::default();
        frame(&ctx, &mut queue, 0.0, &options);
        let first = frame(&ctx, &mut queue, 0.01, &options);
        let later = frame(&ctx, &mut queue, 0.2, &options);
        if reduced {
            assert_eq!(first.shapes, later.shapes);
        } else {
            assert_ne!(first.shapes, later.shapes);
            assert!(later.delay <= Duration::from_millis(16));
        }
        settle(&ctx, &mut queue, 1.0, &options);
        assert!(frame(&ctx, &mut queue, 1.2, &options).delay > Duration::from_millis(100));
    }
}

/// Alpha of the most opaque rectangle painted inside the toast's rect.
fn surface_alpha(read: &Read) -> u8 {
    let rect = read.rect.expect("toast is shown");
    read.shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if rect.contains_rect(shape.rect) => Some(shape.fill.a()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}
#[test]
fn entrance_fade_survives_a_clock_that_stalls_or_runs_backwards() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default());
    let mut queue = ToastQueue::new();
    queue.push(Toast::new("Saved"));
    let options = Options::default();
    frame(&ctx, &mut queue, 100.0, &options);
    frame(&ctx, &mut queue, 100.01, &options);
    let faded_in = frame(&ctx, &mut queue, 101.0, &options);
    assert_eq!(
        surface_alpha(&faded_in),
        255,
        "fade completes after a second"
    );
    let stalled = frame(&ctx, &mut queue, f64::NAN, &options);
    assert_eq!(
        surface_alpha(&stalled),
        255,
        "a non-finite clock keeps the toast visible"
    );
    let rewound = frame(&ctx, &mut queue, 50.0, &options);
    assert_eq!(
        surface_alpha(&rewound),
        255,
        "a clock that runs backwards keeps the toast visible"
    );
    assert_eq!(queue.len(), 1);
}
