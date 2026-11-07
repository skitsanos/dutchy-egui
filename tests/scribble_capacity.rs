//! History shortcuts during a keyboard draft, and gestures that reach the point cap.
mod common;
use dutchy_egui::{
    Scribble, ScribbleAction, ScribbleBrush, ScribbleDrawing, ScribbleError, ScribbleStroke, Theme,
    scribble::MAX_STROKE_POINTS,
};
use egui::{Context, Event, Key, Modifiers, PointerButton, RawInput, Rect, pos2, vec2};

fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn frame(
    ctx: &Context,
    time: u32,
    events: Vec<Event>,
    drawing: &mut ScribbleDrawing,
    focus: bool,
) -> (Rect, Vec<ScribbleAction>, Option<ScribbleError>) {
    let mut out = (Rect::NOTHING, Vec::new(), None);
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(740.0, 650.0))),
            time: Some(f64::from(time) / 60.0),
            events,
            ..Default::default()
        },
        |ui| {
            let response = Scribble::new("pad", drawing).show(ui);
            if focus {
                response.canvas.request_focus();
            }
            out = (response.canvas.rect, response.actions, response.error);
        },
    );
    output.textures_delta.clear();
    out
}
#[test]
fn undo_during_a_keyboard_draft_cancels_the_draft_without_a_history_request() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default());
    let mut drawing = ScribbleDrawing::default();
    drawing
        .push(ScribbleStroke::new(vec![pos2(10.0, 10.0)], ScribbleBrush::default()).unwrap())
        .unwrap();
    frame(&ctx, 0, vec![], &mut drawing, true);
    frame(&ctx, 1, vec![], &mut drawing, false);
    frame(
        &ctx,
        2,
        vec![key(Key::Space, Modifiers::NONE)],
        &mut drawing,
        false,
    );
    frame(
        &ctx,
        3,
        vec![key(Key::ArrowRight, Modifiers::NONE)],
        &mut drawing,
        false,
    );
    let (_, actions, _) = frame(
        &ctx,
        4,
        vec![key(Key::Z, Modifiers::CTRL)],
        &mut drawing,
        false,
    );
    assert!(
        actions.is_empty(),
        "the first Undo only cancels the draft: {actions:?}"
    );
    assert_eq!(drawing.strokes().len(), 1);
    let (_, actions, _) = frame(
        &ctx,
        5,
        vec![key(Key::Z, Modifiers::CTRL)],
        &mut drawing,
        false,
    );
    assert_eq!(actions, [ScribbleAction::Undo]);
    assert_eq!(drawing.strokes().len(), 1);
}
#[test]
fn a_gesture_that_reaches_the_point_cap_commits_the_captured_points() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default());
    let mut drawing = ScribbleDrawing::default();
    let (canvas, _, _) = frame(&ctx, 0, vec![], &mut drawing, false);
    let origin = canvas.left_top() + vec2(10.0, 10.0);
    let mut events = vec![Event::PointerButton {
        pos: origin,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    }];
    for n in 0..(MAX_STROKE_POINTS + 800) {
        let row = (n / 500) as f32;
        let col = (n % 500) as f32;
        events.push(Event::PointerMoved(origin + vec2(col, row * 1.5)));
    }
    let (_, _, error) = frame(&ctx, 1, events, &mut drawing, false);
    assert_eq!(error, Some(ScribbleError::Capacity("gesture points")));
    assert_eq!(
        drawing.strokes().len(),
        1,
        "the captured gesture must be kept"
    );
    assert_eq!(drawing.strokes()[0].points().len(), MAX_STROKE_POINTS);
    let release = Event::PointerButton {
        pos: origin,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    };
    let (_, _, error) = frame(&ctx, 2, vec![release], &mut drawing, false);
    assert_eq!(error, None);
    assert_eq!(drawing.strokes().len(), 1);
}
