//! The application-owned task store: projects, tasks, filters and undo data.
//!
//! Records are keyed by stable ids and kept in creation order. Nothing here
//! draws anything; the views read this model and call its operations.

mod kinds;

pub use kinds::{Filter, Priority};

use dutchy_egui::calendar::Date;
use std::collections::BTreeMap;

/// Stable key of a project.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectId(pub u64);

/// Stable key of a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

/// A named list of tasks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// Stable key.
    pub id: ProjectId,
    /// Visible name.
    pub name: String,
}

/// One item of work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    /// Stable key.
    pub id: TaskId,
    /// Owning project.
    pub project: ProjectId,
    /// One-line title.
    pub title: String,
    /// Free-form notes.
    pub notes: String,
    /// Optional due date.
    pub due: Option<Date>,
    /// Urgency.
    pub priority: Priority,
    /// Completed.
    pub done: bool,
}

/// Editable fields of a task, as the editor drafts them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskDraft {
    /// One-line title; a blank title is rejected.
    pub title: String,
    /// Free-form notes.
    pub notes: String,
    /// Optional due date.
    pub due: Option<Date>,
    /// Urgency.
    pub priority: Priority,
}

impl TaskDraft {
    /// Copy the editable fields of an existing task.
    pub fn from_task(task: &Task) -> Self {
        Self {
            title: task.title.clone(),
            notes: task.notes.clone(),
            due: task.due,
            priority: task.priority,
        }
    }
}

/// Counts for one project or one view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// All tasks.
    pub total: usize,
    /// Completed tasks.
    pub done: usize,
    /// Open tasks due before today.
    pub overdue: usize,
    /// Open tasks due today.
    pub due_today: usize,
}

impl Counts {
    /// Open tasks.
    pub const fn active(self) -> usize {
        self.total - self.done
    }

    /// Completion in percent, zero for an empty set.
    pub fn percent(self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * self.done as f64 / self.total as f64
        }
    }
}

/// Projects and tasks in creation order, keyed by stable ids.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
    projects: Vec<Project>,
    tasks: BTreeMap<TaskId, Task>,
    next_project: u64,
    next_task: u64,
}

impl Store {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Projects in creation order.
    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    /// One project.
    pub fn project(&self, id: ProjectId) -> Option<&Project> {
        self.projects.iter().find(|project| project.id == id)
    }

    /// One task.
    pub fn task(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Every task in creation order.
    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values()
    }

    /// Create a project; blank names become "Untitled project".
    pub fn add_project(&mut self, name: &str) -> ProjectId {
        let id = ProjectId(self.next_project);
        self.next_project += 1;
        let name = name.trim();
        self.projects.push(Project {
            id,
            name: if name.is_empty() {
                "Untitled project".into()
            } else {
                name.to_owned()
            },
        });
        id
    }

    /// Remove a project with its tasks, returning them for undo.
    pub fn remove_project(&mut self, id: ProjectId) -> Option<RemovedProject> {
        let index = self.projects.iter().position(|project| project.id == id)?;
        let project = self.projects.remove(index);
        let tasks = self
            .tasks
            .values()
            .filter(|task| task.project == id)
            .map(|task| task.id)
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|task| self.tasks.remove(&task))
            .collect();
        Some(RemovedProject {
            project,
            tasks,
            index,
        })
    }

    /// Put a removed project and its tasks back at their former position.
    pub fn restore_project(&mut self, removed: RemovedProject) {
        let index = removed.index.min(self.projects.len());
        self.projects.insert(index, removed.project);
        for task in removed.tasks {
            self.tasks.insert(task.id, task);
        }
    }

    /// Create a task from a draft. Blank titles become "Untitled task".
    pub fn add_task(&mut self, project: ProjectId, draft: TaskDraft) -> TaskId {
        let id = TaskId(self.next_task);
        self.next_task += 1;
        let title = draft.title.trim();
        self.tasks.insert(
            id,
            Task {
                id,
                project,
                title: if title.is_empty() {
                    "Untitled task".into()
                } else {
                    title.to_owned()
                },
                notes: draft.notes.trim_end().to_owned(),
                due: draft.due,
                priority: draft.priority,
                done: false,
            },
        );
        id
    }

    /// Apply a draft to an existing task; reports whether it exists.
    pub fn update_task(&mut self, id: TaskId, draft: TaskDraft) -> bool {
        let Some(task) = self.tasks.get_mut(&id) else {
            return false;
        };
        let title = draft.title.trim();
        if !title.is_empty() {
            task.title = title.to_owned();
        }
        task.notes = draft.notes.trim_end().to_owned();
        task.due = draft.due;
        task.priority = draft.priority;
        true
    }

    /// Set completion; returns the new state when the task exists.
    pub fn set_done(&mut self, id: TaskId, done: bool) -> Option<bool> {
        let task = self.tasks.get_mut(&id)?;
        task.done = done;
        Some(done)
    }

    /// Flip completion; returns the new state when the task exists.
    pub fn toggle_done(&mut self, id: TaskId) -> Option<bool> {
        let done = !self.tasks.get(&id)?.done;
        self.set_done(id, done)
    }

    /// Remove a task, returning it for undo.
    pub fn remove_task(&mut self, id: TaskId) -> Option<Task> {
        self.tasks.remove(&id)
    }

    /// Put a removed task back under its original key.
    pub fn restore_task(&mut self, task: Task) {
        self.next_task = self.next_task.max(task.id.0 + 1);
        self.tasks.insert(task.id, task);
    }

    /// Insert a project with an explicit id, used by import.
    pub(crate) fn insert_project(&mut self, project: Project) {
        self.next_project = self.next_project.max(project.id.0 + 1);
        self.projects.retain(|existing| existing.id != project.id);
        self.projects.push(project);
    }

    /// Insert a task with an explicit id, used by import.
    pub(crate) fn insert_task(&mut self, task: Task) {
        self.restore_task(task);
    }

    /// Counts for one project; `today` decides overdue and due-today.
    pub fn counts(&self, project: ProjectId, today: Date) -> Counts {
        count(
            self.tasks.values().filter(|task| task.project == project),
            today,
        )
    }

    /// Tasks of a project that pass the filter and a case-insensitive title
    /// or notes search, ordered by open-before-done, priority, due date and age.
    pub fn visible(&self, project: ProjectId, filter: Filter, query: &str) -> Vec<&Task> {
        let needle = query.trim().to_lowercase();
        let mut tasks: Vec<&Task> = self
            .tasks
            .values()
            .filter(|task| task.project == project && filter.admits(task))
            .filter(|task| {
                needle.is_empty()
                    || task.title.to_lowercase().contains(&needle)
                    || task.notes.to_lowercase().contains(&needle)
            })
            .collect();
        tasks.sort_by_key(|task| {
            (
                task.done,
                std::cmp::Reverse(task.priority),
                task.due.is_none(),
                task.due,
                task.id,
            )
        });
        tasks
    }
}

fn count<'a>(tasks: impl Iterator<Item = &'a Task>, today: Date) -> Counts {
    let mut counts = Counts::default();
    for task in tasks {
        counts.total += 1;
        if task.done {
            counts.done += 1;
        } else if let Some(due) = task.due {
            if due < today {
                counts.overdue += 1;
            } else if due == today {
                counts.due_today += 1;
            }
        }
    }
    counts
}

/// A removed project with its tasks and former position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedProject {
    /// The project record.
    pub project: Project,
    /// Its tasks in creation order.
    pub tasks: Vec<Task>,
    /// Former position in the project list.
    pub index: usize,
}
