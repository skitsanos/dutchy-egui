use super::AlertVariant;
use crate::{
    ControlSize,
    tokens::{border, close as geometry, surface},
    widgets::close,
};
use egui::{Color32, Response, Sense, Stroke, Ui, WidgetInfo, vec2};

pub(in crate::widgets) fn status(ui: &mut Ui, variant: AlertVariant, color: Color32) {
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

pub(in crate::widgets) fn dismiss(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let (_, rect) = ui.allocate_space(vec2(
        ControlSize::Small.height(),
        ControlSize::Small.height(),
    ));
    let response = ui.interact(rect, ui.make_persistent_id("dismiss"), Sense::click());
    response
        .widget_info(|| WidgetInfo::labeled(egui::accesskit::Role::Button, ui.is_enabled(), label));
    close::paint(
        ui,
        &response,
        geometry::ALERT_ICON,
        close::Colors {
            idle: color.gamma_multiply(geometry::IDLE_OPACITY),
            foreground: color,
            focus: color,
        },
    );
    response
}
