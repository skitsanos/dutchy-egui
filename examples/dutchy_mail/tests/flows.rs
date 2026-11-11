use super::{chord, harness, newest, row_label};
use crate::data::GARDEN;
use crate::mailbox::{ARCHIVE, INBOX, TRASH};
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn folders_show_unread_counts_and_switch_the_message_list() {
    let mut h = harness();
    let unread = h.state().mailbox.unread(INBOX);
    assert!(unread > 0);
    h.get_by_role_and_label(Role::TreeItem, "Inbox");
    h.get_by_label(&unread.to_string());
    h.get_by_role_and_label(Role::TreeItem, "Garden club")
        .click();
    h.run();
    assert_eq!(h.state().folder, GARDEN);
    let first = newest(&h, GARDEN);
    let label = row_label(&h, first);
    let row = h.get_by_role_and_label(Role::ListBoxOption, &label);
    assert_eq!(row.accesskit_node().is_selected(), Some(false));
    h.get_by_role_and_label(Role::Label, "Folder: Garden club");
    h.check_accessibility();
}

#[test]
fn opening_a_message_marks_it_read_until_it_is_marked_unread() {
    let mut h = harness();
    let id = h
        .state()
        .mailbox
        .view(INBOX, "")
        .iter()
        .find(|m| m.unread)
        .expect("an unread inbox message")
        .id;
    let before = h.state().mailbox.unread(INBOX);
    let label = row_label(&h, id);
    h.get_by_role_and_label(Role::ListBoxOption, &label).click();
    h.run();
    assert!(!h.state().mailbox.message(id).unwrap().unread);
    assert_eq!(h.state().mailbox.unread(INBOX), before - 1);
    // The reader shows the opened message's subject beside its row text.
    let subject = h.state().mailbox.message(id).unwrap().subject.clone();
    assert!(h.get_all_by_label(&subject).count() >= 2);
    h.get_by_label("Message body");
    let sender = h.state().mailbox.message(id).unwrap().from.clone();
    assert_eq!(
        h.get_all_by_role(Role::Image)
            .filter(|n| n.accesskit_node().label().as_deref() == Some(sender.as_str()))
            .count(),
        2,
        "the sender's avatar shows in the row and in the reader"
    );
    h.get_by_role_and_label(Role::Button, "Read / unread")
        .click();
    h.run();
    h.run_steps(3);
    assert!(
        h.state().mailbox.message(id).unwrap().unread,
        "Mark unread sticks"
    );
}

#[test]
fn archiving_moves_the_selection_and_undo_restores_it() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
        .click();
    h.run();
    chord(&mut h, Modifiers::COMMAND, Key::E);
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, ARCHIVE);
    h.get_by_label_contains("Archived 1 message.");
    h.get_by_role_and_label(Role::Button, "Undo").click();
    h.run();
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, INBOX);
}

#[test]
fn delete_moves_the_selection_to_trash_but_not_while_typing() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
        .click();
    h.run();
    h.get_by_label("Search messages").focus();
    h.run();
    h.get_by_label("Search messages").type_text("ab");
    h.run();
    chord(&mut h, Modifiers::NONE, Key::Delete);
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, INBOX);
    h.state_mut().query.clear();
    h.run();
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
        .click();
    h.run();
    chord(&mut h, Modifiers::NONE, Key::Delete);
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, TRASH);
    assert!(h.state().list.selected.is_empty());
}

#[test]
fn search_filters_sender_and_subject_and_can_be_cleared() {
    let mut h = harness();
    chord(&mut h, Modifiers::COMMAND, Key::F);
    assert!(h.get_by_label("Search messages").is_focused());
    h.get_by_label("Search messages").type_text("ferry");
    h.run();
    let rows: Vec<String> = h
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|n| n.accesskit_node().label())
        .collect();
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|label| label.to_lowercase().contains("ferry"))
    );
    h.get_by_label("Search messages").type_text(" zebra");
    h.run();
    assert!(h.get_all_by_label("No matching messages").next().is_some());
    h.get_by_role_and_label(Role::Button, "Clear search")
        .click();
    h.run();
    assert!(h.state().query.is_empty());
}

#[test]
fn abbreviated_search_leads_with_the_best_match_and_marks_it() {
    let mut h = harness();
    h.get_by_label("Search messages").focus();
    h.run();
    h.get_by_label("Search messages").type_text("frry sat");
    h.run();
    let first = h
        .get_all_by_role(Role::ListBoxOption)
        .find_map(|n| n.accesskit_node().label())
        .expect("a matching row");
    assert!(first.ends_with("Ferry times for Saturday"), "{first}");
    let marked = |text: &str| {
        let shapes = &h.output().shapes;
        let mut galleys = super::galleys(shapes).into_iter();
        galleys.any(|galley| galley.job.text == text && galley.job.sections.len() > 1)
    };
    assert!(marked("Ferry times for Saturday"));
}

#[test]
fn sync_delivers_one_new_message_after_a_short_wait() {
    let mut h = harness();
    let before = h.state().mailbox.total(INBOX);
    h.get_by_role_and_label(Role::Button, "Sync").click();
    h.run_steps(2);
    let busy = h.get_by_role_and_label(Role::ProgressIndicator, "Sync");
    assert!(busy.accesskit_node().is_busy());
    h.run_steps(8);
    assert_eq!(h.state().mailbox.total(INBOX), before + 1);
    h.get_by_label_contains("New message:");
    h.get_by_role_and_label(Role::Button, "Sync");
}

#[test]
fn empty_search_clears_reader_and_prevents_archiving_hidden_selection() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
        .click();
    h.run();
    h.get_by_label("Search messages").focus();
    h.run();
    h.get_by_label("Search messages")
        .type_text("totally unmatched 981276");
    h.run_steps(3);
    assert!(h.get_all_by_label("No matching messages").next().is_some());
    let selection_stale = h.state().list.selected.contains(&id);
    let reader_stale = h.query_by_label("Message body").is_some();
    chord(&mut h, Modifiers::COMMAND, Key::E);
    assert_eq!(
        (
            selection_stale,
            reader_stale,
            h.state().mailbox.message(id).unwrap().folder
        ),
        (false, false, INBOX),
    );
    assert_eq!(h.state().list.focused, None);
    h.get_by_role_and_label(Role::Heading, "No message selected");
    assert!(
        h.get_by_role_and_label(Role::Button, "Archive")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    h.get_by_role_and_label(Role::Button, "Clear search")
        .click();
    h.run();
    assert!(h.state().list.selected.is_empty());
}

#[test]
fn undo_archive_does_not_override_a_later_move_to_trash() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
        .click();
    h.run();
    chord(&mut h, Modifiers::COMMAND, Key::E);
    h.get_by_role_and_label(Role::TreeItem, "Archive").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
        .click();
    h.run();
    chord(&mut h, Modifiers::NONE, Key::Delete);
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, TRASH);
    let toast = h
        .get_all_by_role(Role::Status)
        .find(|node| node.accesskit_node().description().as_deref() == Some("Archived 1 message."))
        .expect("original archive toast");
    toast.get_by_role_and_label(Role::Button, "Undo").click();
    h.run();
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, TRASH);
    h.get_by_role_and_label(Role::Button, "Undo").click();
    h.run();
    assert_eq!(h.state().mailbox.message(id).unwrap().folder, ARCHIVE);
}
