use super::section;
use dutchy_egui::{
    Button, ButtonVariant, CommandPalette, Dropdown, DropdownAlign, Menu, MenuItem,
    MenuItemVariant, MenuWidth, SearchState, SearchableSelect, SelectOption, Text, TextColor,
    TextSize,
    tokens::{border, space},
};
use egui::{Color32, Painter, Rect, Stroke, Ui};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    Create,
    Duplicate,
    Archive,
    Delete,
}

#[derive(Default)]
pub struct MenuState {
    pub open: bool,
    pub context: bool,
    pub search: SearchState,
    pub palette: SearchState,
    pub framework: Option<&'static str>,
    pub last_action: Option<Action>,
    pub actions: usize,
}
impl MenuState {
    fn record(&mut self, action: Option<Action>) {
        if let Some(action) = action {
            self.last_action = Some(action);
            self.actions += 1;
        }
    }
    pub fn commands(&mut self, ctx: &egui::Context) {
        if let Some(result) =
            CommandPalette::new("showcase-commands", &mut self.palette, &items()).show(ctx)
        {
            self.record(result.action);
        }
    }
}

fn items() -> [MenuItem<'static, Action>; 4] {
    [
        MenuItem::new(Action::Create, "Create project")
            .group("Workspace")
            .shortcut("⌘ N")
            .icon(plus),
        MenuItem::new(Action::Duplicate, "Duplicate project")
            .group("Workspace")
            .shortcut("⌘ D"),
        MenuItem::new(Action::Archive, "Archive project")
            .group("Manage")
            .enabled(false),
        MenuItem::new(Action::Delete, "Delete project")
            .group("Manage")
            .variant(MenuItemVariant::Destructive),
    ]
}

fn plus(painter: &Painter, rect: Rect, color: Color32) {
    painter.line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(border::STANDARD, color),
    );
    painter.line_segment(
        [rect.left_center(), rect.right_center()],
        Stroke::new(border::STANDARD, color),
    );
}

fn feedback(ui: &mut Ui, state: &MenuState) {
    ui.add(
        Text::new(format!(
            "Host actions: {} · Last: {:?}",
            state.actions, state.last_action
        ))
        .size(TextSize::Sm)
        .color(TextColor::Muted),
    );
}

pub fn menus(ui: &mut Ui, state: &mut MenuState) {
    ui.add(Text::heading("Actions with structure."));
    ui.add(Text::new(
        "Groups, shortcut hints, and a clear destructive treatment.",
    ));
    section(ui, "01", "Grouped actions / native input", |ui| {
        let result = Menu::new("inline-workspace", &items())
            .accessible_name("Workspace actions")
            .show(ui);
        state.record(result.action);
        ui.add_space(space::LG);
        feedback(ui, state);
    });
    section(ui, "02", "Keyboard behavior", |ui| {
        ui.add(Text::new(
            "Tab into the menu. Up/Down and Home/End move among enabled actions. Enter or Space returns the chosen action to the application.",
        ));
        ui.add_space(space::SM);
        ui.add(Text::new(
            "Shortcut hints are labels. The host registers actual application shortcuts and decides what each action does.",
        ).color(TextColor::Muted));
    });
}

pub fn dropdowns(ui: &mut Ui, state: &mut MenuState) {
    ui.add(Text::heading("Open. Choose. Continue."));
    ui.add(Text::new(
        "Anchored actions, editable search, and global command discovery.",
    ));
    section(ui, "01", "Action dropdown / end alignment", |ui| {
        let result = Dropdown::new("workspace-dropdown", &mut state.open, &items())
            .width(MenuWidth::Small)
            .align(DropdownAlign::End)
            .accessible_name("Project actions")
            .show(ui, |ui| {
                ui.add(Button::new("Project actions").variant(ButtonVariant::Outline))
            });
        state.record(result.action);
        ui.add_space(space::LG);
        feedback(ui, state);
    });
    section(ui, "02", "Context menu", |ui| {
        let target = ui.add(
            Button::new("Right-click this project")
                .variant(ButtonVariant::Secondary)
                .full_width(true),
        );
        let result = Dropdown::new("context-project", &mut state.context, &items())
            .width(MenuWidth::Small)
            .accessible_name("Context project actions")
            .show_context(ui, &target);
        state.record(result.action);
        ui.add_space(space::SM);
        ui.add(
            Text::new("Secondary click or focus this target and press Shift+F10.")
                .size(TextSize::Sm)
                .color(TextColor::Muted),
        );
    });
    section(ui, "03", "Searchable selection", |ui| {
        ui.add(Text::label("Framework"));
        let options = [
            SelectOption::new("egui", "egui").id_salt("egui"),
            SelectOption::new("iced", "Iced").id_salt("iced"),
            SelectOption::new("reserved", "Reserved option")
                .enabled(false)
                .id_salt("reserved"),
            SelectOption::new("slint", "Slint").id_salt("slint"),
        ];
        SearchableSelect::new(
            "framework",
            &mut state.framework,
            &mut state.search,
            &options,
        )
        .accessible_name("Search frameworks")
        .placeholder("Type to filter frameworks…")
        .show(ui);
        ui.add_space(space::SM);
        ui.add(
            Text::new(format!(
                "Committed choice: {}",
                state.framework.unwrap_or("None")
            ))
            .size(TextSize::Sm),
        );
        if ui
            .add(Button::new("Clear search").variant(ButtonVariant::Ghost))
            .clicked()
        {
            state.search.query.clear();
        }
    });
    section(ui, "04", "Global command palette", |ui| {
        if ui.add(Button::new("Open command palette")).clicked() {
            state.palette.open = true;
        }
        ui.add_space(space::SM);
        ui.add(Text::new(
            "Command+K on macOS, Ctrl+K elsewhere. Type to filter, use Up/Down to choose, Enter to activate, or Escape to return. The shortcut works on every showcase page.",
        ).size(TextSize::Sm).color(TextColor::Muted));
    });
}
