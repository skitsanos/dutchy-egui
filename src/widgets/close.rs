use crate::{
    interaction::{focus_is_visible, paint_focus, paint_focus_inside},
    tokens::{border, close as tokens},
};
use egui::{Color32, Rect, Response, Stroke, Ui, vec2};

pub(super) struct Colors {
    pub idle: Color32,
    pub foreground: Color32,
    pub focus: Color32,
}

pub(super) fn paint(ui: &Ui, response: &Response, icon_size: f32, colors: Colors) {
    paint_control(ui, response, response.rect, icon_size, &colors);
    paint_focus(ui, response, colors.focus);
}

pub(super) fn paint_inside(
    ui: &Ui,
    response: &Response,
    inside: Rect,
    icon_size: f32,
    colors: Colors,
) {
    let extent = icon_size
        .min(response.rect.width())
        .min(response.rect.height());
    paint_control(
        ui,
        response,
        inside.intersect(response.rect),
        extent,
        &colors,
    );
    paint_focus_inside(ui, response, colors.focus);
}

fn paint_control(ui: &Ui, response: &Response, bounds: Rect, icon_size: f32, colors: &Colors) {
    let hovered = response.enabled() && response.hovered();
    let pressed = response.enabled() && response.is_pointer_button_down_on();
    let emphasized = hovered || pressed || focus_is_visible(response);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if ui.is_rect_visible(response.rect) {
        let painter = ui
            .painter()
            .with_clip_rect(ui.clip_rect().intersect(bounds));
        if hovered || pressed {
            let tint = if pressed {
                tokens::PRESSED_TINT
            } else {
                tokens::HOVER_TINT
            };
            painter.rect_filled(response.rect, 0, colors.foreground.gamma_multiply(tint));
        }
        let icon = Rect::from_center_size(response.rect.center(), vec2(icon_size, icon_size));
        let color = if emphasized {
            colors.foreground
        } else {
            colors.idle
        };
        let stroke = Stroke::new(border::STANDARD, color);
        painter.line_segment([icon.left_top(), icon.right_bottom()], stroke);
        painter.line_segment([icon.right_top(), icon.left_bottom()], stroke);
    }
}
