use super::AlertVariant;
use crate::{
    interaction::paint_focus,
    tokens::{border, surface},
};
use egui::{Color32, Response, Sense, Stroke, Ui, pos2, vec2};

pub(super) fn status(ui: &mut Ui, variant: AlertVariant, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        vec2(surface::STATUS_ICON, surface::STATUS_ICON),
        Sense::hover(),
    );
    let point = |x, y| rect.min + vec2(x * rect.width(), y * rect.height());
    let stroke = Stroke::new(border::STANDARD, color);
    let line = |points| {
        ui.painter().add(egui::Shape::line(points, stroke));
    };
    match variant {
        AlertVariant::Success => line(vec![point(0.1, 0.5), point(0.4, 0.8), point(0.9, 0.2)]),
        AlertVariant::Error => {
            line(vec![point(0.2, 0.2), point(0.8, 0.8)]);
            line(vec![point(0.2, 0.8), point(0.8, 0.2)]);
        }
        AlertVariant::Info | AlertVariant::Warning => {
            if variant == AlertVariant::Warning {
                line(vec![
                    point(0.5, 0.05),
                    point(0.95, 0.9),
                    point(0.05, 0.9),
                    point(0.5, 0.05),
                ]);
            }
            let (dot, a, b) = if variant == AlertVariant::Info {
                (point(0.5, 0.15), point(0.5, 0.4), point(0.5, 0.85))
            } else {
                (point(0.5, 0.72), point(0.5, 0.3), point(0.5, 0.5))
            };
            ui.painter().rect_filled(
                egui::Rect::from_center_size(dot, vec2(border::STANDARD, border::STANDARD)),
                0,
                color,
            );
            line(vec![a, b]);
        }
    }
}

pub(super) fn dismiss(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            ui.spacing_mut().interact_size = vec2(
                crate::ControlSize::Small.height(),
                crate::ControlSize::Small.height(),
            );
            ui.add(egui::Button::new("").frame(false))
                .accessible_name(label)
        })
        .inner;
    let rect = response.rect.shrink(crate::tokens::space::SM);
    let stroke = Stroke::new(border::STANDARD, color);
    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([pos2(rect.left(), rect.bottom()), rect.right_top()], stroke);
    paint_focus(ui, &response, color);
    response
}
