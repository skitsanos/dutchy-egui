use dutchy_egui::{
    Badge, BadgeVariant, Button, ControlSize, List, ListItem, ListSelectionMode, ListState, Select,
    SelectOption, Switch, Text, TextColor, TextSize, tokens::space,
};
use egui::Ui;

const KEYS: &str = "↑↓ / Home / End / Page keys move. Shift extends. Command \
    (Control on Windows and Linux) toggles; add Shift for ranges. Space toggles; \
    Enter opens. Type a project prefix, spaces included.";

pub struct ListsState {
    rows: Vec<ListItem<'static, u32>>,
    list: ListState<u32>,
    mode: Option<u32>,
    read_only: bool,
    disabled: bool,
    activated: Option<u32>,
}
impl Default for ListsState {
    fn default() -> Self {
        let names = ["Canal", "Harbor", "Market", "Garden", "Studio", "Workshop"];
        Self {
            rows: (0..10000)
                .map(|i| {
                    ListItem::new(
                        i,
                        format!("{} / project {i:05}", names[i as usize % names.len()]),
                    )
                    .enabled(i % 17 != 16)
                })
                .collect(),
            list: ListState::default(),
            mode: Some(2),
            read_only: false,
            disabled: false,
            activated: None,
        }
    }
}
pub fn show(ui: &mut Ui, state: &mut ListsState) {
    ui.add(Text::heading("Lists that keep up.").size(TextSize::Xl5));
    ui.add(
        Text::new("10,000 keyed projects. Only visible rows are drawn.").color(TextColor::Muted),
    );
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        let options = [
            SelectOption::new(0, "No selection"),
            SelectOption::new(1, "Single selection"),
            SelectOption::new(2, "Multiple selection"),
        ];
        ui.add(
            Select::new("list-mode", &mut state.mode, &options).accessible_name("Selection mode"),
        );
        ui.add(Switch::new(
            "list-readonly",
            &mut state.read_only,
            "Read only",
        ));
        ui.add(Switch::new(
            "list-disabled",
            &mut state.disabled,
            "Disabled",
        ));
    });
    ui.add_space(space::LG);
    let mode = match state.mode {
        Some(0) => ListSelectionMode::None,
        Some(1) => ListSelectionMode::Single,
        _ => ListSelectionMode::Multiple,
    };
    let result = List::new("projects", &mut state.list, &state.rows)
        .selection_mode(mode)
        .accessible_name("Studio projects")
        .row_height(56.0)
        .max_height(336.0)
        .empty_text("No projects left. Reset to restore them.")
        .enabled(!state.disabled)
        .read_only(state.read_only)
        .show_with(ui, |ui, item, row| {
            ui.add(Text::new(item.label.as_ref()).size(TextSize::Sm));
            if row.index % 7 == 0 {
                ui.add(Badge::new("Updated").variant(BadgeVariant::Primary));
            }
        });
    if let Some(key) = result.activated {
        state.activated = Some(key);
    }
    ui.add_space(space::MD);
    ui.add(Text::label(format!(
        "{} projects / {} selected / rows {}–{} rendered",
        state.rows.len(),
        state.list.selected.len(),
        result.visible_range.start,
        result.visible_range.end
    )));
    ui.add(Text::new(KEYS).size(TextSize::Sm).color(TextColor::Muted));
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        if ui
            .add(Button::new("Reverse view").size(ControlSize::Small))
            .clicked()
        {
            state.rows.reverse();
        }
        if ui
            .add(
                Button::new("Remove selected")
                    .size(ControlSize::Small)
                    .enabled(!state.list.selected.is_empty()),
            )
            .clicked()
        {
            state
                .rows
                .retain(|item| !state.list.selected.contains(&item.key));
        }
        if ui
            .add(Button::new("Reset projects").size(ControlSize::Small))
            .clicked()
        {
            *state = ListsState::default();
        }
    });
    if let Some(key) = state.activated {
        ui.add(Text::new(format!("Host opened project {key:05}.")).color(TextColor::Primary));
    }
}
