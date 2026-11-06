use super::{
    Step, StepperVariant,
    paint::Cell,
    state::{Model, Status},
};
use crate::{
    Theme,
    tokens::{border, stepper as geometry},
};
use egui::{Id, Response, Ui, pos2, vec2};

pub(super) fn show<T>(
    ui: &mut Ui,
    root: Id,
    items: &[Step<'_, T>],
    model: &Model,
    variant: StepperVariant,
) -> Vec<Response> {
    match variant {
        StepperVariant::Compact => compact(ui, root, items, model),
        StepperVariant::Vertical => vertical(ui, root, items, model),
        _ => columns(ui, root, items, model, variant),
    }
}
fn columns<T>(
    ui: &mut Ui,
    root: Id,
    items: &[Step<'_, T>],
    model: &Model,
    variant: StepperVariant,
) -> Vec<Response> {
    if items.is_empty() {
        return vec![];
    }
    let available = ui.available_width();
    let count = if variant == StepperVariant::Horizontal {
        items.len()
    } else {
        (available / geometry::PROGRESS_COLUMN).floor().max(1.0) as usize
    };
    let count = count.min(items.len());
    let line = ui.painter().add(egui::Shape::Noop);
    let responses = ui
        .with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let mut responses = Vec::with_capacity(items.len());
            // Explicit rows prevent fractional column rounding from wrapping a
            // horizontal indicator and drawing a diagonal connector.
            for (row, chunk) in items.chunks(count).enumerate() {
                ui.columns(count, |columns| {
                    for (column, (offset, step)) in columns.iter_mut().zip(chunk.iter().enumerate())
                    {
                        responses.push(
                            Cell {
                                index: row * count + offset,
                                step,
                                model,
                                variant,
                            }
                            .show(
                                column,
                                root,
                                column.available_width(),
                            ),
                        );
                    }
                });
            }
            responses
        })
        .inner;
    if variant == StepperVariant::Horizontal {
        connect(ui, line, &responses, items, model, variant);
    }
    responses
}
fn vertical<T>(ui: &mut Ui, root: Id, items: &[Step<'_, T>], model: &Model) -> Vec<Response> {
    let line = ui.painter().add(egui::Shape::Noop);
    let mut responses = Vec::with_capacity(items.len());
    for (index, step) in items.iter().enumerate() {
        if index > 0 {
            ui.add_space(geometry::VERTICAL_GAP);
        }
        responses.push(
            Cell {
                index,
                step,
                model,
                variant: StepperVariant::Vertical,
            }
            .show(ui, root, ui.available_width()),
        );
    }
    connect(ui, line, &responses, items, model, StepperVariant::Vertical);
    responses
}
fn compact<T>(ui: &mut Ui, root: Id, items: &[Step<'_, T>], model: &Model) -> Vec<Response> {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(geometry::SCROLL_PADDING))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt("compact-overflow")
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    let line = ui.painter().add(egui::Shape::Noop);
                    let responses = ui
                        .horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let mut responses = Vec::with_capacity(items.len());
                            for (index, step) in items.iter().enumerate() {
                                if index > 0 {
                                    ui.allocate_space(vec2(
                                        geometry::COMPACT_CONNECTOR + geometry::COMPACT_GAP * 2.0,
                                        geometry::COMPACT_INDICATOR,
                                    ));
                                }
                                responses.push(
                                    Cell {
                                        index,
                                        step,
                                        model,
                                        variant: StepperVariant::Compact,
                                    }
                                    .show(
                                        ui,
                                        root,
                                        geometry::COMPACT_INDICATOR,
                                    ),
                                );
                            }
                            responses
                        })
                        .inner;
                    connect(ui, line, &responses, items, model, StepperVariant::Compact);
                    responses
                })
                .inner
        })
        .inner
}
fn connect<T>(
    ui: &Ui,
    line: egui::layers::ShapeIdx,
    responses: &[Response],
    items: &[Step<'_, T>],
    model: &Model,
    variant: StepperVariant,
) {
    let p = Theme::from_context(ui.ctx()).palette;
    let mut lines = Vec::new();
    for (index, pair) in responses.windows(2).enumerate() {
        let a = Cell {
            index,
            step: &items[index],
            model,
            variant,
        }
        .marker(pair[0].rect);
        let b = Cell {
            index: index + 1,
            step: &items[index + 1],
            model,
            variant,
        }
        .marker(pair[1].rect);
        let points = if variant == StepperVariant::Vertical {
            [
                pos2(a.center().x, a.bottom() + geometry::CONNECTOR_INSET),
                pos2(b.center().x, b.top() - geometry::CONNECTOR_INSET),
            ]
        } else {
            let gap = if variant == StepperVariant::Compact {
                geometry::COMPACT_GAP
            } else {
                0.0
            };
            [
                pos2(a.right() + gap, a.center().y),
                pos2(b.left() - gap, b.center().y),
            ]
        };
        let color = if model.status(index) == Status::Completed {
            p.primary
        } else {
            p.border
        };
        lines.push(egui::Shape::line_segment(
            points,
            egui::Stroke::new(border::STANDARD, color),
        ));
    }
    ui.painter().set(line, egui::Shape::Vec(lines));
}
