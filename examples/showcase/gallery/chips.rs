use dutchy_egui::{
    Button, ButtonVariant, Chip, ControlSize, Text, TextColor, TextSize, tokens::space,
};
use egui::{Response, Ui};

const DOCUMENTS: [(&str, &str); 3] = [
    ("Draft", "Color studies"),
    ("In review", "Type specimens"),
    ("Published", "Canal district guide"),
];
const TAGS: [&str; 3] = ["Typography", "Layout", "Icons"];

pub struct ChipsState {
    filters: [bool; DOCUMENTS.len()],
    tags: [bool; TAGS.len()],
}

impl Default for ChipsState {
    fn default() -> Self {
        Self {
            filters: [false; DOCUMENTS.len()],
            tags: [true; TAGS.len()],
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut ChipsState) {
    ui.add(Text::heading("Chips").size(TextSize::Xl5));
    ui.add_space(space::LG);
    ui.add(Text::heading("Filter by status").size(TextSize::Xl2));
    ui.horizontal_wrapped(|ui| {
        for ((status, _), selected) in DOCUMENTS.into_iter().zip(&mut state.filters) {
            if let Some(next) = Chip::new(("status", status), status)
                .selected(*selected)
                .show(ui)
                .toggle_requested
            {
                *selected = next;
            }
        }
    });
    let all = !state.filters.contains(&true);
    for ((_, title), selected) in DOCUMENTS.into_iter().zip(state.filters) {
        if all || selected {
            ui.add(Text::new(title));
        }
    }

    ui.add_space(space::XXL);
    ui.add(Text::heading("Project tags").size(TextSize::Xl2));
    tags(ui, &mut state.tags);
}

fn tags(ui: &mut Ui, tags: &mut [bool; TAGS.len()]) {
    let mut move_focus = false;
    ui.horizontal_wrapped(|ui| {
        for (label, visible) in TAGS.into_iter().zip(tags.iter_mut()) {
            if *visible {
                let chip = Chip::new(("tag", label), label).removable(true).show(ui);
                if chip.remove_requested {
                    *visible = false;
                    move_focus |= chip.remove.as_ref().is_some_and(Response::has_focus);
                }
            }
        }
    });
    if !tags.contains(&true) {
        ui.add(Text::new("No tags").color(TextColor::Muted));
    }
    let restore = ui.add(
        Button::new("Restore tags")
            .size(ControlSize::Small)
            .variant(ButtonVariant::Outline),
    );
    if move_focus {
        restore.request_focus();
    }
    if restore.clicked() {
        *tags = [true; TAGS.len()];
    }
}
