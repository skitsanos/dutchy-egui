use crate::{
    ControlSize, Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus,
    tokens::{border, close as tint, space},
};
use egui::{Color32, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2};
pub(super) fn action(ui: &mut Ui, id: Id, label: &str, color: Color32) -> Response {
    let padding = ControlSize::Small.padding();
    let width = ui.available_width();
    let job = Text::new(label)
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .tracking(Tracking::Wide)
        .uppercase(true)
        .color(color)
        .layout_job(
            &Theme::from_context(ui.ctx()),
            (width - padding * 2.0).max(0.0),
        );
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let height = ControlSize::Small
        .height()
        .max(galley.size().y + space::XS * 2.0);
    let (_, rect) = ui.allocate_space(vec2(width, height));
    let response = ui.interact(rect, id, Sense::click());
    response
        .widget_info(|| WidgetInfo::labeled(egui::accesskit::Role::Button, ui.is_enabled(), label));
    if response.enabled() && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        let opacity = if response.is_pointer_button_down_on() {
            tint::PRESSED_TINT
        } else {
            tint::HOVER_TINT
        };
        ui.painter()
            .rect_filled(rect, 0, color.gamma_multiply(opacity));
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    painter.rect_stroke(
        rect,
        0,
        Stroke::new(border::STANDARD, color),
        StrokeKind::Inside,
    );
    let position = Rect::from_center_size(rect.center(), galley.size()).min;
    painter.galley(position, galley, color);
    paint_focus(ui, &response, color);
    response
}
