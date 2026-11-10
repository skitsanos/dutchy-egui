use super::support::*;
use dutchy_egui::InlineEditEvent;
use egui::{Event, ImeEvent, Key, Modifiers, accesskit::Role};
use egui_kittest::kittest::Queryable;

fn press(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn text_and_enter_in_one_native_frame_commit_the_final_text() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.input_mut().events = vec![Event::Text("Final name".into()), press(Key::Enter)];
    h.run_steps(3);
    assert_eq!(h.state().value, "Final name");
    assert_eq!(
        h.state().events,
        [
            InlineEditEvent::Started,
            InlineEditEvent::Commit("Final name".into())
        ]
    );
}

#[test]
fn discarded_passes_do_not_reactivate_a_committed_or_cancelled_editor() {
    for action in ["editor", "save", "cancel"] {
        let mut h = harness(Page {
            discard: true,
            ..Default::default()
        });
        start(&mut h);
        h.get_by_label(NAME).type_text("Once");
        h.run_steps(3);
        let key_value = match action {
            "save" => {
                h.get_by_role_and_label(Role::Button, "Save Document name")
                    .focus();
                Key::Space
            }
            "cancel" => {
                h.get_by_role_and_label(Role::Button, "Cancel Document name")
                    .focus();
                Key::Space
            }
            _ => Key::Enter,
        };
        h.run_steps(2);
        h.input_mut().events = vec![press(key_value)];
        h.run_steps(3);
        assert!(!h.state().edit.is_editing(), "{action}");
        assert_eq!(
            h.state().events.len(),
            2,
            "{action}: {:?}",
            h.state().events
        );
        assert_eq!(
            h.state().value,
            if action == "cancel" {
                "Design notes.md"
            } else {
                "Once"
            }
        );
    }
}

#[test]
fn ime_enter_keeps_the_session_open_until_composition_commits() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.input_mut().events = vec![
        Event::Ime(ImeEvent::Preedit {
            text: "é".into(),
            active_range_chars: None,
        }),
        press(Key::Enter),
    ];
    h.run_steps(2);
    assert!(h.state().edit.is_editing());
    assert_eq!(h.state().events.len(), 1);
    h.event(Event::Key {
        key: Key::Enter,
        physical_key: Some(Key::Enter),
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.event(Event::Ime(ImeEvent::Commit("é".into())));
    h.run_steps(2);
    key(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(h.state().value, "é");
}

#[test]
fn ending_ime_without_a_commit_does_not_leave_rename_keys_blocked() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.event(Event::Ime(ImeEvent::Preedit {
        text: "e".into(),
        active_range_chars: None,
    }));
    h.run_steps(2);
    h.event(Event::Ime(ImeEvent::Preedit {
        text: String::new(),
        active_range_chars: None,
    }));
    h.run_steps(2);
    key(&mut h, Key::Escape, Modifiers::NONE);
    assert!(!h.state().edit.is_editing());
    assert_eq!(h.state().value, "Design notes.md");
}

#[test]
fn holding_space_to_start_does_not_type_spaces_over_the_selected_name() {
    let mut h = harness(Page::default());
    h.get_by_label("Rename Document name").focus();
    h.run_steps(2);
    let space = |pressed, repeat| Event::Key {
        key: Key::Space,
        physical_key: Some(Key::Space),
        pressed,
        repeat,
        modifiers: Modifiers::NONE,
    };
    h.input_mut().events = vec![space(true, false), Event::Text(" ".into())];
    h.run_steps(1);
    for _ in 0..3 {
        h.input_mut().events = vec![space(true, true), Event::Text(" ".into())];
        h.run_steps(1);
    }
    h.input_mut().events = vec![space(false, false)];
    h.run_steps(2);
    assert_eq!(h.state().edit.draft(), Some("Design notes.md"));
    // After release, Space types again and replaces the still-selected name.
    h.input_mut().events = vec![space(true, false), Event::Text(" ".into())];
    h.run_steps(2);
    assert_eq!(h.state().edit.draft(), Some(" "));
}

#[test]
fn space_release_and_new_press_in_one_frame_leave_the_new_space_intact() {
    let space = |pressed| Event::Key {
        key: Key::Space,
        physical_key: Some(Key::Space),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    for move_focus in [true, false] {
        let mut h = harness(Page::default());
        h.get_by_label("Rename Document name").focus();
        h.run_steps(2);
        h.input_mut().events = vec![space(true), Event::Text(" ".into())];
        h.run_steps(2);
        if move_focus {
            h.get_by_label("Other field").focus();
            h.run_steps(2);
            assert!(h.get_by_label("Other field").is_focused());
        }
        h.input_mut().events = vec![space(false), space(true), Event::Text(" ".into())];
        h.run_steps(3);
        if move_focus {
            assert_eq!(h.state().other, " ");
            assert_eq!(h.state().edit.draft(), Some("Design notes.md"));
        } else {
            assert_eq!(h.state().edit.draft(), Some(" "));
        }
    }
}

#[test]
fn cancellation_does_not_leave_old_drafts_in_the_next_session_undo_history() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.get_by_label(NAME).type_text("Never restore this draft");
    h.run_steps(3);
    key(&mut h, Key::Escape, Modifiers::NONE);
    start(&mut h);
    key(&mut h, Key::Z, Modifiers::COMMAND);
    assert_eq!(h.state().edit.draft(), Some("Design notes.md"));
    h.ctx
        .set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
    h.event(Event::Paste("é🦊".repeat(50)));
    h.run_steps(3);
    assert_eq!(h.state().edit.draft().unwrap().chars().count(), 80);
    assert_eq!(h.state().value, "Design notes.md");
}
