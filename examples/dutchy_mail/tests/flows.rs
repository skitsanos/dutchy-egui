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
