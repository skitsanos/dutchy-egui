//! Original geometric glyphs for the mail toolbar.

use dutchy_egui::tokens::border;
use egui::{Color32, Painter, Rect, Stroke, pos2};

pub fn archive(painter: &Painter, bounds: Rect, color: Color32) {
    let stroke = Stroke::new(border::STANDARD, color);
    path(
        painter,
        bounds,
        &[
            (0.14, 0.18),
            (0.86, 0.18),
            (0.86, 0.38),
            (0.14, 0.38),
            (0.14, 0.18),
        ],
        stroke,
    );
    path(
        painter,
        bounds,
        &[(0.21, 0.38), (0.21, 0.84), (0.79, 0.84), (0.79, 0.38)],
        stroke,
    );
    path(painter, bounds, &[(0.39, 0.55), (0.61, 0.55)], stroke);
}

pub fn envelope(painter: &Painter, bounds: Rect, color: Color32) {
    let stroke = Stroke::new(border::STANDARD, color);
    path(
        painter,
        bounds,
        &[
            (0.13, 0.24),
            (0.87, 0.24),
            (0.87, 0.76),
            (0.13, 0.76),
            (0.13, 0.24),
        ],
        stroke,
    );
    path(
        painter,
        bounds,
        &[(0.13, 0.28), (0.5, 0.56), (0.87, 0.28)],
        stroke,
    );
}

fn path(painter: &Painter, bounds: Rect, points: &[(f32, f32)], stroke: Stroke) {
    painter.add(egui::Shape::line(
        points
            .iter()
            .map(|(x, y)| {
                pos2(
                    bounds.left() + bounds.width() * x,
                    bounds.top() + bounds.height() * y,
                )
            })
            .collect(),
        stroke,
    ));
}
