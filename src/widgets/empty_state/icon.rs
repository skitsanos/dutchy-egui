use super::{EmptyStateIcon, EmptyStateVariant};
use crate::{
    Theme,
    tokens::{border, space, surface},
};
use egui::{Sense, Stroke, StrokeKind, Ui, vec2};

pub(super) fn show(ui: &mut Ui, icon: EmptyStateIcon, variant: EmptyStateVariant, side: f32) {
    if icon == EmptyStateIcon::None {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let (rect, decorative) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
    decorative
        .ctx
        .accesskit_node_builder(decorative.id, |node| node.set_hidden());
    let dashboard = variant == EmptyStateVariant::Dashboard;
    ui.painter().rect_filled(
        rect,
        0,
        if dashboard {
            p.background.lerp_to_gamma(p.primary, surface::STATUS_TINT)
        } else {
            p.muted
        },
    );
    let rect = rect.shrink(side / 4.0);
    let point = |x, y| rect.min + vec2(x * rect.width(), y * rect.height());
    let stroke = Stroke::new(
        border::STANDARD,
        if dashboard {
            p.primary
        } else {
            p.muted_foreground
        },
    );
    match icon {
        EmptyStateIcon::None => {}
        EmptyStateIcon::Search => {
            let glass = egui::Rect::from_min_max(point(0.05, 0.05), point(0.7, 0.7));
            ui.painter()
                .rect_stroke(glass, 0, stroke, StrokeKind::Inside);
            ui.painter()
                .line_segment([point(0.68, 0.68), point(0.95, 0.95)], stroke);
        }
        EmptyStateIcon::Inbox => {
            ui.painter().rect_stroke(
                egui::Rect::from_min_max(point(0.05, 0.15), point(0.95, 0.85)),
                0,
                stroke,
                StrokeKind::Inside,
            );
            ui.painter().add(egui::Shape::line(
                vec![
                    point(0.05, 0.55),
                    point(0.3, 0.55),
                    point(0.4, 0.7),
                    point(0.6, 0.7),
                    point(0.7, 0.55),
                    point(0.95, 0.55),
                ],
                stroke,
            ));
        }
        EmptyStateIcon::Dashboard => {
            for (x, y) in [(0.0, 0.0), (0.6, 0.0), (0.0, 0.6), (0.6, 0.6)] {
                ui.painter().rect_stroke(
                    egui::Rect::from_min_max(point(x, y), point(x + 0.4, y + 0.4)),
                    0,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        }
    }
    ui.add_space(space::XXL);
}
