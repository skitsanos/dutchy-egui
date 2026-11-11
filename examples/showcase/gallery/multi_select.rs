use dutchy_egui::{
    Field, MultiSelect, SearchState, SelectOption, Text, TextColor, TextSize, tokens::space,
};
use egui::Ui;

#[derive(Clone, Copy, PartialEq)]
enum Media {
    Drawing,
    Photo,
    Print,
    Sculpture,
    Video,
}

const ASSETS: [(Media, &str); 5] = [
    (Media::Drawing, "Waterline sketches"),
    (Media::Photo, "Canal at dusk"),
    (Media::Photo, "Light meter study"),
    (Media::Print, "Studio colour proof"),
    (Media::Sculpture, "Bronze maquette"),
];

pub struct AssetFilters {
    selection: Vec<Media>,
    search: SearchState,
}

impl Default for AssetFilters {
    fn default() -> Self {
        Self {
            selection: vec![Media::Photo],
            search: SearchState::default(),
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut AssetFilters) {
    ui.add(Text::heading("Multi-select").size(TextSize::Xl5));
    ui.add_space(space::LG);
    let options = [
        SelectOption::new(Media::Drawing, "Drawings"),
        SelectOption::new(Media::Photo, "Photography"),
        SelectOption::new(Media::Print, "Prints"),
        SelectOption::new(Media::Sculpture, "Sculpture"),
        SelectOption::new(Media::Video, "Video").enabled(false),
    ];
    Field::new("Media types")
        .help("Search by name, then use the arrow keys and Enter to change the selection.")
        .show(ui, |ui| {
            let result =
                MultiSelect::new("media-types", &state.selection, &mut state.search, &options)
                    .accessible_name("Media types")
                    .placeholder("Find media types…")
                    .show(ui);
            for request in result.requests {
                state.selection.retain(|value| *value != request.value);
                if request.selected {
                    state.selection.push(request.value);
                }
                ui.ctx().request_repaint();
            }
            result.response
        });
    ui.add_space(space::LG);
    ui.add(Text::heading("Matching assets").size(TextSize::Xl));
    for (kind, label) in ASSETS {
        if state.selection.is_empty() || state.selection.contains(&kind) {
            ui.add(Text::new(label).color(TextColor::Muted));
        }
    }
}
