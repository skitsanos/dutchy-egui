use super::{config, finish, harness};
use crate::{
    app::App,
    chat::Status,
    provider::{Role, Scenario},
};
use dutchy_egui::Mode;
use egui::{Event, ImeEvent, Key, Modifiers, OutputCommand, accesskit::Role as AccessibleRole};
use egui_kittest::kittest::Queryable;

#[test]
fn enter_sends_shift_enter_edits_and_stop_keeps_the_draft() {
    let mut h = harness(App::new(config()), 1160.0, Mode::Light, 1.0);
    h.get_by_label("Message").focus();
    h.step();
    h.get_by_label("Message").type_text("Show Rust");
    h.step();
    h.key_press_modifiers(Modifiers::SHIFT, Key::Enter);
    h.step();
    assert!(h.state().chat.current().draft.contains('\n'));
    assert!(h.state().chat.current().messages.is_empty());
    h.key_press(Key::Enter);
    h.step();
    assert_eq!(h.state().chat.current().messages.len(), 2);
    assert!(h.state().chat.current().active.is_some());
    assert!(h.query_by_label("Waiting for offline reply").is_some());
    h.get_by_label("Message").type_text("Keep this next draft");
    h.step();
    h.get_by_label("Stop reply").click();
    h.step();
    assert!(h.state().chat.current().active.is_none());
    assert_eq!(h.state().chat.current().messages[1].status, Status::Stopped);
    assert!(
        h.state()
            .chat
            .current()
            .draft
            .contains("Keep this next draft")
    );
    h.check_accessibility();
}

#[test]
fn ime_preedit_blocks_send_across_frames_until_commit() {
    let mut h = harness(App::new(config()), 1160.0, Mode::Light, 1.0);
    h.get_by_label("Message").focus();
    h.step();
    h.input_mut().events.push(Event::Ime(ImeEvent::Preedit {
        text: "é".into(),
        active_range_chars: None,
    }));
    h.step();
    h.key_press(Key::Enter);
    h.step();
    assert!(h.state().chat.current().messages.is_empty());
    h.input_mut()
        .events
        .push(Event::Ime(ImeEvent::Commit("é".into())));
    h.step();
    assert!(h.state().chat.current().messages.is_empty());
    h.key_press(Key::Enter);
    h.step();
    assert_eq!(h.state().chat.current().messages[0].text, "é");
}

#[test]
fn markdown_code_copy_and_reported_links_stay_inside_the_example() {
    let mut app = App::new(config());
    app.chat.current_mut().draft = "Rust".into();
    app.chat.send(&app.config, Scenario::Normal, 0.0).unwrap();
    finish(&mut app);
    let mut h = harness(app, 1160.0, Mode::Light, 1.0);
    h.get_by_role_and_label(AccessibleRole::Button, "Copy")
        .scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(AccessibleRole::Button, "Copy")
        .click();
    h.step();
    assert!(
        h.output()
            .platform_output
            .commands
            .iter()
            .any(|c| matches!(c,OutputCommand::CopyText(s) if s.contains("fn main()")))
    );
    assert!(!h.state().toasts.is_empty());
    h.get_by_role_and_label(AccessibleRole::Link, "About the offline provider")
        .scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(AccessibleRole::Link, "About the offline provider")
        .click();
    h.step();
    assert!(
        !h.output()
            .platform_output
            .commands
            .iter()
            .any(|c| matches!(c, OutputCommand::OpenUrl(_)))
    );
    assert!(
        h.state()
            .chat
            .current()
            .messages
            .iter()
            .any(|m| m.role == Role::Assistant)
    );
    h.check_accessibility();
}

#[test]
fn retry_uses_current_scenario_and_settings_remain_keyboard_accessible() {
    let mut app = App::new(config());
    app.scenario = Scenario::Authentication;
    app.chat.current_mut().draft = "Rust".into();
    app.chat.send(&app.config, app.scenario, 0.0).unwrap();
    finish(&mut app);
    let mut h = harness(app, 1160.0, Mode::Dark, 1.0);
    h.get_by_label("Chat settings").click();
    h.run_steps(4);
    assert!(h.query_by_label("Model name").is_some());
    assert_eq!(h.state().config.effort, "low");
    h.key_press(Key::Escape);
    h.step();
    h.run_steps(3);
    assert!(!h.state().settings_open);
    h.state_mut().scenario = Scenario::Normal;
    h.get_by_label("Retry reply").click();
    h.step();
    assert!(h.state().chat.current().active.is_some());
    assert_eq!(h.state().chat.current().messages.len(), 2);
    finish(h.state_mut());
    h.step();
    assert_eq!(
        h.state().chat.current().messages[1].status,
        Status::Complete
    );
    h.check_accessibility();
}

#[test]
fn compact_conversation_navigation_keeps_composer_visible() {
    let mut h = harness(App::new(config()), 560.0, Mode::Light, 1.0);
    assert!(h.query_by_label("Conversation").is_some());
    h.get_by_label("Start new conversation").click();
    h.step();
    assert_eq!(h.state().chat.conversations.len(), 2);
    let rect = h.get_by_label("Send message").rect();
    assert!(rect.bottom() <= 900.0 && rect.right() <= 560.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::N);
    h.step();
    assert_eq!(h.state().chat.conversations.len(), 3);
    h.check_accessibility();
}
