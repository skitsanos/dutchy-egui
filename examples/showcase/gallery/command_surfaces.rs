#[path = "command_surfaces/catalog.rs"]
mod catalog;
#[path = "command_surfaces/preview.rs"]
mod preview;
use dutchy_egui::{
    Button, ButtonVariant, CommandPalette, CommandRegistry, CommandRequest, ControlSize, Field,
    KeyRecorder, MenuBar, MenuBarItem, MenuBarState, SearchState, SplitButton, Switch, Text,
    TextColor, TextSize, Toolbar, ToolbarEvent, ToolbarItem, tokens::space,
};
use egui::Ui;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    Save,
    Undo,
    Copy,
    CopySvg,
    Grid,
    Rulers,
    Export,
    Image,
    Png,
    Svg,
    Palette,
    File,
    Edit,
    View,
}
pub struct CommandState {
    pub bar: MenuBarState,
    pub grid: bool,
    pub rulers: bool,
    pub registry: CommandRegistry<Action>,
    pub palette: SearchState,
    accept: bool,
    saves: usize,
    last: String,
    split: bool,
}
impl Default for CommandState {
    fn default() -> Self {
        Self {
            bar: MenuBarState::default(),
            grid: true,
            rulers: false,
            registry: catalog::registry(),
            palette: SearchState::default(),
            accept: true,
            saves: 0,
            last: "Ready to create".into(),
            split: false,
        }
    }
}
pub fn show(ui: &mut Ui, state: &mut CommandState) {
    for request in state.registry.dispatch(ui.ctx()) {
        apply(state, request);
    }
    let registry = state.registry.clone();
    let ctx = ui.ctx().clone();
    catalog::with_tree(&registry, &ctx, |tree| {
        let menus = [
            MenuBarItem::new("file", "File", tree.file),
            MenuBarItem::new("edit", "Edit", tree.edit),
            MenuBarItem::new("view", "View", tree.view),
        ];
        if let Some(action) = MenuBar::new("studio-menus", &mut state.bar, &menus)
            .accessible_name("Canal Studio menus")
            .show(ui)
            .action
        {
            activate(state, action);
        }
        ui.add_space(space::SM);
        let tool = |key| {
            registry
                .toolbar_item(ui.ctx(), &key)
                .expect("catalog action exists")
        };
        let tools = [
            tool(Action::Save),
            tool(Action::Undo),
            tool(Action::Grid).separator_before(true),
            ToolbarItem::action(Action::Export, "Export artwork")
                .icon(dutchy_egui::Icon::Download)
                .icon_only(false)
                .menu(tree.exports),
            tool(Action::Palette),
        ];
        if let Some(ToolbarEvent::Activated(value) | ToolbarEvent::Toggled { value, .. }) =
            Toolbar::new("studio-tools", &tools)
                .accessible_name("Canal Studio tools")
                .show(ui)
                .event
        {
            activate(state, value);
        }
        ui.add_space(space::MD);
        ui.horizontal_wrapped(|ui| {
            let out = SplitButton::new(
                "studio-export",
                "Export PNG",
                &mut state.split,
                tree.exports,
            )
            .size(ControlSize::Small)
            .alternatives_name("Export alternatives")
            .show(ui);
            if out.primary_clicked {
                activate(state, Action::Png);
            }
            if let Some(action) = out.action {
                activate(state, action);
            }
            if ui
                .add(
                    Button::new("Find command")
                        .size(ControlSize::Small)
                        .variant(ButtonVariant::Outline),
                )
                .clicked()
            {
                state.palette.open = true;
            }
        });
    });
    ui.add_space(space::LG);
    preview::show(ui, state);
    ui.add_space(space::MD);
    ui.add(
        Text::new(format!("{} · Saved {} times", state.last, state.saves))
            .size(TextSize::Sm)
            .color(TextColor::Muted),
    );
    ui.add(Switch::new(
        "accept-view",
        &mut state.accept,
        "Apply view changes",
    ));
    ui.add_space(space::MD);
    let reserved = state.registry.shortcuts_except(&Action::Save);
    let binding = &mut state.registry.get_mut(&Action::Save).unwrap().shortcut;
    Field::new("Save shortcut").show(ui, |ui| {
        KeyRecorder::new("studio-save", binding)
            .accessible_name("Save shortcut")
            .reserved(&reserved)
            .size(ControlSize::Small)
            .show(ui)
            .response
    });
    ui.add(Text::new("Menus, tools and search share command metadata. Record a new Save shortcut; every hint follows the same binding.").size(TextSize::Sm).color(TextColor::Muted));
    palette(state, ui.ctx());
}
pub fn palette(state: &mut CommandState, ctx: &egui::Context) {
    let registry = state.registry.clone();
    catalog::with_tree(&registry, ctx, |tree| {
        if let Some(out) = CommandPalette::new("studio-palette", &mut state.palette, tree.palette)
            .shortcut(None)
            .accessible_name("Canal Studio commands")
            .show(ctx)
            && let Some(action) = out.action
        {
            activate(state, action);
        }
    });
}
fn activate(state: &mut CommandState, action: Action) {
    if let Some(request) = state.registry.request(&action) {
        apply(state, request);
    }
}
fn apply(state: &mut CommandState, request: CommandRequest<Action>) {
    if let Some(checked) = request.checked {
        if state.accept {
            match request.value {
                Action::Grid => state.grid = checked,
                Action::Rulers => state.rulers = checked,
                _ => {}
            }
            state.registry.get_mut(&request.value).unwrap().checked = Some(checked);
            state.last = "View settings updated".into();
        } else {
            state.last = "View change declined".into();
        }
    } else {
        state.last = match request.value {
            Action::Save => {
                state.saves += 1;
                "Artwork saved"
            }
            Action::Png => "PNG export requested",
            Action::Svg => "SVG export requested",
            Action::Copy | Action::CopySvg => "Copy requested",
            Action::Palette => {
                state.bar.close();
                state.split = false;
                state.palette.open = !state.palette.open;
                "Find a command"
            }
            _ => "Ready to create",
        }
        .into();
    }
}
