use dutchy_egui::{FilterChipGroup, FilterChipItem, Text, TextColor, TextSize, tokens::space};
use egui::Ui;
use std::collections::HashSet;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Medium {
    Drawing,
    Photo,
    Print,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Collection {
    Canal,
    Studio,
    Archive,
}

const ASSETS: [(Medium, Collection, &str); 4] = [
    (Medium::Photo, Collection::Canal, "Canal at dusk"),
    (Medium::Drawing, Collection::Canal, "Waterline sketches"),
    (Medium::Print, Collection::Studio, "Studio colour proof"),
    (Medium::Photo, Collection::Studio, "Light meter study"),
];

#[derive(Default)]
pub struct FilterChipsState {
    media: HashSet<Medium>,
    collection: Option<Collection>,
}

pub fn show(ui: &mut Ui, state: &mut FilterChipsState) {
    ui.add(Text::heading("Filter chip group").size(TextSize::Xl5));
    ui.add_space(space::LG);
    filters(ui, state);
    ui.add_space(space::XXL);
    ui.add(Text::heading("Assets").size(TextSize::Xl2));
    let mut matches = false;
    for (medium, collection, title) in ASSETS {
        if (state.media.is_empty() || state.media.contains(&medium))
            && state.collection.is_none_or(|current| current == collection)
        {
            ui.add(Text::new(title));
            matches = true;
        }
    }
    if !matches {
        ui.add(Text::new("No assets match these filters.").color(TextColor::Muted));
    }
}

fn filters(ui: &mut Ui, state: &mut FilterChipsState) {
    let media = [
        FilterChipItem::new(Medium::Drawing, "Drawings"),
        FilterChipItem::new(Medium::Photo, "Photography"),
        FilterChipItem::new(Medium::Print, "Prints"),
    ];
    ui.add(Text::label("Media types"));
    for request in FilterChipGroup::new("media", &media)
        .accessible_name("Media types")
        .show(ui, |key| state.media.contains(key))
        .requests
    {
        if request.selected {
            state.media.insert(request.key);
        } else {
            state.media.remove(&request.key);
        }
    }

    ui.add_space(space::LG);
    let collections = [
        FilterChipItem::new(Collection::Canal, "Canal district"),
        FilterChipItem::new(Collection::Studio, "Studio work"),
        FilterChipItem::new(Collection::Archive, "Archive").enabled(false),
    ];
    ui.add(Text::label("Collection"));
    for request in FilterChipGroup::new("collection", &collections)
        .accessible_name("Collection")
        .show(ui, |key| state.collection == Some(*key))
        .requests
    {
        if request.selected {
            state.collection = Some(request.key);
        } else if state.collection == Some(request.key) {
            state.collection = None;
        }
    }
}
