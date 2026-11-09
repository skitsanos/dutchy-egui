//! Keyboard flows: N creates, Enter edits, Space toggles, Delete removes,
//! Escape closes editors, and focus returns to the list row.

use super::harness;
use crate::app::App;
use crate::store::{Filter, Priority};
use crate::support::data::FIXED_TODAY;
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const FIRST: &str = "Order seed trays";
const SECOND: &str = "Plant garlic in the raised bed";
const LAST: &str = "Clear the gutters";

fn titles(h: &Harness<'_, App>) -> Vec<String> {
    h.state()
        .visible_tasks()
        .into_iter()
        .map(|task| task.title)
        .collect()
}

fn focus_row(h: &mut Harness<'_, App>, title: &str) {
    h.get_by_role_and_label(Role::ListItem, title).focus();
    h.run();
    assert!(h.get_by_role_and_label(Role::ListItem, title).is_focused());
}

fn task_done(h: &Harness<'_, App>, title: &str) -> bool {
    h.state()
        .store
        .tasks()
        .find(|task| task.title == title)
        .expect("task exists")
        .done
}

#[test]
fn rows_are_ordered_and_arrows_move_focus_within_the_list() {
    let mut h = harness();
    assert_eq!(titles(&h)[0], FIRST);
    assert_eq!(titles(&h)[1], SECOND);
    assert_eq!(titles(&h)[5], LAST);
    focus_row(&mut h, FIRST);
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::ListItem, SECOND).is_focused());
    h.key_press(Key::End);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::ListItem, LAST).is_focused());
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::ListItem, LAST).is_focused(),
        "the last row keeps focus"
    );
    h.key_press(Key::Home);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::ListItem, FIRST).is_focused());
    h.key_press(Key::ArrowUp);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::ListItem, FIRST).is_focused());
    h.check_accessibility();
}

#[test]
fn space_completes_with_an_undo_toast_that_returns_focus_to_the_row() {
    let mut h = harness();
    focus_row(&mut h, FIRST);
    h.key_press(Key::Space);
    h.run_steps(3);
    assert!(task_done(&h, FIRST));
    assert_eq!(titles(&h).last().map(String::as_str), Some(LAST));
    assert!(
        h.get_by_role_and_label(Role::ListItem, FIRST).is_focused(),
        "the completed row keeps focus while it moves down the list"
    );
    let toast = h.get_by_role_and_label(Role::Status, "Success: Task completed");
    toast.get_by_label("Undo").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(!task_done(&h, FIRST));
    assert!(h.state().toasts.is_empty());
    assert!(h.get_by_role_and_label(Role::ListItem, FIRST).is_focused());
    h.key_press(Key::Space);
    h.run_steps(3);
    h.key_press(Key::Space);
    h.run_steps(3);
    assert!(!task_done(&h, FIRST), "Space toggles back without a toast");
    assert_eq!(h.state().toasts.len(), 1);
}

#[test]
fn the_checkbox_toggles_by_pointer_without_opening_the_editor() {
    let mut h = harness();
    h.get_by_role_and_label(Role::CheckBox, SECOND).click();
    h.run_steps(3);
    assert!(task_done(&h, SECOND));
    assert!(!h.state().editor.open);
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, SECOND)
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
}

#[test]
fn enter_edits_in_a_drawer_and_escape_returns_focus_to_the_row() {
    let mut h = harness();
    focus_row(&mut h, FIRST);
    h.key_press(Key::Enter);
    h.run_steps(5);
    assert!(h.state().editor.open);
    h.get_by_role_and_label(Role::Dialog, "Edit task");
    assert!(
        h.get_by_label("Title").is_focused(),
        "the title field takes focus"
    );
    assert_eq!(h.state().editor.draft.title, FIRST);
    h.check_accessibility();
    h.key_press(Key::Escape);
    h.run_steps(5);
    assert!(!h.state().editor.open);
    assert!(
        h.query_by_role_and_label(Role::Dialog, "Edit task")
            .is_none()
    );
    assert!(h.get_by_role_and_label(Role::ListItem, FIRST).is_focused());
    assert_eq!(titles(&h)[0], FIRST, "Escape saved nothing");
}

#[test]
fn enter_in_the_title_saves_the_edit_and_keeps_the_row_focused() {
    let mut h = harness();
    focus_row(&mut h, FIRST);
    h.key_press(Key::Enter);
    h.run_steps(5);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_label("Title")
        .type_text("Order seed trays and labels");
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(5);
    assert!(!h.state().editor.open);
    assert_eq!(titles(&h)[0], "Order seed trays and labels");
    assert!(
        h.get_by_role_and_label(Role::ListItem, "Order seed trays and labels")
            .is_focused()
    );
}

#[test]
fn a_blank_title_is_rejected_and_keeps_the_editor_open() {
    let mut h = harness();
    focus_row(&mut h, FIRST);
    h.key_press(Key::Enter);
    h.run_steps(5);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.key_press(Key::Backspace);
    h.run();
    h.get_by_label("Save task").click();
    h.run_steps(3);
    assert!(h.state().editor.open);
    h.get_by_role_and_label(Role::Alert, "Enter a title before saving.");
    assert!(h.get_by_label("Title").is_focused());
    assert_eq!(titles(&h)[0], FIRST);
}

#[test]
fn n_creates_a_task_unless_a_text_field_has_focus() {
    let mut h = harness();
    h.get_by_label("Search tasks").focus();
    h.run();
    h.get_by_label("Search tasks").type_text("n");
    h.run_steps(2);
    assert!(
        !h.state().editor.open,
        "typing into the search is not a shortcut"
    );
    assert_eq!(h.state().query, "n");
    h.get_by_label("Clear Search tasks").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "New task").focus();
    h.run();
    h.key_press(Key::Escape);
    h.run();
    h.key_press(Key::N);
    h.run_steps(5);
    assert!(h.state().editor.open);
    h.get_by_role_and_label(Role::Dialog, "New task");
    assert!(h.get_by_label("Title").is_focused());
    h.get_by_label("Title").type_text("Water the ferns");
    h.run();
    h.get_by_role_and_label(Role::RadioButton, "High").click();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(
        h.state().editor.open,
        "Enter on the priority control is not a save"
    );
    h.get_by_label("Save task").click();
    h.run_steps(5);
    assert!(!h.state().editor.open);
    let task = h
        .state()
        .store
        .tasks()
        .find(|task| task.title == "Water the ferns")
        .expect("the task was created");
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.project, h.state().selected.unwrap());
    assert!(
        h.get_by_role_and_label(Role::ListItem, "Water the ferns")
            .is_focused()
    );
}

#[test]
fn delete_removes_focuses_the_neighbor_and_undo_restores_the_row() {
    let mut h = harness();
    focus_row(&mut h, FIRST);
    h.key_press(Key::Delete);
    h.run_steps(4);
    assert!(!titles(&h).iter().any(|title| title == FIRST));
    assert!(h.get_by_role_and_label(Role::ListItem, SECOND).is_focused());
    h.get_by_role_and_label(Role::Status, "Information: Task deleted")
        .get_by_label("Undo")
        .click();
    h.run_steps(4);
    assert_eq!(titles(&h)[0], FIRST);
    assert!(h.get_by_role_and_label(Role::ListItem, FIRST).is_focused());
    focus_row(&mut h, LAST);
    h.key_press(Key::Backspace);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::ListItem, "Compare rainwater tanks")
            .is_focused(),
        "deleting the last row focuses the one before it"
    );
}

#[test]
fn the_editor_delete_button_removes_the_task_and_moves_focus_on() {
    let mut h = harness();
    focus_row(&mut h, SECOND);
    h.key_press(Key::Enter);
    h.run_steps(5);
    h.get_by_label("Delete task").click();
    h.run_steps(5);
    assert!(!h.state().editor.open);
    assert!(!titles(&h).iter().any(|title| title == SECOND));
    assert!(
        h.get_by_role_and_label(Role::ListItem, "Repair the fence post by the gate")
            .is_focused()
    );
}

#[test]
fn filters_and_search_narrow_the_list_and_offer_a_way_back() {
    let mut h = harness();
    h.get_by_role_and_label(Role::RadioButton, "Done").click();
    h.run_steps(2);
    assert_eq!(titles(&h), [LAST]);
    h.get_by_role_and_label(Role::RadioButton, "Active").click();
    h.run_steps(2);
    assert_eq!(titles(&h).len(), 5);
    assert!(!titles(&h).iter().any(|title| title == LAST));
    h.get_by_label("Search tasks").focus();
    h.run();
    h.get_by_label("Search tasks").type_text("gutters");
    h.run_steps(2);
    h.get_by_role_and_label(Role::Heading, "No matching tasks");
    h.get_by_label("Clear search").click();
    h.run_steps(3);
    assert!(h.state().query.is_empty());
    assert!(h.get_by_label("Search tasks").is_focused());
    assert_eq!(titles(&h).len(), 5);
    h.get_by_label("Search tasks").type_text("gutters");
    h.run_steps(2);
    h.get_by_role_and_label(Role::RadioButton, "All").click();
    h.run_steps(2);
    assert_eq!(titles(&h), [LAST]);
}

#[test]
fn empty_filters_explain_themselves_and_an_empty_project_invites_a_task() {
    let mut h = harness();
    h.get_by_label("New project name").focus();
    h.run();
    h.get_by_label("New project name").type_text("Holiday prep");
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(3);
    let selected = h.state().selected.unwrap();
    assert_eq!(
        h.state().store.project(selected).unwrap().name,
        "Holiday prep"
    );
    assert!(
        h.get_by_role_and_label(Role::Button, "Holiday prep")
            .is_focused()
    );
    h.get_by_role_and_label(Role::Heading, "No tasks yet");
    h.get_by_label("Add a task").click();
    h.run_steps(4);
    assert!(h.state().editor.open);
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.state_mut().filter = Some(Filter::Done);
    h.run_steps(2);
    h.get_by_role_and_label(Role::Heading, "Nothing completed yet");
    h.get_by_label("Show all tasks").click();
    h.run_steps(2);
    assert_eq!(h.state().filter, Some(Filter::All));
}

#[test]
fn deleting_a_project_needs_confirmation_and_undo_restores_everything() {
    let mut h = harness();
    let garden = h.state().selected.unwrap();
    let before = h.state().store.clone();
    h.get_by_label("Delete project…").click();
    h.run_steps(5);
    let dialog = h.get_by_role_and_label(Role::Dialog, "Delete project?");
    assert!(dialog.accesskit_node().data().is_modal());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().store, before, "Escape deletes nothing");
    assert!(h.get_by_label("Delete project…").is_focused());
    h.key_press(Key::Enter);
    h.run_steps(5);
    h.get_by_label("Delete project").click();
    h.run_steps(5);
    assert!(h.state().store.project(garden).is_none());
    assert_eq!(h.state().store.projects().len(), 2);
    let selected = h.state().selected.unwrap();
    assert_eq!(
        h.state().store.project(selected).unwrap().name,
        "Kitchen refit"
    );
    assert!(
        h.get_by_role_and_label(Role::Button, "Kitchen refit")
            .is_focused()
    );
    h.get_by_role_and_label(Role::Status, "Information: Project deleted")
        .get_by_label("Undo")
        .click();
    h.run_steps(4);
    assert_eq!(h.state().store, before);
    assert_eq!(h.state().selected, Some(garden));
    assert!(
        h.get_by_role_and_label(Role::Button, "Garden renovation")
            .is_focused()
    );
}

#[test]
fn without_projects_the_welcome_state_leads_to_the_project_field() {
    let mut h = super::harness_with(
        App::empty(FIXED_TODAY),
        dutchy_egui::Accent::Orange,
        dutchy_egui::Mode::Light,
        1.0,
    );
    h.get_by_role_and_label(Role::Heading, "No projects yet");
    h.key_press(Key::N);
    h.run_steps(3);
    assert!(!h.state().editor.open);
    assert!(h.get_by_label("New project name").is_focused());
    h.key_press(Key::Escape);
    h.run();
    h.get_by_label("Create project").click();
    h.run_steps(3);
    assert!(h.get_by_label("New project name").is_focused());
    h.check_accessibility();
}
