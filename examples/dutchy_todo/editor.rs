//! The task editor Drawer and the project-deletion Modal. Both bind
//! application-owned open flags and return focus to the control that opened them.

use crate::{
    app::{App, Focus},
    store::{Priority, ProjectId, Task, TaskDraft, TaskId},
};
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, DatePicker, Drawer, DrawerWidth, Field, Input,
    Modal, ModalWidth, OverlayOptions, Segmented, Text, TextColor, Textarea,
    calendar::Date,
    tokens::{ControlSize, space},
    validation::{FieldValidation, Rule},
};
use egui::{Context, Id, Key, Ui};

/// Which task the drawer edits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Target {
    /// A task that does not exist yet.
    #[default]
    New,
    /// An existing task.
    Existing(TaskId),
}

/// Drawer state. The draft survives dismissal until the next open.
#[derive(Default)]
pub struct Editor {
    /// Host-owned visibility.
    pub open: bool,
    /// New or existing task.
    pub target: Target,
    /// Fields being edited.
    pub draft: TaskDraft,
    /// Control to focus after closing.
    pub return_focus: Option<Id>,
    /// Focus the title field on the next frame.
    pub focus_title: bool,
    /// Title validation shown by the Field.
    pub validation: FieldValidation,
    /// A save failure that must remain visible inside the drawer.
    pub save_error: Option<&'static str>,
}

impl Editor {
    /// Start a blank task.
    pub fn open_new(&mut self, return_focus: Option<Id>) {
        self.start(Target::New, TaskDraft::default(), return_focus);
    }

    /// Edit an existing task.
    pub fn open_existing(&mut self, task: &Task, return_focus: Option<Id>) {
        self.start(
            Target::Existing(task.id),
            TaskDraft::from_task(task),
            return_focus,
        );
    }

    fn start(&mut self, target: Target, draft: TaskDraft, return_focus: Option<Id>) {
        self.open = true;
        self.target = target;
        self.draft = draft;
        self.return_focus = return_focus;
        self.focus_title = true;
        self.validation.clear();
        self.save_error = None;
    }
}

/// Project-deletion confirmation state.
#[derive(Default)]
pub struct Confirm {
    /// Host-owned visibility.
    pub open: bool,
    /// Project under confirmation.
    pub project: Option<ProjectId>,
    /// Control to focus after closing.
    pub return_focus: Option<Id>,
}

impl Confirm {
    /// Ask before deleting a project.
    pub fn ask(&mut self, project: ProjectId, return_focus: Option<Id>) {
        self.open = true;
        self.project = Some(project);
        self.return_focus = return_focus;
    }
}

/// Footer activation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Save,
    Delete,
}

/// Show the editor drawer every frame; it renders only while open.
pub fn drawer(app: &mut App, ctx: &Context) {
    let today = app.today;
    let Editor {
        open,
        target,
        draft,
        return_focus,
        focus_title,
        validation,
        save_error,
    } = &mut app.editor;
    let existing = matches!(target, Target::Existing(_));
    let title = if existing { "Edit task" } else { "New task" };
    let result = Drawer::new("task-editor", open, title)
        .width(DrawerWidth::Medium)
        .options(OverlayOptions::default().return_focus(*return_focus))
        .show_with_footer(
            ctx,
            |ui| {
                if let Some(message) = *save_error {
                    Alert::new("task-save-error", message)
                        .title("Task limit reached")
                        .variant(AlertVariant::Error)
                        .show(ui);
                    ui.add_space(space::MD);
                }
                fields(ui, draft, validation, focus_title, today)
            },
            |ui| footer(ui, existing),
        );
    let Some(result) = result else {
        return;
    };
    let (submitted, action) = result.inner;
    match action {
        Some(Action::Save) => save(app),
        Some(Action::Delete) => delete(app),
        None if submitted => save(app),
        None => {}
    }
}

fn fields(
    ui: &mut Ui,
    draft: &mut TaskDraft,
    validation: &mut FieldValidation,
    focus_title: &mut bool,
    today: Date,
) -> bool {
    let title = Field::new("Title")
        .required(true)
        .validation(validation.state())
        .show(ui, |ui| {
            ui.add(Input::new("task-title", &mut draft.title).placeholder("What needs doing?"))
        });
    if *focus_title {
        title.request_focus();
        *focus_title = false;
    }
    let submitted = title.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
    ui.add_space(space::MD);
    Field::new("Notes").show(ui, |ui| {
        ui.add(
            Textarea::new("task-notes", &mut draft.notes)
                .placeholder("Details, links, next steps…")
                .rows(4),
        )
    });
    ui.add_space(space::MD);
    Field::new("Due date")
        .help("Leave empty for no date.")
        .show(ui, |ui| {
            ui.add(DatePicker::new("task-due", &mut draft.due).today(today))
        });
    ui.add_space(space::MD);
    let mut priority = Some(draft.priority);
    Field::new("Priority").show(ui, |ui| {
        ui.add(
            Segmented::new("task-priority", &mut priority, &Priority::options())
                .size(ControlSize::Small)
                .full_width(true),
        )
    });
    if let Some(priority) = priority {
        draft.priority = priority;
    }
    ui.add_space(space::MD);
    ui.add(
        Text::new("Enter in the title saves. Escape closes without saving.")
            .color(TextColor::Muted),
    );
    submitted
}

fn footer(ui: &mut Ui, existing: bool) -> Option<Action> {
    ui.horizontal_wrapped(|ui| {
        let mut action = None;
        if ui.add(Button::new("Save task")).clicked() {
            action = Some(Action::Save);
        }
        if ui
            .add(Button::new("Cancel").variant(ButtonVariant::Outline))
            .clicked()
        {
            ui.close();
        }
        if existing
            && ui
                .add(Button::new("Delete task").variant(ButtonVariant::Destructive))
                .clicked()
        {
            action = Some(Action::Delete);
        }
        action
    })
    .inner
}

fn save(app: &mut App) {
    app.editor.save_error = None;
    let rules = [Rule::required("Enter a title before saving.")];
    if !app
        .editor
        .validation
        .validate(&app.editor.draft.title, &rules)
    {
        app.editor.focus_title = true;
        return;
    }
    let draft = app.editor.draft.clone();
    match app.editor.target {
        Target::New => {
            if let Some(project) = app.selected {
                match app.store.add_task(project, draft) {
                    Ok(id) => app.focus = Some(Focus::Row(id)),
                    Err(_) => {
                        app.editor.save_error = Some("No more tasks can be added to this list.");
                        return;
                    }
                }
            }
        }
        Target::Existing(id) => {
            app.store.update_task(id, draft);
        }
    }
    app.editor.open = false;
}

fn delete(app: &mut App) {
    app.editor.open = false;
    if let Target::Existing(id) = app.editor.target {
        let next = app.neighbor_of(id);
        app.delete_task(id, next);
    }
}

/// Show the deletion dialog every frame; it renders only while open.
pub fn confirm(app: &mut App, ctx: &Context) {
    let project = app
        .confirm
        .project
        .and_then(|id| app.store.project(id).cloned());
    let count = project
        .as_ref()
        .map_or(0, |project| app.store.counts(project.id, app.today).total);
    let name = project.as_ref().map_or("", |project| project.name.as_str());
    let message = format!(
        "Delete \"{name}\" and its {count} tasks? You can undo this from the notification."
    );
    let result = Modal::new("delete-project", &mut app.confirm.open, "Delete project?")
        .width(ModalWidth::Medium)
        .options(OverlayOptions::default().return_focus(app.confirm.return_focus))
        .show_with_footer(
            ctx,
            |ui| {
                ui.add(Text::new(message));
            },
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(Button::new("Cancel").variant(ButtonVariant::Outline))
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.add(Button::new("Delete project").variant(ButtonVariant::Destructive))
                        .clicked()
                })
                .inner
            },
        );
    if result.is_some_and(|result| result.inner.1)
        && let Some(project) = project
    {
        app.confirm.open = false;
        app.remove_project(project.id);
    }
}
