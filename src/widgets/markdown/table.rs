//! Document tables: content-sized columns that shrink to the available width,
//! a muted uppercase header, one-point row separators and a two-point border.

use super::{
    Renderer,
    inline::{self, Typography},
    model::{CellAlign, Inline, Table},
};
use crate::tokens::{border, markdown as tokens};
use egui::{
    Align, Frame, Layout, Margin, Rect, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder,
    WidgetInfo, accesskit::Role, pos2, vec2,
};

pub(super) fn show(r: &mut Renderer, ui: &mut Ui, table: &Table) {
    let columns = table
        .alignments
        .len()
        .max(table.head.len())
        .max(table.rows.iter().map(Vec::len).max().unwrap_or(0));
    if columns == 0 {
        return;
    }
    let widths = widths(r, ui, table, columns);
    let total: f32 = widths.iter().sum();
    if total > ui.available_width() + 0.5 {
        egui::ScrollArea::horizontal()
            .id_salt(r.next_id())
            .auto_shrink([false, true])
            .show(ui, |ui| grid(r, ui, table, &widths));
    } else {
        grid(r, ui, table, &widths);
    }
}

/// Natural (unwrapped) widths. When they overflow, columns give up the slack
/// above their widest word in proportion; words never break mid-column.
fn widths(r: &Renderer, ui: &mut Ui, table: &Table, columns: usize) -> Vec<f32> {
    let padding = 2.0 * tokens::CELL_PADDING_X;
    let mut natural = vec![padding; columns];
    let mut minimum = vec![padding; columns];
    let mut note = |ui: &mut Ui, column: usize, cell: &[Inline], typography: Typography| {
        let measure = inline::measure(ui, &r.theme, cell, typography);
        natural[column] = natural[column].max(measure.natural + padding);
        minimum[column] = minimum[column].max(measure.minimum + padding);
    };
    for (column, cell) in table.head.iter().enumerate().take(columns) {
        note(ui, column, cell, Typography::header());
    }
    for row in &table.rows {
        for (column, cell) in row.iter().enumerate().take(columns) {
            note(ui, column, cell, Typography::cell());
        }
    }
    fit(natural, &minimum, ui.available_width())
}

fn fit(natural: Vec<f32>, minimum: &[f32], available: f32) -> Vec<f32> {
    let total: f32 = natural.iter().sum();
    if !available.is_finite() || total <= available {
        return natural;
    }
    let floor: f32 = minimum.iter().sum();
    if floor >= available {
        return minimum.to_vec();
    }
    let deficit = total - available;
    let slack = total - floor;
    natural
        .iter()
        .zip(minimum)
        .map(|(natural, minimum)| natural - deficit * (natural - minimum) / slack)
        .collect()
}

fn grid(r: &mut Renderer, ui: &mut Ui, table: &Table, widths: &[f32]) {
    let p = r.theme.palette;
    let width: f32 = widths.iter().sum();
    let result = ui.scope_builder(UiBuilder::new().sense(Sense::hover()), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        if !table.head.is_empty() {
            row(r, ui, widths, &table.alignments, &table.head, 0, true);
            separator(ui, width, border::STANDARD, p.border);
        }
        for (index, cells) in table.rows.iter().enumerate() {
            if index > 0 {
                separator(ui, width, border::THIN, p.border);
            }
            row(r, ui, widths, &table.alignments, cells, index + 1, false);
        }
        let rect = ui.min_rect();
        ui.painter().rect_stroke(
            Rect::from_min_size(rect.min, vec2(width, rect.height())),
            0,
            Stroke::new(border::STANDARD, p.border),
            StrokeKind::Inside,
        );
    });
    result
        .response
        .widget_info(|| WidgetInfo::labeled(Role::Table, ui.is_enabled(), "Table"));
    let rows = table.rows.len() + usize::from(!table.head.is_empty());
    result
        .response
        .ctx
        .accesskit_node_builder(result.response.id, |node| {
            node.set_row_count(rows);
            node.set_column_count(widths.len());
        });
}

fn separator(ui: &mut Ui, width: f32, thickness: f32, color: egui::Color32) {
    let (_, rect) = ui.allocate_space(vec2(width, thickness));
    ui.painter().rect_filled(rect, 0, color);
}

fn halign(align: CellAlign) -> Align {
    match align {
        CellAlign::Left => Align::Min,
        CellAlign::Center => Align::Center,
        CellAlign::Right => Align::Max,
    }
}

/// Fixed column rectangles; the row is as tall as its tallest cell.
fn row(
    r: &mut Renderer,
    ui: &mut Ui,
    widths: &[f32],
    alignments: &[CellAlign],
    cells: &[Vec<Inline>],
    index: usize,
    header: bool,
) {
    let p = r.theme.palette;
    let typography = if header {
        Typography::header()
    } else {
        Typography::cell()
    };
    let result = ui.scope_builder(UiBuilder::new().sense(Sense::hover()), |ui| {
        let background = ui.painter().add(Shape::Noop);
        let origin = ui.cursor().min;
        let mut x = origin.x;
        let mut height = 0.0_f32;
        for (column, &width) in widths.iter().enumerate() {
            let rect = Rect::from_min_max(pos2(x, origin.y), pos2(x + width, f32::INFINITY));
            let align = alignments.get(column).copied().unwrap_or_default();
            let mut child = ui.new_child(
                UiBuilder::new()
                    .scope_id(r.next_id())
                    .max_rect(rect)
                    .layout(Layout::top_down(halign(align))),
            );
            child.set_clip_rect(ui.clip_rect().intersect(rect));
            child.set_width(width);
            cell(r, &mut child, cells.get(column), typography, align);
            height = height.max(child.min_rect().height());
            let response = child.response();
            response.ctx.accesskit_node_builder(response.id, |node| {
                node.set_role(if header {
                    Role::ColumnHeader
                } else {
                    Role::Cell
                });
                node.set_row_index(index);
                node.set_column_index(column);
            });
            x += width;
        }
        let width: f32 = widths.iter().sum();
        ui.allocate_space(vec2(width, height));
        if header {
            let rect = Rect::from_min_size(origin, vec2(width, height));
            ui.painter()
                .set(background, Shape::rect_filled(rect, 0, p.muted));
        }
    });
    result
        .response
        .ctx
        .accesskit_node_builder(result.response.id, |node| {
            node.set_role(Role::Row);
            node.set_label(format!("Row {}", index + 1));
            node.set_row_index(index);
        });
}

fn cell(
    r: &mut Renderer,
    ui: &mut Ui,
    inlines: Option<&Vec<Inline>>,
    typography: Typography,
    align: CellAlign,
) {
    let margin = Margin::symmetric(tokens::CELL_PADDING_X as i8, tokens::CELL_PADDING_Y as i8);
    Frame::NONE.inner_margin(margin).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if let Some(inlines) = inlines {
            inline::paragraph(r, ui, inlines, typography, halign(align), Role::Label);
        }
    });
}
