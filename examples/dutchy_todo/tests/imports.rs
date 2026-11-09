//! Replacing a whole store invalidates its old actions and transient bindings.
use super::{harness, harness_with};
use crate::{
    app::{App, Focus},
    format::{self, HEADER},
    history::Undo,
    io::Outcome,
    store::{ProjectId, Store, TaskId},
    support::data::FIXED_TODAY,
};
use dutchy_egui::{Accent, Mode};
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;

fn imported(name: &str) -> Store {
    format::import(&format!("{HEADER}\nproject\t0\t{name}\nproject\t1\tOther\ntask\t0\t0\t0\tlow\t-\tDifferent task\t\ntask\t1\t0\t0\tlow\t-\tAnother task\t\n")).unwrap()
}

fn deliver(app: &mut App, store: Store) {
    app.finish_job(Outcome::Imported {
        name: "verification.txt".into(),
        store,
    });
}

#[test]
fn import_rejects_old_completion_deletion_and_project_requests() {
    let mut app = App::seeded(FIXED_TODAY);
    let complete = TaskId(0);
    let delete = TaskId(1);
    app.set_task_done(complete, true);
    let old_complete = app.history.request(Undo::Complete(complete));
    let deleted = app.store.task(delete).unwrap().clone();
    app.delete_task(delete, Focus::NewTask);
    let old_delete = app.history.request(Undo::Delete(deleted));
    let removed = app.store.clone().remove_project(ProjectId(1)).unwrap();
    app.remove_project(ProjectId(1));
    let old_project = app.history.request(Undo::DeleteProject(removed));
    let task = app.store.task(complete).unwrap().clone();
    app.editor.open_existing(&task, None);
    app.confirm.ask(task.project, None);
    app.focus = Some(Focus::Row(complete));
    let incoming = imported("Incoming");
    deliver(&mut app, incoming.clone());
    assert_eq!(app.toasts.len(), 1, "only the new import offers Undo");
    assert!(!app.editor.open && !app.confirm.open);
    assert_ne!(app.focus, Some(Focus::Row(complete)));
    for stale in [old_complete, old_delete, old_project] {
        app.undo(stale);
        assert_eq!(app.store, incoming, "an old event cannot change reused ids");
    }
}

#[test]
fn undo_import_restores_the_store_and_invalidates_actions_on_both_sides() {
    let mut h = harness();
    h.state_mut().set_task_done(TaskId(0), true);
    let old_complete = h.state().history.request(Undo::Complete(TaskId(0)));
    let original = h.state().store.clone();
    deliver(h.state_mut(), imported("Incoming"));
    h.state_mut().set_task_done(TaskId(0), true);
    let imported_complete = h.state().history.request(Undo::Complete(TaskId(0)));
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::Status, "Success: Task completed")
            .is_some(),
        "the current store's action is available before replacement"
    );
    h.get_by_role_and_label(Role::Status, "Success: Imported")
        .get_by_label("Undo")
        .click();
    h.run_steps(4);
    assert_eq!(h.state().store, original);
    assert!(h.state().toasts.is_empty());
    for stale in [old_complete, imported_complete] {
        h.state_mut().undo(stale);
        assert_eq!(
            h.state().store,
            original,
            "returning does not revive old actions"
        );
    }
    h.check_accessibility();
}

#[test]
fn a_second_import_rejects_the_older_import_undo() {
    let mut app = App::seeded(FIXED_TODAY);
    let original = app.store.clone();
    deliver(&mut app, imported("First"));
    let first_undo = app.history.request(Undo::Import(original));
    let second = imported("Second");
    deliver(&mut app, second.clone());
    app.undo(first_undo);
    assert_eq!(app.store, second);
    assert_eq!(app.toasts.len(), 1);
}

#[test]
fn failed_import_preserves_the_store_and_its_valid_undo() {
    let mut app = App::seeded(FIXED_TODAY);
    let original = app.store.clone();
    app.set_task_done(TaskId(0), true);
    let undo = app.history.request(Undo::Complete(TaskId(0)));
    let before = app.store.clone();
    app.finish_job(Outcome::Failed("Invalid identifier on line 2".into()));
    assert_eq!(app.store, before);
    app.undo(undo);
    assert_eq!(app.store, original);
}

#[test]
fn exhausted_creation_keeps_project_and_task_drafts_and_existing_records() {
    let id = u64::MAX - 1;
    let store = format::import(&format!(
        "{HEADER}\nproject\t{id}\tFull\ntask\t{id}\t{id}\t0\tlow\t-\tExisting task\t\n"
    ))
    .unwrap();
    let mut app = App::empty(FIXED_TODAY);
    deliver(&mut app, store.clone());
    app.new_project = "Keep this project draft".into();
    app.add_project();
    assert_eq!(app.new_project, "Keep this project draft");
    assert_eq!(app.store, store);
    app.create_task(None);
    let mut h = harness_with(app, Accent::Orange, Mode::Light, 1.0);
    h.get_by_label("Title").type_text("Keep this task draft");
    h.run();
    h.get_by_label("Save task").click();
    h.run_steps(4);
    assert!(h.state().editor.open);
    assert_eq!(h.state().editor.draft.title, "Keep this task draft");
    assert_eq!(h.state().store, store);
    h.get_by_role_and_label(Role::Alert, "Error: Task limit reached");
    h.check_accessibility();
}
