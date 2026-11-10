//! Shared application commands. Execution and file dialogs remain host concerns.
use crate::{
    app::{App, Focus},
    format,
};
use dutchy_egui::{
    Button, ButtonVariant, Command, CommandPalette, CommandRegistry, ControlSize, Drawer,
    DrawerWidth, Field, KeyRecorder, OverlayOptions, SearchState, Text, TextColor, tokens::space,
};
use egui::{Context, Id, Key, KeyboardShortcut, Modifiers, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    New,
    Search,
    Import,
    Export,
    Palette,
}
pub struct Commands {
    pub registry: CommandRegistry<Action>,
    pub palette: SearchState,
    pub settings: bool,
    pub split: bool,
    return_focus: Option<Id>,
}
impl Default for Commands {
    fn default() -> Self {
        let bind = |modifiers, key| KeyboardShortcut::new(modifiers, key);
        Self {
            registry: CommandRegistry::new(
                "todo-commands",
                [
                    Command::new(Action::New, "New task").shortcut(bind(Modifiers::NONE, Key::N)),
                    Command::new(Action::Search, "Search tasks")
                        .shortcut(bind(Modifiers::COMMAND, Key::F))
                        .allow_in_text(true),
                    Command::new(Action::Import, "Import tasks")
                        .shortcut(bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::I)),
                    Command::new(Action::Export, "Export tasks")
                        .shortcut(bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::E)),
                    Command::new(Action::Palette, "Command palette")
                        .shortcut(bind(Modifiers::COMMAND, Key::K))
                        .allow_in_text(true)
                        .allow_in_overlays(true),
                ],
            ),
            palette: SearchState::default(),
            settings: false,
            split: false,
            return_focus: None,
        }
    }
}
pub fn shortcuts(app: &mut App, ctx: &Context) {
    let selected = app.selected.is_some();
    let idle = !app.io.busy();
    app.commands
        .registry
        .get_mut(&Action::Search)
        .unwrap()
        .enabled = selected;
    for action in [Action::Import, Action::Export] {
        app.commands.registry.get_mut(&action).unwrap().enabled = idle;
    }
    app.commands
        .registry
        .get_mut(&Action::Palette)
        .unwrap()
        .enabled = !app.editor.open && !app.confirm.open && !app.commands.settings;
    for request in app.commands.registry.dispatch(ctx) {
        execute(app, request.value, None, ctx);
    }
}
pub fn execute(app: &mut App, action: Action, return_focus: Option<Id>, ctx: &Context) {
    if matches!(action, Action::Import | Action::Export) && app.io.busy() {
        return;
    }
    if app.commands.registry.request(&action).is_none() {
        return;
    }
    match action {
        Action::New => app.create_task(return_focus),
        Action::Search => app.focus = Some(Focus::Search),
        Action::Import => app.io.import(ctx),
        Action::Export => app
            .io
            .export(ctx, format::export(&app.store), app.store.tasks().count()),
        Action::Palette => {
            app.commands.split = false;
            app.commands.palette.open = !app.commands.palette.open;
        }
    }
}
pub fn controls(ui: &mut Ui, app: &mut App) {
    ui.horizontal_wrapped(|ui| {
        let palette = ui.add(
            Button::new("Commands")
                .size(ControlSize::Small)
                .variant(ButtonVariant::Outline),
        );
        if palette.clicked() {
            execute(app, Action::Palette, Some(palette.id), ui.ctx());
        }
        if let Some(hint) = app
            .commands
            .registry
            .shortcut_label(ui.ctx(), &Action::Palette)
        {
            ui.add(dutchy_egui::Kbd::new(hint));
        }
        let settings = ui.add(
            Button::new("Shortcuts")
                .size(ControlSize::Small)
                .variant(ButtonVariant::Ghost),
        );
        if settings.clicked() {
            app.commands.settings = true;
            app.commands.return_focus = Some(settings.id);
        }
    });
}
pub fn overlays(app: &mut App, ctx: &Context) {
    let items = app
        .commands
        .registry
        .menu_items(ctx)
        .into_iter()
        .filter(|item| item.value != Action::Palette)
        .collect::<Vec<_>>();
    if let Some(response) = CommandPalette::new("todo-palette", &mut app.commands.palette, &items)
        .shortcut(None)
        .show(ctx)
        && let Some(action) = response.action
    {
        execute(app, action, None, ctx);
    }
    let commands = &mut app.commands;
    Drawer::new(
        "todo-shortcuts",
        &mut commands.settings,
        "Keyboard shortcuts",
    )
    .width(DrawerWidth::Medium)
    .options(OverlayOptions::default().return_focus(commands.return_focus))
    .show(ctx, |ui| {
        ui.add(Text::new("Bindings belong to this application session.").color(TextColor::Muted));
        ui.add_space(space::MD);
        for action in [
            Action::New,
            Action::Search,
            Action::Import,
            Action::Export,
            Action::Palette,
        ] {
            let reserved = commands.registry.shortcuts_except(&action);
            let command = commands.registry.get_mut(&action).unwrap();
            let name = format!("{} shortcut", command.label);
            Field::new(name.clone()).show(ui, |ui| {
                KeyRecorder::new(action, &mut command.shortcut)
                    .reserved(&reserved)
                    .accessible_name(name)
                    .show(ui)
                    .response
            });
            ui.add_space(space::MD);
        }
    });
}
