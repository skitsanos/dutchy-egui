use super::{Page, frame};
use dutchy_egui::{
    ControlSize, List, ListItem, ListSelectionMode, ListState, Text, TextColor, Theme,
    tokens::{border, space},
};
use egui::{Align, Layout, Ui};

pub(super) fn show(ui: &mut Ui, theme: Theme, page: &mut Page, state: &mut ListState<Page>) {
    egui::Panel::left("showcase-navigation")
        .exact_size(208.0)
        .resizable(false)
        .frame(frame(theme, 16))
        .show(ui, |ui| {
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                foundation(ui, theme);
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.add(Text::label("The system").color(TextColor::Primary));
                    ui.add_space(space::LG);
                    pages(ui, page, state);
                });
            });
        });
}

fn pages(ui: &mut Ui, page: &mut Page, state: &mut ListState<Page>) {
    let items: Vec<_> = Page::all()
        .map(|page| ListItem::new(page, page.name()))
        .collect();
    if !state.selected.contains(page) {
        state.selected.clear();
        state.selected.insert(*page);
        state.focused = Some(*page);
    }
    let result = List::new("pages", state, &items)
        .accessible_name("Showcase pages")
        .selection_mode(ListSelectionMode::Single)
        .row_height(ControlSize::Small.height())
        .max_height(ui.available_height())
        .show(ui);
    if let Some(selected) = result
        .activated
        .or_else(|| state.selected.iter().next().copied())
    {
        *page = selected;
    }
}

/// Keep the footer below the scrolling page list.
fn foundation(ui: &mut Ui, theme: Theme) {
    ui.add(Text::new("Zero radius.\nFlat color.\nBold typography.").color(TextColor::Muted));
    ui.add(Text::label("01 / Foundation"));
    ui.add_space(space::LG);
    let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), border::THIN));
    ui.painter().rect_filled(rect, 0, theme.palette.border);
    ui.add_space(space::LG);
}
