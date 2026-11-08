//! Bullet, numbered and task lists: a marker column in front of item blocks.

use super::{Renderer, blocks, model::Block, model::Item, source::plain_text};
use crate::{
    Family, Text, TextColor, TextSize, Weight,
    tokens::{border, markdown as tokens},
};
use egui::{
    Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, accesskit::Role, pos2, vec2,
};

pub(super) fn show(r: &mut Renderer, ui: &mut Ui, start: Option<u64>, items: &[Item]) {
    let line = TextSize::Base.points() * TextSize::Base.line_height();
    let marker_width = marker_width(r, ui, start, items.len());
    let result = ui.scope_builder(UiBuilder::new().sense(Sense::hover()), |ui| {
        r.depth += 1;
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                ui.add_space(tokens::ITEM_GAP);
            }
            let number = start.map(|start| start.saturating_add(index as u64));
            show_item(r, ui, item, number, marker_width, line);
        }
        r.depth -= 1;
    });
    result.response.widget_info(|| WidgetInfo::new(Role::List));
}

fn number_text(text: &str) -> Text<'_> {
    Text::new(text)
        .family(Family::Mono)
        .size(TextSize::Sm)
        .weight(Weight::Medium)
        .color(TextColor::Muted)
}

/// Numbered markers reserve the width of the largest number in the list.
fn marker_width(r: &Renderer, ui: &mut Ui, start: Option<u64>, count: usize) -> f32 {
    let Some(start) = start else {
        return tokens::MARKER_WIDTH;
    };
    let last = start.saturating_add(count.saturating_sub(1) as u64);
    let job = number_text(&format!("{last}.")).layout_job(&r.theme, f32::INFINITY);
    let width = ui.fonts_mut(|fonts| fonts.layout_job(job)).size().x;
    (width + tokens::MARKER_GAP).max(tokens::MARKER_WIDTH)
}

fn show_item(
    r: &mut Renderer,
    ui: &mut Ui,
    item: &Item,
    number: Option<u64>,
    marker_width: f32,
    line: f32,
) {
    let result = ui.scope_builder(UiBuilder::new().sense(Sense::hover()), |ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            marker(r, ui, item, number, marker_width, line);
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                blocks::show_spaced(r, ui, &item.blocks, tokens::ITEM_BLOCK_GAP);
            });
        });
    });
    result
        .response
        .widget_info(|| WidgetInfo::new(Role::ListItem));
}

fn marker(r: &mut Renderer, ui: &mut Ui, item: &Item, number: Option<u64>, width: f32, line: f32) {
    let (_, rect) = ui.allocate_space(vec2(width, line));
    let p = r.theme.palette;
    if let Some(checked) = item.task {
        task_box(r, ui, rect, checked, &item_text(item));
        return;
    }
    match number {
        Some(number) => {
            let job = number_text(&format!("{number}.")).layout_job(&r.theme, f32::INFINITY);
            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
            let origin = pos2(
                rect.right() - tokens::MARKER_GAP - galley.size().x,
                rect.center().y - galley.size().y / 2.0,
            );
            ui.painter().galley(origin, galley, p.muted_foreground);
        }
        None => bullet(ui, rect, r.depth, p.foreground),
    }
}

/// Filled square, then an outlined square, then a smaller filled square.
fn bullet(ui: &Ui, rect: Rect, depth: usize, color: egui::Color32) {
    let side = if depth >= 3 {
        tokens::BULLET - 2.0
    } else {
        tokens::BULLET
    };
    let center = pos2(
        rect.right() - tokens::MARKER_GAP - tokens::BULLET / 2.0,
        rect.center().y,
    );
    let square = Rect::from_center_size(center, Vec2::splat(side));
    if depth == 2 {
        ui.painter().rect_stroke(
            square,
            0,
            Stroke::new(border::THIN, color),
            StrokeKind::Inside,
        );
    } else {
        ui.painter().rect_filled(square, 0, color);
    }
}

/// Read-only task state; the document cannot be edited here.
fn task_box(r: &mut Renderer, ui: &mut Ui, rect: Rect, checked: bool, label: &str) {
    let p = r.theme.palette;
    let side = tokens::TASK_BOX;
    let square = Rect::from_center_size(
        pos2(
            rect.right() - tokens::MARKER_GAP - side / 2.0,
            rect.center().y,
        ),
        Vec2::splat(side),
    );
    let response = ui.interact(square, r.next_id(), Sense::hover());
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, ui.is_enabled(), checked, label));
    response
        .ctx
        .accesskit_node_builder(response.id, |node| node.set_read_only());
    let (fill, edge) = if checked {
        (p.primary, p.primary)
    } else {
        (p.background, p.border)
    };
    ui.painter().rect(
        square,
        0,
        fill,
        Stroke::new(border::STANDARD, edge),
        StrokeKind::Inside,
    );
    if checked {
        let point = |x: f32, y: f32| square.min + vec2(x * side, y * side);
        ui.painter().add(egui::Shape::line(
            vec![point(0.2, 0.5), point(0.42, 0.72), point(0.8, 0.25)],
            Stroke::new(border::STANDARD, p.on_primary),
        ));
    }
}

fn item_text(item: &Item) -> String {
    match item.blocks.first() {
        Some(Block::Paragraph(inlines)) => plain_text(inlines),
        _ => String::from("Task"),
    }
}
