//! DS-103 regressions: draft, focus, accessibility and layout faults.
use super::{config, finish, harness};
use crate::{app::App, provider::Scenario};
use dutchy_egui::Mode;
use egui::{Galley, Key, Modifiers, Shape, accesskit::Role, epaint::ClippedShape};
use egui_kittest::kittest::{NodeT, Queryable};

const HINT: &str = "Reply running · your next draft stays here";

fn sent(scenario: Scenario, done: bool) -> App {
    let mut app = App::new(config());
    app.chat.current_mut().draft = "Show a Rust example".into();
    app.chat.send(&app.config, scenario, 0.0).unwrap();
    if done {
        finish(&mut app);
    }
    app
}

fn text<'a>(shapes: &'a [ClippedShape], label: &str) -> (f32, &'a Galley) {
    fn find<'a>(shape: &'a Shape, label: &str) -> Option<(f32, &'a Galley)> {
        match shape {
            Shape::Text(text) if text.galley.job.text == label => Some((
                text.pos.y + text.galley.mesh_bounds.center().y,
                &text.galley,
            )),
            Shape::Vec(children) => children.iter().find_map(|s| find(s, label)),
            _ => None,
        }
    }
    shapes
        .iter()
        .find_map(|s| find(&s.shape, label))
        .unwrap_or_else(|| panic!("{label} painted"))
}

#[test]
fn sample_action_sends_the_example_and_keeps_the_typed_draft() {
    let mut app = App::new(config());
    app.chat.current_mut().draft = "My own question".into();
    let mut h = harness(app, 1160.0, Mode::Light, 1.0);
    h.get_by_label("Try the Rust example").click();
    h.step();
    let current = h.state().chat.current();
    assert_eq!(current.messages[0].text, "Show a Rust example");
    assert_eq!(current.draft, "My own question");
}

#[test]
fn settings_shortcut_returns_focus_to_the_message_field() {
    let mut h = harness(App::new(config()), 1160.0, Mode::Light, 1.0);
    h.get_by_role_and_label(Role::Button, "Chat settings")
        .click();
    h.run_steps(4);
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.get_by_label("Message").focus();
    h.step();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run_steps(4);
    assert!(h.state().settings_open);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().settings_open);
    assert!(h.get_by_label("Message").is_focused());
}

#[test]
fn sidebar_rows_expose_selection_and_running_replies() {
    let mut app = sent(Scenario::Normal, false);
    app.chat.new_conversation().unwrap();
    let h = harness(app, 1160.0, Mode::Light, 1.0);
    let running = h.get_by_label("Open conversation: Show a Rust example");
    assert_eq!(running.accesskit_node().is_selected(), Some(false));
    assert_eq!(
        running.accesskit_node().description().as_deref(),
        Some("Reply running")
    );
    let current = h.get_by_label("Open conversation: New conversation");
    assert_eq!(current.accesskit_node().is_selected(), Some(true));
    assert_eq!(current.accesskit_node().description(), None);
}

#[test]
fn failed_empty_reply_adds_no_empty_text_row() {
    let h = harness(sent(Scenario::RateLimit, true), 560.0, Mode::Dark, 1.0);
    let empty = h
        .query_all_by_role(Role::Label)
        .filter(|n| {
            let node = n.accesskit_node();
            node.label().unwrap_or_default().is_empty()
                && node.value().unwrap_or_default().is_empty()
        })
        .count();
    assert_eq!(empty, 0);
}

#[test]
fn heading_and_composer_copy_center_on_their_actions() {
    for width in [560.0, 1160.0] {
        let h = harness(sent(Scenario::Normal, true), width, Mode::Light, 1.0);
        let shapes = &h.output().shapes;
        // Mixed-case ink spans descenders, so allow a little more than caps.
        for (copy, action) in [
            (
                "Scripted replies · no network required",
                "DELETE CONVERSATION",
            ),
            ("Enter sends · Shift+Enter adds a line", "SEND MESSAGE"),
        ] {
            let offset = text(shapes, copy).0 - text(shapes, action).0;
            assert!(offset.abs() <= 3.0, "{width}: {copy} sits {offset} off");
        }
    }
}

#[test]
fn running_hint_never_splits_across_rows() {
    for width in [480.0, 560.0, 1160.0] {
        let h = harness(sent(Scenario::Normal, false), width, Mode::Dark, 1.0);
        let (_, galley) = text(&h.output().shapes, HINT);
        assert_eq!(galley.rows.len(), 1, "{width}: hint splits across rows");
    }
}
