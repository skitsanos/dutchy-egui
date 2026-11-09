use super::{config, finish};
use crate::{app::App, chat::Status, config::Config, provider::Mode};
use egui_kittest::kittest::Queryable;

#[test]
fn key_presence_selects_live_and_debug_stays_redacted() {
    let c = Config::from_sources(
        Some("OPENAI_API_KEY=fixture-secret-value\nMODEL_NAME=gpt-6-luna\nMODEL_EFFORT=low\n"),
        |_| None,
    );
    assert_eq!(c.provider, Mode::OpenAi);
    assert!(c.key_configured);
    assert!(!format!("{c:?}").contains("fixture-secret-value"));
}
#[test]
fn unavailable_credentials_fail_live_without_substituting_a_mock() {
    let mut app = App::new(config());
    app.config.provider = Mode::OpenAi;
    app.chat.current_mut().draft = "Hi".into();
    app.chat.send(&app.config, app.scenario, 0.0).unwrap();
    app.chat.poll(0.0);
    let reply = &app.chat.current().messages[1];
    assert_eq!(reply.provider, Mode::OpenAi);
    assert!(reply.text.is_empty());
    assert!(matches!(reply.status, Status::Failed(_)));
    app.config.provider = Mode::Mock;
    app.chat.retry(&app.config, app.scenario, 0.0).unwrap();
    finish(&mut app);
    assert_eq!(app.chat.current().messages.len(), 2);
    assert_eq!(app.chat.current().messages[1].provider, Mode::Mock);
    assert_eq!(app.chat.current().messages[1].status, Status::Complete);
}
#[test]
fn switching_provider_does_not_relabel_a_running_request() {
    let mut app = App::new(config());
    app.chat.current_mut().draft = "Rust".into();
    app.chat.send(&app.config, app.scenario, 0.0).unwrap();
    app.config.provider = Mode::OpenAi;
    finish(&mut app);
    assert_eq!(app.chat.current().messages[1].provider, Mode::Mock);
}

#[test]
fn provider_choice_is_accessible_and_never_sends_until_requested() {
    let mut app = App::new(config());
    app.config.provider = Mode::OpenAi;
    let mut h = super::harness(app, 1160.0, dutchy_egui::Mode::Light, 1.0);
    h.get_by_label("Chat settings").click();
    h.run_steps(4);
    h.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Provider")
        .click();
    h.run_steps(3);
    h.get_by_label("Offline mock").click();
    h.step();
    assert_eq!(h.state().config.provider, Mode::Mock);
    assert!(h.state().chat.current().messages.is_empty());
    h.key_press(egui::Key::Escape);
    h.run_steps(4);
    h.check_accessibility();
}
