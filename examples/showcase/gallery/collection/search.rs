use super::CollectionState;
use crate::gallery::section;
use dutchy_egui::{
    ControlSize, Field, SearchInput, Switch, Text, TextColor, TextSize, tokens::space,
};
use egui::Ui;

pub(super) fn show(ui: &mut Ui, state: &mut CollectionState) {
    ui.add(Text::label("Native additions / Search").color(TextColor::Primary));
    ui.add(Text::heading("Find a little clarity.").size(TextSize::Xl5));
    ui.add(Text::new("Native text editing, a dedicated clear action and an optional key hint. Filtering and shortcuts belong to your application.").color(TextColor::Muted));
    section(ui, "01", "Shared sizes / native editing", |ui| {
        ui.add(Switch::new(
            "search-disabled",
            &mut state.disabled,
            "Disable sample searches",
        ));
        ui.add(Switch::new(
            "search-readonly",
            &mut state.read_only,
            "Read-only sample searches",
        ));
        for (index, (size, name)) in [
            (ControlSize::Small, "Compact search"),
            (ControlSize::Medium, "Default search"),
            (ControlSize::Large, "Large search"),
        ]
        .into_iter()
        .enumerate()
        {
            Field::new(name)
                .help("Type to reveal Clear; the key chip is a host-owned hint.")
                .show(ui, |ui| {
                    ui.add(
                        SearchInput::new(name, &mut state.samples[index])
                            .accessible_name(name)
                            .size(size)
                            .placeholder("Styles, moods or techniques…")
                            .key_hint("Ctrl+K")
                            .enabled(!state.disabled)
                            .read_only(state.read_only)
                            .char_limit(80),
                    )
                });
        }
    });
    ui.add_space(space::LG);
    section(ui, "02", "Application composition", |ui| {
        ui.add(Text::new("The Media cards page combines SearchInput, sort selection, category filters and explicit result counts. Clear and saved toggles only update host state.").color(TextColor::Muted));
    });
}
