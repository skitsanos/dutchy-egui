//! The task view: heading, search and filter, overview, the row list and the
//! empty state for the current filter.

use crate::{
    app::{App, Focus},
    commands::{self, Action},
    row::{self, RowAction},
    store::{Filter, ProjectId, Task},
    support::shell,
};
use dutchy_egui::{
    EmptyState, EmptyStateIcon, EmptyStateVariant, Kbd, Progress, ProgressVariant, SearchInput,
    Segmented, SplitButton, Stat, StatsGrid, StatsLayout, Text, TextColor, TextSize,
    tokens::{ControlSize, ProgressSize, border, control, space},
};
use egui::{Id, Rect, Ui, accesskit::Role};

/// Draw the central panel.
pub fn show(ui: &mut Ui, app: &mut App) {
    let Some(project) = app.selected else {
        no_projects(ui, app);
        return;
    };
    heading(ui, app, project);
    ui.add_space(space::LG);
    toolbar(ui, app);
    ui.add_space(space::LG);
    overview(ui, app, project);
    ui.add_space(space::XL);
    list(ui, app, project);
}

fn heading(ui: &mut Ui, app: &mut App, project: ProjectId) {
    let name = app
        .store
        .project(project)
        .map_or_else(String::new, |project| project.name.clone());
    // Reserve the actions' width so the heading wraps instead of colliding.
    let hint = app
        .commands
        .registry
        .shortcut_label(ui.ctx(), &Action::New)
        .unwrap_or_else(|| "Unbound".into());
    let alternatives = [Action::Import, Action::Export]
        .iter()
        .filter_map(|key| app.commands.registry.menu_item(ui.ctx(), key))
        .collect::<Vec<_>>();
    let actions = ControlSize::Small.height()
        + shell::button_width(ui, "New task", ControlSize::Small)
        + shell::kbd_width(ui, &hint)
        + 3.0 * space::SM;
    ui.horizontal(|ui| {
        let width = (ui.available_width() - actions).max(space::HERO_LARGE);
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(width);
                ui.add(Text::heading(name).size(TextSize::Xl3));
            },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let output = SplitButton::new(
                "new-task",
                "New task",
                &mut app.commands.split,
                &alternatives,
            )
            .size(ControlSize::Small)
            .alternatives_name("More task actions")
            .show(ui);
            let button = output.primary;
            if app.claim_focus(Focus::NewTask) {
                button.request_focus();
            }
            if button.clicked() {
                commands::execute(app, Action::New, Some(button.id), ui.ctx());
            }
            if let Some(action) = output.action {
                commands::execute(app, action, Some(button.id), ui.ctx());
            }
            ui.add(Kbd::new(hint));
        });
    });
}

/// Search beside the filter while both fit at a useful width; stacked otherwise.
/// Segmented shrinks instead of wrapping, so the host decides the break.
fn toolbar(ui: &mut Ui, app: &mut App) {
    const SEARCH_MIN: f32 = 240.0;
    const SEARCH_MAX: f32 = 420.0;
    let filter_width = filter_width(ui);
    let available = ui.available_width();
    if available >= SEARCH_MIN + space::MD + filter_width {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            let width = (available - filter_width - space::MD).min(SEARCH_MAX);
            search(ui, app, width);
            filter(ui, app);
        });
    } else {
        search(ui, app, available);
        ui.add_space(space::SM);
        filter(ui, app);
    }
}

fn search(ui: &mut Ui, app: &mut App, width: f32) {
    let search = ui.add(
        SearchInput::new("task-search", &mut app.query)
            .accessible_name("Search tasks")
            .placeholder("Search tasks…")
            .size(ControlSize::Small)
            .width(width),
    );
    if app.claim_focus(Focus::Search) {
        search.request_focus();
    }
}

fn filter(ui: &mut Ui, app: &mut App) {
    ui.add(
        Segmented::new("task-filter", &mut app.filter, &Filter::options())
            .accessible_name("Show tasks")
            .size(ControlSize::Small),
    );
}

/// Natural width of the filter: three equal segments around the widest label.
fn filter_width(ui: &Ui) -> f32 {
    let theme = dutchy_egui::Theme::from_context(ui.ctx());
    let widest = Filter::ALL
        .iter()
        .map(|filter| {
            let job = Text::new(filter.name())
                .size(ControlSize::Small.text_size())
                .weight(dutchy_egui::Weight::Bold)
                .layout_job(&theme, f32::INFINITY);
            ui.fonts_mut(|fonts| fonts.layout_job(job)).size().x
        })
        .fold(0.0, f32::max);
    (widest + 2.0 * control::SEGMENT_PADDING_X) * Filter::ALL.len() as f32 + 2.0 * border::STANDARD
}

fn overview(ui: &mut Ui, app: &App, project: ProjectId) {
    let counts = app.store.counts(project, app.today);
    let stats = [
        Stat::new("Open", counts.active().to_string()),
        Stat::new("Done", counts.done.to_string()),
        Stat::new("Overdue", counts.overdue.to_string()),
        Stat::new("Due today", counts.due_today.to_string()),
    ];
    ui.add(
        StatsGrid::new("overview", &stats)
            .layout(StatsLayout::Compact)
            .accessible_name("Project overview"),
    );
    ui.add_space(space::MD);
    let variant = if counts.total > 0 && counts.done == counts.total {
        ProgressVariant::Success
    } else {
        ProgressVariant::Primary
    };
    ui.add(
        Progress::new(counts.percent())
            .label("Completion")
            .show_value(true)
            .size(ProgressSize::Small)
            .variant(variant),
    );
}

/// The scrolling region holds either the rows or the empty state, so both
/// stay reachable in short windows.
fn list(ui: &mut Ui, app: &mut App, project: ProjectId) {
    let tasks = app.visible_tasks();
    let ids: Vec<_> = tasks.iter().map(|task| task.id).collect();
    let action = row::keys(ui, &ids);
    egui::ScrollArea::vertical()
        .id_salt(("tasks", project.0))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if tasks.is_empty() {
                empty(ui, app);
            } else {
                rows(ui, app, &tasks, project);
            }
        });
    match action {
        Some(RowAction::Edit(id)) => app.edit_task(id, Some(row::id(id))),
        Some(RowAction::Toggle(id)) => app.toggle_task(id),
        Some(RowAction::Delete(id)) => {
            let next = app.neighbor_of(id);
            app.delete_task(id, next);
        }
        None => {}
    }
}

fn rows(ui: &mut Ui, app: &mut App, tasks: &[Task], project: ProjectId) {
    let today = app.today;
    let result = ui.scope_builder(
        egui::UiBuilder::new().id_salt(("task-list", project.0)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut focused = None;
            for (index, task) in tasks.iter().enumerate() {
                let outcome = row::show(ui, task, today);
                if app.claim_focus(Focus::Row(task.id)) || outcome.response.clicked() {
                    row::focus(ui.ctx(), task.id);
                }
                if let Some(id) = outcome.focused_control() {
                    focused = Some(((id, index), outcome.response.rect));
                }
                if let Some(done) = outcome.done {
                    app.set_task_done(task.id, done);
                }
                if outcome.edit {
                    app.edit_task(task.id, Some(outcome.response.id));
                }
            }
            reveal_focused_row(ui, focused);
        },
    );
    result
        .response
        .ctx
        .accesskit_node_builder(result.response.id, |node| {
            node.set_role(Role::List);
            node.set_label("Tasks");
        });
}

/// Reveal new focus or a reordered row. Ordinary scrolling leaves the focused
/// row in place even when it moves out of the viewport.
fn reveal_focused_row(ui: &Ui, focused: Option<((Id, usize), Rect)>) {
    let id = ui.make_persistent_id("focused-row");
    let current = focused.map(|(key, _)| key);
    let previous = ui.data_mut(|data| {
        let previous = data.get_temp::<(Id, usize)>(id);
        if let Some(current) = current {
            data.insert_temp(id, current);
        } else {
            data.remove::<(Id, usize)>(id);
        }
        previous
    });
    if let Some((key, rect)) = focused
        && previous != Some(key)
        && !ui.clip_rect().contains_rect(rect)
    {
        ui.scroll_to_rect_animation(rect, None, egui::style::ScrollAnimation::none());
    }
}

fn empty(ui: &mut Ui, app: &mut App) {
    if !app.query.trim().is_empty() {
        let result = EmptyState::new("no-matches", "No matching tasks")
            .variant(EmptyStateVariant::Search)
            .description("Try another word, or clear the search to see every task.")
            .action("Clear search")
            .announce(true)
            .show(ui);
        if result.action_clicked {
            app.query.clear();
            app.focus = Some(Focus::Search);
        }
        return;
    }
    let (id, title, description, action) = match app.filter() {
        Filter::All => (
            "no-tasks",
            "No tasks yet",
            "Add the first task to this project.",
            "Add a task",
        ),
        Filter::Active => (
            "all-done",
            "All caught up",
            "Every task in this project is done.",
            "Show all tasks",
        ),
        Filter::Done => (
            "none-done",
            "Nothing completed yet",
            "Finished tasks appear here.",
            "Show all tasks",
        ),
    };
    let result = EmptyState::new(id, title)
        .icon(EmptyStateIcon::Inbox)
        .description(description)
        .action(action)
        .announce(true)
        .show(ui);
    if result.action_clicked {
        if app.filter() == Filter::All {
            app.create_task(None);
        } else {
            app.filter = Some(Filter::All);
        }
    }
}

fn no_projects(ui: &mut Ui, app: &mut App) {
    ui.add(Text::heading("DutchyTodo").size(TextSize::Xl3));
    ui.add(Text::new("Lists, with the state in your hands.").color(TextColor::Muted));
    let result = EmptyState::new("welcome", "No projects yet")
        .icon(EmptyStateIcon::Inbox)
        .description("Create a project in the sidebar, or import a saved list.")
        .action("Create project")
        .show(ui);
    if result.action_clicked {
        app.focus = Some(Focus::NewProject);
    }
}
