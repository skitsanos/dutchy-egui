//! Native-style split key down/up frames retain arrow focus but allow Tab exit.
mod common;
use dutchy_egui::{Scribble, ScribbleBrush, ScribbleDrawing, ScribbleStroke, Theme};
use egui::{Context, Event, Key, Modifiers, RawInput, Rect, pos2, vec2};
#[test]
fn split_native_key_frames_draw_without_directional_focus_escaping_to_clear() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default());
    let mut drawing = ScribbleDrawing::default();
    drawing
        .push(ScribbleStroke::new(vec![pos2(10.0, 10.0)], ScribbleBrush::default()).unwrap())
        .unwrap();
    let mut canvas_id = None;
    let mut requests = vec![];
    for frame in 0..12 {
        let events = match frame {
            2 | 3 => vec![key(Key::Space, frame == 2)],
            4 | 5 => vec![key(Key::ArrowDown, frame == 4)],
            6 | 7 => vec![key(Key::ArrowRight, frame == 6)],
            8 | 9 => vec![key(Key::Enter, frame == 8)],
            10 | 11 => vec![key(Key::Tab, frame == 10)],
            _ => vec![],
        };
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(740.0, 650.0))),
                time: Some(f64::from(frame) / 60.0),
                events,
                ..Default::default()
            },
            |ui| {
                let response = Scribble::new("pad", &mut drawing).show(ui);
                canvas_id = Some(response.canvas.id);
                requests.extend(response.actions);
                if frame == 1 {
                    response.canvas.request_focus();
                }
            },
        );
        output.textures_delta.clear();
        if (2..=9).contains(&frame) {
            assert_eq!(ctx.memory(|m| m.focused()), canvas_id, "frame {frame}");
        }
    }
    assert_ne!(
        ctx.memory(|m| m.focused()),
        canvas_id,
        "Tab must leave the canvas"
    );
    assert!(
        requests.is_empty(),
        "Drawing must not activate the Clear button: {requests:?}"
    );
    assert_eq!(drawing.strokes().len(), 2);
    assert_eq!(
        drawing.strokes()[1].points(),
        [pos2(300.0, 100.0), pos2(300.0, 104.0), pos2(304.0, 104.0)]
    );
}
fn key(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
