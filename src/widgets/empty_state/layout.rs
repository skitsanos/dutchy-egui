use super::{EmptyState, EmptyStateVariant};
use crate::{
    Button, ButtonVariant, ControlSize, Text, TextSize, Theme, Tracking, Weight,
    tokens::{border, metrics, space},
};
use egui::{Response, Stroke, Ui, vec2};

pub(super) fn text(ui: &mut Ui, text: Text<'_>, width: f32) -> Response {
    ui.allocate_ui_with_layout(
        vec2(width, 0.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.add(text)
        },
    )
    .inner
}

fn button_width(ui: &Ui, label: &str) -> f32 {
    let theme = Theme::from_context(ui.ctx());
    let job = Text::new(label)
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .tracking(Tracking::Wide)
        .uppercase(true)
        .layout_job(&theme, f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job).size().x) + 2.0 * ControlSize::Small.padding()
}

pub(super) fn actions(ui: &mut Ui, state: &EmptyState<'_>) -> (bool, bool) {
    if state.action.is_none() && state.secondary.is_none() {
        return (false, false);
    }
    ui.add_space(space::XXL);
    let width = [state.action.as_deref(), state.secondary.as_deref()]
        .into_iter()
        .flatten()
        .map(|label| button_width(ui, label).min(ui.available_width()))
        .sum::<f32>()
        + if state.action.is_some() && state.secondary.is_some() {
            space::LG
        } else {
            0.0
        };
    if width <= ui.available_width() {
        ui.allocate_ui_with_layout(
            vec2(width, 0.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_min_width(width);
                ui.spacing_mut().item_spacing.x = space::LG;
                buttons(ui, state)
            },
        )
        .inner
    } else {
        ui.vertical_centered(|ui| buttons(ui, state)).inner
    }
}

fn buttons(ui: &mut Ui, state: &EmptyState<'_>) -> (bool, bool) {
    let action = state.action.as_deref().is_some_and(|label| {
        ui.scope_builder(
            egui::UiBuilder::new().scope_id(ui.make_persistent_id("primary-action")),
            |ui| {
                ui.add(
                    Button::new(label)
                        .size(ControlSize::Small)
                        .enabled(state.action_enabled)
                        .variant(if state.variant == EmptyStateVariant::Table {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Primary
                        }),
                )
            },
        )
        .inner
        .clicked()
    });
    let secondary = state.secondary.as_deref().is_some_and(|label| {
        ui.scope_builder(
            egui::UiBuilder::new().scope_id(ui.make_persistent_id("secondary-action")),
            |ui| {
                ui.add(
                    Button::new(label)
                        .size(ControlSize::Small)
                        .variant(ButtonVariant::Outline)
                        .enabled(state.secondary_enabled),
                )
            },
        )
        .inner
        .clicked()
    });
    (action, secondary)
}

pub(super) fn dashboard_border(ui: &Ui, rect: egui::Rect) {
    let rect = rect.shrink(border::ACCENT / 2.0);
    let points = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    ui.painter().extend(egui::Shape::dashed_line(
        &points,
        Stroke::new(border::ACCENT, Theme::from_context(ui.ctx()).palette.border),
        space::SM,
        space::SM,
    ));
}

pub(super) fn text_width(ui: &Ui, variant: EmptyStateVariant) -> f32 {
    ui.available_width()
        .min(if variant == EmptyStateVariant::Dashboard {
            metrics::DASHBOARD_TEXT_WIDTH
        } else {
            metrics::EMPTY_TEXT_WIDTH
        })
        .max(0.0)
}
