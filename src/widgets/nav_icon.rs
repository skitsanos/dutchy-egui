use crate::tokens::{border, navigation as geometry};
use egui::{Color32, Painter, Rect, Stroke, pos2};

pub(super) fn chevron(painter: &Painter, rect: Rect, color: Color32, right: bool) {
    let rect = rect.shrink(geometry::ICON_INSET);
    let (start, end) = if right {
        (rect.left(), rect.right())
    } else {
        (rect.right(), rect.left())
    };
    painter.add(egui::Shape::line(
        vec![
            pos2(start, rect.top()),
            pos2(end, rect.center().y),
            pos2(start, rect.bottom()),
        ],
        Stroke::new(border::STANDARD, color),
    ));
}
