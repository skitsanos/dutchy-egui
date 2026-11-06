use crate::tokens::tooltip::ARROW;
use egui::{Color32, Pos2, Response, Shape, Stroke, pos2};

pub(super) fn arrow(anchor: &Response, tooltip: &Response, fill: Color32) {
    let rect = tooltip.rect;
    let target = anchor.interact_rect;
    if rect.width() < ARROW * 2.0 || rect.height() < ARROW * 2.0 {
        return;
    }
    let x = target
        .center()
        .x
        .clamp(rect.left() + ARROW, rect.right() - ARROW);
    let y = target
        .center()
        .y
        .clamp(rect.top() + ARROW, rect.bottom() - ARROW);
    let points: [Pos2; 3] = if rect.bottom() <= target.top() {
        [
            pos2(x - ARROW, rect.bottom()),
            pos2(x + ARROW, rect.bottom()),
            pos2(x, rect.bottom() + ARROW),
        ]
    } else if rect.top() >= target.bottom() {
        [
            pos2(x - ARROW, rect.top()),
            pos2(x + ARROW, rect.top()),
            pos2(x, rect.top() - ARROW),
        ]
    } else if rect.right() <= target.left() {
        [
            pos2(rect.right(), y - ARROW),
            pos2(rect.right(), y + ARROW),
            pos2(rect.right() + ARROW, y),
        ]
    } else if rect.left() >= target.right() {
        [
            pos2(rect.left(), y - ARROW),
            pos2(rect.left(), y + ARROW),
            pos2(rect.left() - ARROW, y),
        ]
    } else {
        return;
    };
    anchor
        .ctx
        .layer_painter(tooltip.layer_id)
        .add(Shape::convex_polygon(points.to_vec(), fill, Stroke::NONE));
}
