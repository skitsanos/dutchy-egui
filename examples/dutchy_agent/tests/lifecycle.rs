use super::{config, finish};
use crate::{
    app::App,
    chat::{MAX_PROMPT, Status},
    provider::{Change, Event, Role, Scenario, Usage},
};

fn start(app: &mut App, text: &str) {
    app.chat.current_mut().draft = text.into();
    app.chat.send(&app.config, app.scenario, 0.0).unwrap();
}
#[test]
fn sending_validates_without_losing_drafts_and_titles_use_unicode() {
    let mut app = App::new(config());
    for draft in ["  ".to_owned(), "🦀".repeat(MAX_PROMPT + 1)] {
        app.chat.current_mut().draft = draft.clone();
        assert!(app.chat.send(&app.config, app.scenario, 0.0).is_err());
        assert_eq!(app.chat.current().draft, draft);
        assert!(app.chat.current().messages.is_empty());
    }
    start(&mut app, "漢字 Rust sketch");
    assert_eq!(app.chat.current().title, "漢字 Rust sketch");
    assert_eq!(app.chat.current().messages.len(), 2);
    assert!(app.chat.current().draft.is_empty());
    app.chat.current_mut().draft = "A future draft".into();
    assert!(app.chat.send(&app.config, app.scenario, 0.1).is_err());
    assert_eq!(app.chat.current().draft, "A future draft");
}
#[test]
fn switching_preserves_drafts_and_independent_background_replies() {
    let mut app = App::new(config());
    let first = app.chat.selected;
    start(&mut app, "Rust first");
    app.chat.current_mut().draft = "First draft".into();
    app.chat.new_conversation().unwrap();
    let second = app.chat.selected;
    start(&mut app, "Second prompt");
    assert_ne!(app.chat.current().active, app.chat.conversations[0].active);
    finish(&mut app);
    assert!(
        app.chat
            .conversations
            .iter()
            .all(|c| c.active.is_none() && c.usage.is_some())
    );
    app.chat.select(first);
    assert_eq!(app.chat.current().draft, "First draft");
    app.chat.select(second);
    assert_eq!(app.chat.current().title, "Second prompt");
}
#[test]
fn stop_preserves_partial_text_and_retry_ignores_old_events() {
    let mut app = App::new(config());
    start(&mut app, "Rust");
    let old = app.chat.current().active.unwrap();
    app.chat.poll(0.5);
    let partial = app.chat.current().messages[1].text.clone();
    assert!(!partial.is_empty());
    assert!(app.chat.stop());
    assert!(!app.chat.stop());
    assert_eq!(app.chat.current().messages[1].text, partial);
    app.chat.retry(&app.config, Scenario::Normal, 1.0).unwrap();
    let run = app.chat.current().active;
    assert!(
        app.chat
            .apply(Event {
                request: old,
                change: Change::Delta("stale".into())
            })
            .is_none()
    );
    assert!(
        app.chat
            .apply(Event {
                request: old,
                change: Change::Complete(Usage::default())
            })
            .is_none()
    );
    assert_eq!(app.chat.current().active, run);
    assert!(app.chat.current().messages[1].text.is_empty());
    finish(&mut app);
    assert_eq!(app.chat.current().messages.len(), 2);
    assert_eq!(app.chat.current().messages[1].status, Status::Complete);
}
#[test]
fn failed_retry_keeps_one_user_turn_and_captures_request_settings() {
    let mut app = App::new(config());
    app.scenario = Scenario::Network;
    start(&mut app, "Rust");
    finish(&mut app);
    assert!(matches!(
        app.chat.current().messages[1].status,
        Status::Failed(_)
    ));
    assert!(!app.chat.current().messages[1].text.is_empty());
    app.chat.retry(&app.config, Scenario::Normal, 0.0).unwrap();
    app.config.model = "changed-later".into();
    finish(&mut app);
    assert_eq!(
        app.chat
            .current()
            .messages
            .iter()
            .filter(|m| m.role == Role::User)
            .count(),
        1
    );
    assert!(app.chat.current().messages[1].text.contains("gpt-6-luna"));
    assert!(
        !app.chat.current().messages[1]
            .text
            .contains("changed-later")
    );
}
#[test]
fn deleting_cancels_runs_and_always_retains_a_valid_selection() {
    let mut app = App::new(config());
    start(&mut app, "Delete this");
    let old = app.chat.selected;
    app.chat.delete_current().unwrap();
    assert_ne!(app.chat.selected, old);
    assert!(app.chat.next_due().is_none());
    assert!(app.chat.current().messages.is_empty());
    assert_eq!(app.chat.conversations.len(), 1);
    let first = app.chat.selected;
    app.chat.new_conversation().unwrap();
    let second = app.chat.selected;
    app.chat.select(first);
    app.chat.delete_current().unwrap();
    assert_eq!(app.chat.selected, second);
    app.chat.select(u64::MAX);
    assert_eq!(app.chat.selected, second);
}
