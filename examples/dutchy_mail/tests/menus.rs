use super::{chord, harness, newest, row_label};
use crate::{
    app::App,
    commands::Action,
    mailbox::{ARCHIVE, INBOX, MessageId, TRASH},
};
use egui::{Key, KeyboardShortcut, Modifiers, accesskit::Role};
use egui_kittest::{
    Harness, Node,
    kittest::{NodeT, Queryable},
};

fn row<'h>(h: &'h Harness<'_, App>, id: MessageId) -> Node<'h> {
    let label = row_label(h, id);
    h.get_all_by_role(Role::ListBoxOption)
        .find(|node| node.accesskit_node().label().as_deref() == Some(label.as_str()))
        .expect("message row is visible")
}

fn open_menu(h: &mut Harness<'_, App>, name: &str) {
    h.get_by_role_and_label(Role::MenuItem, name).click();
    h.run_steps(3);
}

fn folder(h: &Harness<'_, App>, id: MessageId) -> crate::mailbox::FolderId {
    h.state().mailbox.message(id).unwrap().folder
}

#[test]
fn menu_bar_shares_availability_and_executes_the_selected_command_once() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    h.get_by_role_and_label(Role::MenuBar, "DutchyMail menus");
    open_menu(&mut h, "Message");
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Archive")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    chord(&mut h, Modifiers::NONE, Key::Escape);
    row(&h, id).click();
    h.run();
    open_menu(&mut h, "Message");
    let archive = h.get_by_role_and_label(Role::MenuItem, "Archive");
    assert!(!archive.accesskit_node().data().is_disabled());
    archive.click();
    h.run_steps(3);
    assert_eq!(folder(&h, id), ARCHIVE);
    assert_eq!(h.state().toasts.len(), 1);
    assert!(
        h.query_by_role_and_label(Role::MenuItem, "Archive")
            .is_none()
    );
    h.check_accessibility();
}

#[test]
fn right_click_an_unselected_row_targets_it_without_replacing_the_selection() {
    let mut h = harness();
    let ids: Vec<_> = h
        .state()
        .mailbox
        .view(INBOX, "")
        .iter()
        .take(2)
        .map(|m| m.id)
        .collect();
    row(&h, ids[0]).click();
    h.run();
    let target = row(&h, ids[1]);
    let rect = target.rect();
    target.click_secondary();
    h.run_steps(3);
    assert_eq!(row(&h, ids[1]).rect(), rect);
    assert_eq!(h.state().opened, Some(ids[0]));
    assert_eq!(h.state().list.selected.len(), 1);
    h.get_by_role_and_label(Role::Menu, "Message context actions");
    h.get_by_role_and_label(Role::MenuItem, "Archive").click();
    h.run_steps(3);
    assert_eq!(folder(&h, ids[0]), INBOX);
    assert_eq!(folder(&h, ids[1]), ARCHIVE);
    assert!(h.state().list.selected.contains(&ids[0]));
}

#[test]
fn row_actions_work_without_a_selection_and_close_when_the_row_is_filtered_out() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    let unread = h.state().mailbox.message(id).unwrap().unread;
    row(&h, id).click_secondary();
    h.run_steps(3);
    h.get_by_role_and_label(Role::MenuItem, "Read / unread")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().mailbox.message(id).unwrap().unread, !unread);
    assert!(h.state().list.selected.is_empty());
    assert_eq!(h.state().opened, None);

    row(&h, id).click_secondary();
    h.run_steps(3);
    h.get_by_role_and_label(Role::MenuItem, "Archive");
    h.state_mut().query = "zqxv 981276".into();
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::MenuItem, "Archive")
            .is_none()
    );
    assert_eq!(folder(&h, id), INBOX);
}

#[test]
fn keyboard_context_menu_returns_focus_and_targets_the_selected_group() {
    let mut h = harness();
    let ids: Vec<_> = h
        .state()
        .mailbox
        .view(INBOX, "")
        .iter()
        .take(2)
        .map(|m| m.id)
        .collect();
    row(&h, ids[0]).click();
    h.run();
    row(&h, ids[1]).click_modifiers(Modifiers::COMMAND);
    h.run();
    assert_eq!(h.state().list.selected.len(), 2);
    chord(&mut h, Modifiers::SHIFT, Key::F10);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Archive")
            .is_focused()
    );
    chord(&mut h, Modifiers::NONE, Key::Escape);
    assert!(row(&h, ids[1]).is_focused());
    chord(&mut h, Modifiers::SHIFT, Key::F10);
    chord(&mut h, Modifiers::NONE, Key::ArrowDown);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Delete")
            .is_focused()
    );
    chord(&mut h, Modifiers::NONE, Key::Enter);
    assert!(ids.iter().all(|id| folder(&h, *id) == TRASH));
    assert!(h.state().list.selected.is_empty());
}

#[test]
fn changing_one_registry_entry_updates_all_surfaces_and_dispatch() {
    let mut h = harness();
    let id = newest(&h, INBOX);
    let binding = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::Y);
    let hint = h.ctx.format_shortcut(&binding);
    let command = h
        .state_mut()
        .commands
        .registry
        .get_mut(&Action::Archive)
        .unwrap();
    command.label = "File away".into();
    command.shortcut = Some(binding);
    row(&h, id).click();
    h.run();
    h.get_by_role_and_label(Role::Button, "File away");
    open_menu(&mut h, "Message");
    let description = format!("Shortcut: {hint}");
    assert_eq!(
        h.get_by_role_and_label(Role::MenuItem, "File away")
            .accesskit_node()
            .description()
            .as_deref(),
        Some(description.as_str())
    );
    chord(&mut h, Modifiers::NONE, Key::Escape);
    row(&h, id).click_secondary();
    h.run_steps(3);
    assert_eq!(
        h.get_by_role_and_label(Role::MenuItem, "File away")
            .accesskit_node()
            .description()
            .as_deref(),
        Some(description.as_str())
    );
    chord(&mut h, binding.modifiers, binding.logical_key);
    assert_eq!(folder(&h, id), INBOX, "an open menu owns keyboard input");
    chord(&mut h, Modifiers::NONE, Key::Escape);
    h.run_steps(2);
    chord(&mut h, Modifiers::COMMAND, Key::E);
    assert_eq!(folder(&h, id), INBOX);
    chord(&mut h, binding.modifiers, binding.logical_key);
    assert_eq!(folder(&h, id), ARCHIVE);
}
