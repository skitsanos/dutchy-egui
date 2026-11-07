use super::capture::State;
use crate::{
    ScribbleDrawing, ScribbleStroke, Text, TextSize,
    tokens::{border, scribble as tokens},
};
use egui::{Pos2, Rect, Response, Stroke, Ui, emath::RectTransform, pos2, vec2};

pub(super) fn allocate(
    ui: &mut Ui,
    drawing: &ScribbleDrawing,
    preferred: Option<f32>,
) -> (Rect, Rect) {
    let available = if ui.available_width().is_finite() {
        ui.available_width().max(1.0)
    } else {
        drawing.size().x
    };
    let aspect = drawing.size().y / drawing.size().x;
    let width = preferred
        .unwrap_or(available)
        .min(available)
        .min(tokens::MAX_DIMENSION as f32)
        .max(1.0);
    let inset = border::STANDARD.min(width / 4.0);
    let height = (width - inset * 2.0) * aspect + inset * 2.0;
    let fit = (tokens::MAX_DIMENSION as f32 / height).min(1.0);
    let (_, outer) = ui.allocate_space(vec2(width, height) * fit);
    let inset = inset * fit;
    (outer, outer.shrink(inset))
}
pub(super) fn show(
    ui: &Ui,
    outer: Rect,
    canvas: &Response,
    drawing: &ScribbleDrawing,
    state: &State,
    placeholder: &str,
    editable: bool,
) {
    let theme = crate::Theme::from_context(ui.ctx());
    ui.painter().rect_filled(outer, 0, tokens::PAPER);
    ui.painter().rect_stroke(
        outer,
        0,
        Stroke::new(border::STANDARD, theme.palette.border),
        egui::StrokeKind::Inside,
    );
    if !ui.is_rect_visible(canvas.rect) {
        return;
    }
    let painter = ui
        .painter()
        .with_clip_rect(ui.clip_rect().intersect(canvas.rect));
    let transform =
        RectTransform::from_to(Rect::from_min_size(Pos2::ZERO, drawing.size()), canvas.rect);
    for stroke in drawing.strokes() {
        ink(&painter, &transform, stroke);
    }
    if let Some(draft) = &state.draft {
        polyline(
            &painter,
            &transform,
            &draft.points,
            draft.brush.ink(),
            draft.brush.pen_width().points(),
        );
    }
    if drawing.is_empty()
        && state.draft.is_none()
        && canvas.rect.height() >= crate::tokens::space::LG
    {
        let job = Text::new(placeholder)
            .size(TextSize::Sm)
            .color(tokens::MUTED_INK)
            .layout_job(&theme, canvas.rect.width());
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        painter.galley(
            canvas.rect.center() - galley.size() / 2.0,
            galley,
            tokens::MUTED_INK,
        );
    }
    if editable
        && crate::interaction::focus_is_visible(canvas)
        && let Some(cursor) = state.cursor
    {
        cursor_marker(&painter, transform.transform_pos(cursor));
    }
    crate::interaction::paint_focus_inside(ui, canvas, theme.palette.primary);
}
fn ink(painter: &egui::Painter, transform: &RectTransform, stroke: &ScribbleStroke) {
    polyline(
        painter,
        transform,
        stroke.points(),
        stroke.brush().ink(),
        stroke.brush().pen_width().points(),
    );
}
fn polyline(
    painter: &egui::Painter,
    transform: &RectTransform,
    points: &[Pos2],
    color: egui::Color32,
    width: f32,
) {
    let width = width * transform.scale().x;
    if points.len() == 1 {
        painter.circle_filled(transform.transform_pos(points[0]), width / 2.0, color);
    } else {
        painter.add(egui::Shape::line(
            points.iter().map(|p| transform.transform_pos(*p)).collect(),
            Stroke::new(width, color),
        ));
    }
}
fn cursor_marker(painter: &egui::Painter, point: Pos2) {
    let half = tokens::CURSOR_SIZE / 2.0;
    let stroke = Stroke::new(border::THIN, tokens::MUTED_INK);
    painter.line_segment(
        [pos2(point.x - half, point.y), pos2(point.x + half, point.y)],
        stroke,
    );
    painter.line_segment(
        [pos2(point.x, point.y - half), pos2(point.x, point.y + half)],
        stroke,
    );
}
