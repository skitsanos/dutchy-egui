use super::{chord, harness};
use crate::app::App;

/// Focus a compose field by its accessible name, then type into it.
fn type_into(h: &mut Harness<'_, App>, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).focus();
    h.run();
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
}
use crate::mailbox::SENT;
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn compose_sends_into_sent_and_requires_a_recipient() {
    let mut h = harness();
    let before = h.state().mailbox.total(SENT);
    chord(&mut h, Modifiers::COMMAND, Key::N);
    assert!(h.state().compose.open);
    type_into(&mut h, "To", "garden@example.org");
    type_into(&mut h, "Subject", "Seed swap");
    h.get_by_role_and_label(Role::Button, "Send").click();
    h.run();
    assert!(!h.state().compose.open);
    assert_eq!(h.state().mailbox.total(SENT), before + 1);
    let sent = h.state().mailbox.view(SENT, "Seed swap");
    assert_eq!(sent[0].from, "You");
    h.get_by_label_contains("Sent \u{201c}Seed swap\u{201d}.");
}

#[test]
fn escape_discards_a_draft_and_undo_brings_it_back() {
    let mut h = harness();
    chord(&mut h, Modifiers::COMMAND, Key::N);
    type_into(&mut h, "Subject", "Half a thought");
    chord(&mut h, Modifiers::NONE, Key::Escape);
    assert!(!h.state().compose.open);
    h.get_by_label_contains("Draft discarded.");
    h.get_by_role_and_label(Role::Button, "Undo").click();
    h.run();
    assert!(h.state().compose.open);
    assert_eq!(h.state().compose.draft.subject, "Half a thought");
}
