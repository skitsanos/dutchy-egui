//! Application state and the frame: shell panels, overlays, toasts, shortcuts
//! and the single place where focus requests are queued.

use crate::{
    commands, data, editor,
    history::{History, UndoRequest},
    io::Io,
    sidebar,
    store::{Filter, ProjectId, Store, Task, TaskId},
    support::{
        capture::Capture,
        shell::{self, Hint},
        theme::ThemeSettings,
    },
    tasks,
};
use dutchy_egui::{
    Flex, Text, TextColor, TextSize, ToastEvent, ToastQueue, Toasts, calendar::Date, tokens::space,
};
use egui::{Context, Ui};

/// Where keyboard focus should land once the target control exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    /// A task row.
    Row(TaskId),
    /// The "New task" button.
    NewTask,
    /// A project button in the sidebar.
    Project(ProjectId),
    /// The new-project name input.
    NewProject,
    /// The task search field.
    Search,
}

/// The whole application. Views borrow it mutably for one frame.
pub struct App {
    /// Projects and tasks.
    pub store: Store,
    /// The host's calendar date; the library reads no clock.
    pub today: Date,
    /// Accent, mode and motion settings.
    pub theme: ThemeSettings,
    /// Project shown in the task list.
    pub selected: Option<ProjectId>,
    /// Segmented filter binding; always `Some` in practice.
    pub filter: Option<Filter>,
    /// Search text.
    pub query: String,
    /// Draft name for a new project.
    pub new_project: String,
    /// Drawer editor state.
    pub editor: editor::Editor,
    /// Project deletion confirmation.
    pub confirm: editor::Confirm,
    /// Notifications with undo payloads.
    pub toasts: ToastQueue<UndoRequest>,
    /// Identity of the store that can accept current Undo requests.
    pub history: History,
    /// Pending focus request, consumed by the view that owns the target.
    pub focus: Option<Focus>,
    /// Import and export jobs.
    pub io: Io,
    /// Shared bindings, palette and shortcut settings.
    pub commands: commands::Commands,
    /// Opt-in native frame capture for verification runs.
    pub capture: Capture,
}

impl App {
    /// The demo data set anchored on `today`.
    pub fn seeded(today: Date) -> Self {
        let (store, selected) = data::seeded(today);
        Self::with_store(store, Some(selected), today)
    }

    /// An application without any project.
    #[cfg(test)]
    pub fn empty(today: Date) -> Self {
        Self::with_store(Store::new(), None, today)
    }

    fn with_store(store: Store, selected: Option<ProjectId>, today: Date) -> Self {
        Self {
            store,
            today,
            theme: ThemeSettings::default(),
            selected,
            filter: Some(Filter::All),
            query: String::new(),
            new_project: String::new(),
            editor: editor::Editor::default(),
            confirm: editor::Confirm::default(),
            toasts: ToastQueue::new(),
            history: History::default(),
            focus: None,
            io: Io::default(),
            commands: commands::Commands::default(),
            capture: Capture::default(),
        }
    }

    /// Draw one frame.
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.finish_jobs();
        commands::shortcuts(self, &ctx);
        self.header(ui);
        self.status(ui);
        sidebar::show(ui, self);
        egui::CentralPanel::default()
            .frame(shell::frame(ui, space::XXL as i8))
            .show(ui, |ui| tasks::show(ui, self));
        editor::drawer(self, &ctx);
        editor::confirm(self, &ctx);
        commands::overlays(self, &ctx);
        self.notifications(&ctx);
        self.redirect_stale_focus();
        self.capture.tick(&ctx);
    }

    /// Current filter.
    pub fn filter(&self) -> Filter {
        self.filter.unwrap_or_default()
    }

    /// Take the pending focus request when it names `target`.
    pub fn claim_focus(&mut self, target: Focus) -> bool {
        if self.focus == Some(target) {
            self.focus = None;
            true
        } else {
            false
        }
    }

    /// Tasks the list shows right now.
    pub fn visible_tasks(&self) -> Vec<Task> {
        self.selected.map_or_else(Vec::new, |project| {
            self.store
                .visible(project, self.filter(), &self.query)
                .into_iter()
                .cloned()
                .collect()
        })
    }

    /// The row after `id` in the visible list, else the one before, else the
    /// "New task" button.
    pub fn neighbor_of(&self, id: TaskId) -> Focus {
        let visible = self.visible_tasks();
        let Some(index) = visible.iter().position(|task| task.id == id) else {
            return Focus::NewTask;
        };
        visible
            .get(index + 1)
            .or_else(|| index.checked_sub(1).and_then(|before| visible.get(before)))
            .map_or(Focus::NewTask, |task| Focus::Row(task.id))
    }

    /// Show the project.
    pub fn select_project(&mut self, id: ProjectId) {
        self.selected = Some(id);
    }

    /// Make sure a project is selected while any exists.
    pub fn ensure_selection(&mut self) {
        let valid = self
            .selected
            .is_some_and(|id| self.store.project(id).is_some());
        if !valid {
            self.selected = self.store.projects().first().map(|project| project.id);
        }
    }

    fn header(&mut self, ui: &mut Ui) {
        shell::header(ui, "todo-header", |ui| {
            Flex::row("brand").wrap(true).show(ui, |ui| {
                ui.add(Text::heading("DutchyTodo").size(TextSize::Xl2));
                ui.add_space(space::LG);
                ui.add(Text::label("Example application").color(TextColor::Muted));
                ui.add_space(space::LG);
                commands::controls(ui, self);
            });
            ui.add_space(space::SM);
            self.theme.controls(ui);
        });
    }

    fn status(&mut self, ui: &mut Ui) {
        let (name, counts) = self.selected.map_or_else(
            || ("None".to_owned(), Default::default()),
            |id| {
                (
                    self.store
                        .project(id)
                        .map_or_else(String::new, |project| project.name.clone()),
                    self.store.counts(id, self.today),
                )
            },
        );
        let segments = [
            ("Project", name),
            ("Open", counts.active().to_string()),
            ("Done", counts.done.to_string()),
            ("Shown", self.visible_tasks().len().to_string()),
        ];
        let new_hint = self
            .commands
            .registry
            .shortcut_label(ui.ctx(), &commands::Action::New)
            .unwrap_or_else(|| "Unbound".into());
        let hints = [
            Hint {
                key: &new_hint,
                action: "New task",
            },
            Hint {
                key: "Enter",
                action: "Edit",
            },
            Hint {
                key: "Space",
                action: "Toggle",
            },
            Hint {
                key: "Del",
                action: "Remove",
            },
            Hint {
                key: "Esc",
                action: "Close",
            },
        ];
        shell::status_strip(ui, "todo-status", &segments, &hints);
    }

    fn notifications(&mut self, ctx: &Context) {
        let result = Toasts::new("dutchy-todo-toasts", &mut self.toasts).show(ctx);
        for event in result.events {
            if let ToastEvent::Action { action, .. } = event {
                self.undo(action);
            }
        }
    }

    /// A focus request for a row that is filtered out or gone falls back to
    /// the "New task" button instead of lingering.
    fn redirect_stale_focus(&mut self) {
        if let Some(Focus::Row(id)) = self.focus
            && !self.visible_tasks().iter().any(|task| task.id == id)
        {
            self.focus = Some(Focus::NewTask);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
