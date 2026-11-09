//! Projects with open-task Badges, project creation, deletion and file transfer.

use crate::{
    app::{App, Focus},
    commands::{self, Action},
};
use dutchy_egui::{
    Badge, BadgeVariant, Button, ButtonVariant, EmptyState, EmptyStateVariant, Input, Text,
    TextColor,
    tokens::{ControlSize, space},
};
use egui::{Key, Ui};

/// Sidebar width in points.
pub const WIDTH: f32 = 296.0;

/// Draw the sidebar panel.
pub fn show(ui: &mut Ui, app: &mut App) {
    crate::support::shell::sidebar(ui, "todo-sidebar", WIDTH, |ui| {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            actions(ui, app);
            // Everything above the pinned actions scrolls, so a short window
            // never lets the two regions overlap.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("projects")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.add(Text::label("Projects").color(TextColor::Primary));
                        ui.add_space(space::MD);
                        creator(ui, app);
                        ui.add_space(space::LG);
                        projects(ui, app);
                    });
            });
        });
    });
}

fn projects(ui: &mut Ui, app: &mut App) {
    if app.store.projects().is_empty() {
        EmptyState::new("no-projects", "No projects")
            .variant(EmptyStateVariant::Table)
            .description("Name a project above to start a list.")
            .show(ui);
        return;
    }
    let rows: Vec<_> = app
        .store
        .projects()
        .iter()
        .map(|project| {
            (
                project.id,
                project.name.clone(),
                app.store.counts(project.id, app.today).active(),
            )
        })
        .collect();
    for (id, name, open) in rows {
        let selected = app.selected == Some(id);
        // Bound the row height: a bare right-to-left layout would take the
        // whole remaining height and center one row in it.
        let size = egui::vec2(ui.available_width(), ControlSize::Small.height());
        let response = ui
            .allocate_ui_with_layout(
                size,
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    let variant = if open == 0 {
                        BadgeVariant::Muted
                    } else if selected {
                        BadgeVariant::Primary
                    } else {
                        BadgeVariant::Outline
                    };
                    ui.add(Badge::new(open.to_string()).variant(variant));
                    let variant = if selected {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    };
                    ui.add(
                        Button::new(name)
                            .variant(variant)
                            .size(ControlSize::Small)
                            .full_width(true),
                    )
                },
            )
            .inner;
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_description(format!("{open} open tasks"));
            node.set_selected(selected);
        });
        if app.claim_focus(Focus::Project(id)) {
            response.request_focus();
        }
        if response.clicked() {
            app.select_project(id);
        }
    }
}

fn creator(ui: &mut Ui, app: &mut App) {
    let input = ui.add(
        Input::new("new-project", &mut app.new_project)
            .accessible_name("New project name")
            .placeholder("New project")
            .size(ControlSize::Small),
    );
    if app.claim_focus(Focus::NewProject) {
        input.request_focus();
    }
    let submitted = input.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
    ui.add_space(space::SM);
    let add = ui.add(
        Button::new("Add project")
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .full_width(true),
    );
    if (submitted || add.clicked()) && !app.new_project.trim().is_empty() {
        app.add_project();
    }
}

/// Pinned under the project list, laid out bottom-up.
fn actions(ui: &mut Ui, app: &mut App) {
    ui.horizontal(|ui| {
        let busy = app.io.busy();
        if ui
            .add(
                Button::new("Import…")
                    .variant(ButtonVariant::Outline)
                    .size(ControlSize::Small)
                    .enabled(!busy),
            )
            .clicked()
        {
            commands::execute(app, Action::Import, None, ui.ctx());
        }
        if ui
            .add(
                Button::new("Export…")
                    .variant(ButtonVariant::Outline)
                    .size(ControlSize::Small)
                    .enabled(!busy),
            )
            .clicked()
        {
            commands::execute(app, Action::Export, None, ui.ctx());
        }
    });
    ui.add_space(space::SM);
    let delete = ui.add(
        Button::new("Delete project…")
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small)
            .full_width(true)
            .enabled(app.selected.is_some()),
    );
    if delete.clicked()
        && let Some(project) = app.selected
    {
        app.confirm.ask(project, Some(delete.id));
    }
    ui.add_space(space::LG);
    crate::support::shell::divider(ui);
    ui.add_space(space::LG);
}
