//! Pointer activation retains native focus without a lingering keyboard ring.
mod common;
use dutchy_egui::{Theme, interaction::focus_is_visible};
use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput};

fn draw(ctx: &Context, events: Vec<Event>) -> bool {
    let mut visible = false;
    let mut output = ctx.run_ui(
        RawInput {
            events,
            ..Default::default()
        },
        |ui| {
            let response = ui.button("Control");
            response.request_focus();
            visible = focus_is_visible(&response);
            assert!(response.has_focus());
        },
    );
    output.textures_delta.clear();
    visible
}

#[test]
fn focus_cues_follow_keyboard_and_pointer_mode_without_removing_focus() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default());
    assert!(draw(&ctx, vec![]));
    assert!(!draw(
        &ctx,
        vec![Event::PointerButton {
            pos: Pos2::ZERO,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE
        }]
    ));
    assert!(!draw(&ctx, vec![]));
    assert!(!draw(
        &ctx,
        vec![Event::PointerMoved(egui::pos2(40.0, 40.0))]
    ));
    assert!(draw(
        &ctx,
        vec![Event::Key {
            key: Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE
        }]
    ));
    assert!(draw(&ctx, vec![Event::PointerMoved(Pos2::ZERO)]));
}

#[test]
fn text_ime_and_clipboard_input_restore_keyboard_mode() {
    let ctx = Context::default();
    common::install(&ctx, Theme::default());
    for event in [
        Event::Text("a".into()),
        Event::Ime(egui::ImeEvent::Preedit {
            text: "a".into(),
            active_range_chars: None,
        }),
        Event::Paste("a".into()),
        Event::Copy,
        Event::Cut,
    ] {
        assert!(!draw(
            &ctx,
            vec![Event::PointerButton {
                pos: Pos2::ZERO,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE
            }]
        ));
        assert!(draw(&ctx, vec![event.clone()]), "{event:?}");
    }
}
