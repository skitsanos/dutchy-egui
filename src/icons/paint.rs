use super::Icon;
use crate::tokens::border;
use egui::{Color32, Painter, Rect, Stroke, pos2};
type Point = (f32, f32);

pub(super) fn show(icon: Icon, p: &Painter, r: Rect, color: Color32) {
    let stroke = Stroke::new(border::STANDARD, color);
    match icon {
        Icon::Search => {
            p.circle_stroke(
                point(r, (0.4, 0.4)),
                r.width().min(r.height()) * 0.27,
                stroke,
            );
            line(p, r, &[(0.6, 0.6), (0.88, 0.88)], stroke);
        }
        Icon::More => {
            for x in [0.2, 0.5, 0.8] {
                p.circle_filled(point(r, (x, 0.5)), border::STANDARD, color);
            }
        }
        Icon::Grid => {
            for (x, y) in [(0.12, 0.12), (0.58, 0.12), (0.12, 0.58), (0.58, 0.58)] {
                box_path(p, r, (x, y), (x + 0.3, y + 0.3), stroke);
            }
        }
        Icon::List => {
            for y in [0.2, 0.5, 0.8] {
                line(p, r, &[(0.12, y), (0.2, y)], stroke);
                line(p, r, &[(0.36, y), (0.88, y)], stroke);
            }
        }
        Icon::AlignLeft | Icon::AlignCenter | Icon::AlignRight => alignment(p, r, icon, stroke),
        _ => {
            for path in paths(icon) {
                line(p, r, path, stroke);
            }
        }
    }
}
fn point(r: Rect, (x, y): Point) -> egui::Pos2 {
    pos2(r.left() + r.width() * x, r.top() + r.height() * y)
}
fn line(p: &Painter, r: Rect, path: &[Point], stroke: Stroke) {
    p.add(egui::Shape::line(
        path.iter().map(|xy| point(r, *xy)).collect(),
        stroke,
    ));
}
fn box_path(p: &Painter, r: Rect, (x, y): Point, (right, bottom): Point, stroke: Stroke) {
    line(
        p,
        r,
        &[(x, y), (right, y), (right, bottom), (x, bottom), (x, y)],
        stroke,
    );
}
fn alignment(p: &Painter, r: Rect, icon: Icon, stroke: Stroke) {
    for (index, y) in [0.15, 0.38, 0.62, 0.85].into_iter().enumerate() {
        let short = index % 2 == 1;
        let (left, right) = match (icon, short) {
            (_, false) => (0.12, 0.88),
            (Icon::AlignCenter, true) => (0.3, 0.7),
            (Icon::AlignRight, true) => (0.4, 0.88),
            _ => (0.12, 0.6),
        };
        line(p, r, &[(left, y), (right, y)], stroke);
    }
}
fn paths(icon: Icon) -> &'static [&'static [Point]] {
    match icon {
        Icon::Plus => &[&[(0.5, 0.12), (0.5, 0.88)], &[(0.12, 0.5), (0.88, 0.5)]],
        Icon::Minus => &[&[(0.12, 0.5), (0.88, 0.5)]],
        Icon::Close => &[&[(0.18, 0.18), (0.82, 0.82)], &[(0.82, 0.18), (0.18, 0.82)]],
        Icon::Check => &[&[(0.1, 0.5), (0.38, 0.78), (0.9, 0.2)]],
        Icon::Copy => &[
            &[
                (0.65, 0.18),
                (0.65, 0.1),
                (0.1, 0.1),
                (0.1, 0.65),
                (0.18, 0.65),
            ],
            &[
                (0.35, 0.35),
                (0.9, 0.35),
                (0.9, 0.9),
                (0.35, 0.9),
                (0.35, 0.35),
            ],
        ],
        Icon::Save => &[
            &[
                (0.12, 0.12),
                (0.72, 0.12),
                (0.88, 0.28),
                (0.88, 0.88),
                (0.12, 0.88),
                (0.12, 0.12),
            ],
            &[(0.3, 0.12), (0.3, 0.42), (0.7, 0.42), (0.7, 0.12)],
            &[(0.3, 0.88), (0.3, 0.62), (0.7, 0.62), (0.7, 0.88)],
        ],
        Icon::Download => &[
            &[(0.5, 0.1), (0.5, 0.65)],
            &[(0.25, 0.42), (0.5, 0.67), (0.75, 0.42)],
            &[(0.12, 0.68), (0.12, 0.88), (0.88, 0.88), (0.88, 0.68)],
        ],
        Icon::Trash => &[
            &[(0.1, 0.25), (0.9, 0.25)],
            &[(0.35, 0.25), (0.35, 0.1), (0.65, 0.1), (0.65, 0.25)],
            &[(0.22, 0.25), (0.28, 0.9), (0.72, 0.9), (0.78, 0.25)],
            &[(0.42, 0.4), (0.42, 0.74)],
            &[(0.58, 0.4), (0.58, 0.74)],
        ],
        Icon::Bold => &[
            &[
                (0.25, 0.12),
                (0.25, 0.88),
                (0.62, 0.88),
                (0.78, 0.72),
                (0.78, 0.6),
                (0.62, 0.5),
                (0.25, 0.5),
            ],
            &[
                (0.25, 0.12),
                (0.6, 0.12),
                (0.75, 0.25),
                (0.75, 0.38),
                (0.6, 0.5),
            ],
        ],
        Icon::Italic => &[
            &[(0.4, 0.12), (0.82, 0.12)],
            &[(0.18, 0.88), (0.6, 0.88)],
            &[(0.62, 0.12), (0.38, 0.88)],
        ],
        Icon::Underline => &[
            &[
                (0.25, 0.1),
                (0.25, 0.55),
                (0.4, 0.7),
                (0.6, 0.7),
                (0.75, 0.55),
                (0.75, 0.1),
            ],
            &[(0.15, 0.9), (0.85, 0.9)],
        ],
        Icon::Undo => &[
            &[(0.38, 0.15), (0.12, 0.4), (0.38, 0.65)],
            &[(0.12, 0.4), (0.62, 0.4), (0.85, 0.62), (0.85, 0.85)],
        ],
        Icon::Redo => &[
            &[(0.62, 0.15), (0.88, 0.4), (0.62, 0.65)],
            &[(0.88, 0.4), (0.38, 0.4), (0.15, 0.62), (0.15, 0.85)],
        ],
        Icon::Play => &[&[(0.25, 0.12), (0.88, 0.5), (0.25, 0.88), (0.25, 0.12)]],
        Icon::Pause => &[&[(0.3, 0.12), (0.3, 0.88)], &[(0.7, 0.12), (0.7, 0.88)]],
        _ => &[],
    }
}
