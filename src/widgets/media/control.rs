use crate::tokens::Palette;
use crate::{
    Text, TextSize, Theme, Weight,
    interaction::paint_focus_inside,
    tokens::{border, image as geometry},
};
use egui::{Id, Rect, Response, Sense, Stroke, Ui, WidgetInfo, pos2, vec2};

#[derive(Clone, Copy)]
pub(super) enum Icon {
    Expand,
    Previous,
    Next,
    Close,
}
pub(super) fn icon(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    label: &str,
    icon: Icon,
    p: Palette,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    if response.clicked() {
        response.request_focus();
    }
    response
        .widget_info(|| WidgetInfo::labeled(egui::accesskit::Role::Button, ui.is_enabled(), label));
    let hover = response.enabled() && (response.hovered() || response.is_pointer_button_down_on());
    let (fill, foreground) = if hover {
        (p.primary, p.on_primary)
    } else {
        (
            p.foreground.gamma_multiply(geometry::EXPAND_OPACITY),
            p.background,
        )
    };
    ui.painter().rect_filled(rect, 0, fill);
    let extent = if matches!(icon, Icon::Expand) {
        if rect.width() <= geometry::GALLERY_EXPAND_SIZE {
            geometry::GALLERY_EXPAND_ICON
        } else {
            geometry::EXPAND_ICON
        }
    } else {
        geometry::VIEWER_ICON
    };
    paint(ui, rect, extent, icon, foreground);
    paint_focus_inside(ui, &response, p.primary);
    if response.enabled() && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}
fn paint(ui: &Ui, rect: Rect, extent: f32, icon: Icon, color: egui::Color32) {
    let r = Rect::from_center_size(
        rect.center(),
        egui::Vec2::splat(extent.min(rect.width()).min(rect.height())),
    );
    let point = |x: f32, y: f32| r.min + vec2(x * r.width(), y * r.height());
    let segments: Vec<_> = match icon {
        Icon::Close => vec![
            [r.left_top(), r.right_bottom()],
            [r.right_top(), r.left_bottom()],
        ],
        Icon::Previous => vec![
            [point(0.65, 0.2), point(0.35, 0.5)],
            [point(0.35, 0.5), point(0.65, 0.8)],
        ],
        Icon::Next => vec![
            [point(0.35, 0.2), point(0.65, 0.5)],
            [point(0.65, 0.5), point(0.35, 0.8)],
        ],
        Icon::Expand => vec![
            [point(0.6, 0.0), point(1.0, 0.0)],
            [point(1.0, 0.0), point(1.0, 0.4)],
            [point(1.0, 0.0), point(0.55, 0.45)],
            [point(0.0, 0.6), point(0.0, 1.0)],
            [point(0.0, 1.0), point(0.4, 1.0)],
            [point(0.0, 1.0), point(0.45, 0.55)],
        ],
    };
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    for line in segments {
        painter.line_segment(line, Stroke::new(border::STANDARD, color));
    }
}
pub(super) fn retry(ui: &mut Ui, id: Id, rect: Rect, label: &str, p: Palette) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    if response.clicked() {
        response.request_focus();
    }
    response
        .widget_info(|| WidgetInfo::labeled(egui::accesskit::Role::Button, ui.is_enabled(), label));
    let hover = response.enabled() && response.hovered();
    ui.painter()
        .rect_filled(rect, 0, if hover { p.primary } else { p.muted });
    let job = Text::new("Retry image")
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .uppercase(true)
        .color(if hover { p.on_primary } else { p.foreground })
        .layout_job(&Theme::from_context(ui.ctx()), rect.width());
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        p.foreground,
    );
    paint_focus_inside(ui, &response, p.primary);
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}
