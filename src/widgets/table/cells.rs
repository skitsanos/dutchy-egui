use super::heights::Heights;
use egui::{Id, Rect, Response, Ui, UiBuilder, pos2, vec2};
/// Fixed column rectangles prevent oversized host content from shifting later columns.
pub(super) fn show<T>(
    ui: &mut Ui,
    widths: &[f32],
    ids: &[Id],
    header: bool,
    row: usize,
    mut content: impl FnMut(&mut Ui, usize) -> T,
) -> (Vec<Response>, Vec<T>) {
    let origin = ui.cursor().min;
    let mut x = origin.x;
    let mut height = 0.0_f32;
    let mut responses = Vec::new();
    let mut inner = Vec::new();
    let previous = Heights::load(ui, ids);
    let mut measured = Vec::with_capacity(widths.len());
    for (index, &width) in widths.iter().enumerate() {
        let rect = Rect::from_min_max(
            pos2(x, origin.y + previous.offset(ids[index])),
            pos2(x + width, f32::INFINITY),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .scope_id(ids[index])
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        child.set_clip_rect(ui.clip_rect().intersect(rect));
        child.set_width(width);
        inner.push(content(&mut child, index));
        let natural_height = child.min_rect().height();
        measured.push((ids[index], natural_height));
        height = height.max(natural_height);
        let response = child.response();
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_role(if header {
                egui::accesskit::Role::ColumnHeader
            } else {
                egui::accesskit::Role::Cell
            })
        });
        responses.push(response);
        x += width;
    }
    previous.save(ui, measured);
    ui.allocate_space(vec2(widths.iter().sum(), height));
    let mut x = origin.x;
    for (index, response) in responses.iter_mut().enumerate() {
        response.rect = Rect::from_min_size(pos2(x, origin.y), vec2(widths[index], height));
        response.interact_rect = response.rect;
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_row_index(row);
            node.set_column_index(index);
        });
        bounds(response, response.rect);
        x += widths[index];
    }
    (responses, inner)
}
pub(super) fn row_metadata(response: &Response, index: usize, selected: Option<bool>) {
    bounds(response, response.rect);
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Row);
        node.set_label(format!("Row {index}"));
        node.set_row_index(index);
        if let Some(selected) = selected {
            node.set_selected(selected);
        }
    });
}
pub(super) fn bounds(response: &Response, rect: Rect) {
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_bounds(egui::accesskit::Rect {
            x0: rect.left().into(),
            y0: rect.top().into(),
            x1: rect.right().into(),
            y1: rect.bottom().into(),
        });
    });
}
pub(super) fn separator(ui: &mut Ui, width: f32, thickness: f32) {
    let (_, rect) = ui.allocate_space(vec2(width, thickness));
    ui.painter()
        .rect_filled(rect, 0, crate::Theme::from_context(ui.ctx()).palette.border);
}
