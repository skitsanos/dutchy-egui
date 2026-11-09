//! Operations the views call: task and project changes, notifications with
//! undo, and finished file jobs. Every change goes through here so the toast,
//! focus and selection rules live in one place.

use crate::{
    app::{App, Focus},
    history::{Undo, UndoRequest},
    io::Outcome,
    store::{ProjectId, Store, TaskId},
};
use dutchy_egui::{Toast, ToastVariant};
use egui::Id;
use std::time::Duration;

impl App {
    /// Open the editor for a new task in the selected project.
    pub fn create_task(&mut self, return_focus: Option<Id>) {
        if self.selected.is_some() {
            self.editor.open_new(return_focus);
        } else {
            self.focus = Some(Focus::NewProject);
        }
    }

    /// Open the editor for an existing task.
    pub fn edit_task(&mut self, id: TaskId, return_focus: Option<Id>) {
        if let Some(task) = self.store.task(id) {
            self.editor.open_existing(task, return_focus);
        }
    }

    /// Set completion from the checkbox; completing offers undo.
    pub fn set_task_done(&mut self, id: TaskId, done: bool) {
        if self.store.set_done(id, done) == Some(true) {
            self.completed(id);
        }
    }

    /// Flip completion from the keyboard.
    pub fn toggle_task(&mut self, id: TaskId) {
        if self.store.toggle_done(id) == Some(true) {
            self.completed(id);
        }
    }

    fn completed(&mut self, id: TaskId) {
        self.notify(
            "Task completed",
            "Marked as done.",
            ToastVariant::Success,
            Some(Undo::Complete(id)),
        );
    }

    /// Remove a task, move focus to `next` and offer undo.
    pub fn delete_task(&mut self, id: TaskId, next: Focus) {
        if let Some(task) = self.store.remove_task(id) {
            self.focus = Some(next);
            self.notify(
                "Task deleted",
                format!("Removed \"{}\".", task.title),
                ToastVariant::Info,
                Some(Undo::Delete(task)),
            );
        }
    }

    /// Create a project from the sidebar draft and show it.
    pub fn add_project(&mut self) {
        let id = match self.store.add_project(&self.new_project) {
            Ok(id) => id,
            Err(_) => {
                self.notify(
                    "Project limit reached",
                    "No more projects can be added to this list.",
                    ToastVariant::Error,
                    None,
                );
                return;
            }
        };
        self.new_project.clear();
        self.selected = Some(id);
        self.focus = Some(Focus::Project(id));
    }

    /// Remove a project with its tasks after confirmation and offer undo.
    pub fn remove_project(&mut self, id: ProjectId) {
        let Some(removed) = self.store.remove_project(id) else {
            return;
        };
        self.ensure_selection();
        self.focus = Some(self.selected.map_or(Focus::NewProject, Focus::Project));
        let message = format!(
            "Removed \"{}\" with {} tasks.",
            removed.project.name,
            removed.tasks.len()
        );
        self.notify(
            "Project deleted",
            message,
            ToastVariant::Info,
            Some(Undo::DeleteProject(removed)),
        );
    }

    /// Queue a notification; `undo` adds a single-use action.
    pub fn notify(
        &mut self,
        title: &str,
        message: impl Into<String>,
        variant: ToastVariant,
        undo: Option<Undo>,
    ) {
        let mut toast = Toast::new(message).title(title).variant(variant);
        if let Some(undo) = undo {
            toast = toast
                .action("Undo", self.history.request(undo))
                .timeout(Some(Duration::from_secs(8)));
        }
        self.toasts.push(toast);
    }

    pub(crate) fn undo(&mut self, request: UndoRequest) {
        let Some(undo) = self.history.take(request) else {
            return;
        };
        match undo {
            Undo::Delete(task) => {
                let id = task.id;
                if self.store.restore_task(task).is_ok() {
                    self.focus = Some(Focus::Row(id));
                }
            }
            Undo::Complete(id) => {
                self.store.set_done(id, false);
                self.focus = Some(Focus::Row(id));
            }
            Undo::DeleteProject(removed) => {
                let id = removed.project.id;
                if self.store.restore_project(removed).is_ok() {
                    self.selected = Some(id);
                    self.focus = Some(Focus::Project(id));
                }
            }
            Undo::Import(previous) => {
                self.replace_store(previous);
            }
        }
    }

    pub(crate) fn finish_jobs(&mut self) {
        if let Some(outcome) = self.io.poll() {
            self.finish_job(outcome);
        }
    }

    pub(crate) fn finish_job(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Exported { name, tasks } => self.notify(
                "Exported",
                format!("Wrote {tasks} tasks to {name}."),
                ToastVariant::Success,
                None,
            ),
            Outcome::Imported { name, store } => {
                let previous = self.replace_store(store);
                let tasks = self.store.tasks().count();
                self.notify(
                    "Imported",
                    format!("Loaded {tasks} tasks from {name}."),
                    ToastVariant::Success,
                    Some(Undo::Import(previous)),
                );
            }
            Outcome::Failed(message) => {
                self.notify(
                    "Import or export failed",
                    message,
                    ToastVariant::Error,
                    None,
                );
            }
        }
    }

    fn replace_store(&mut self, store: Store) -> Store {
        self.history.invalidate();
        self.toasts.clear();
        self.editor = Default::default();
        self.confirm = Default::default();
        let previous = std::mem::replace(&mut self.store, store);
        self.ensure_selection();
        self.focus = Some(self.selected.map_or(Focus::NewProject, Focus::Project));
        previous
    }
}
