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
    let dot = |center| {
        ui.painter().rect_filled(
            egui::Rect::from_center_size(center, vec2(border::STANDARD, border::STANDARD)),
            0,
            color,
        );
    };
    // Circle-backed glyphs follow the live info, check-circle, alert-circle
    // and x-circle icons; proportions are the icons' 24-unit paths.
    ui.painter().circle_stroke(
        rect.center(),
        rect.width() / 2.0 - border::STANDARD / 2.0,
        stroke,
    );
    match variant {
        AlertVariant::Success => line(vec![
            point(0.375, 0.5),
            point(0.46, 0.585),
            point(0.625, 0.415),
        ]),
        AlertVariant::Error => {
            line(vec![point(0.375, 0.375), point(0.625, 0.625)]);
            line(vec![point(0.375, 0.625), point(0.625, 0.375)]);
        }
        AlertVariant::Info => {
            dot(point(0.5, 0.33));
            line(vec![point(0.5, 0.5), point(0.5, 0.7)]);
        }
        AlertVariant::Warning => {
            line(vec![point(0.5, 0.3), point(0.5, 0.5)]);
            dot(point(0.5, 0.67));
        }
    }
}

pub(in crate::widgets) fn dismiss(
    ui: &mut Ui,
    label: &str,
    idle: Color32,
    color: Color32,
) -> Response {
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
            idle,
            foreground: color,
            focus: color,
        },
    );
    response
}
