use crate::tokens::border;
use egui::{Color32, Rect, Stroke, Ui, pos2};
pub(super) fn upload(ui: &Ui, rect: Rect, color: Color32) {
    let p = |x: f32, y: f32| {
        pos2(
            rect.left() + x * rect.width(),
            rect.top() + y * rect.height(),
        )
    };
    let stroke = Stroke::new(border::STANDARD, color);
    for points in [
        vec![p(0.15, 0.65), p(0.15, 0.9), p(0.85, 0.9), p(0.85, 0.65)],
        vec![p(0.25, 0.4), p(0.5, 0.15), p(0.75, 0.4)],
        vec![p(0.5, 0.15), p(0.5, 0.75)],
    ] {
        ui.painter().add(egui::Shape::line(points, stroke));
    }
}
pub(super) fn file(ui: &Ui, rect: Rect, color: Color32) {
    let stroke = Stroke::new(border::STANDARD, color);
    ui.painter()
        .rect_stroke(rect, 0, stroke, egui::StrokeKind::Inside);
    for y in [0.35, 0.65] {
        ui.painter().line_segment(
            [
                pos2(
                    rect.left() + rect.width() * 0.25,
                    rect.top() + rect.height() * y,
                ),
                pos2(
                    rect.left() + rect.width() * 0.75,
                    rect.top() + rect.height() * y,
                ),
            ],
            stroke,
        );
    }
}
