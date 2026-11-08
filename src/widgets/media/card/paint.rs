use crate::{
    Text, TextColor, TextSize, Theme, Tracking,
    interaction::{focus_is_visible, paint_focus_inside},
    tokens::{border, media_card as tokens},
};
use egui::{Align, Layout, Rect, Response, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2};

pub(super) fn save(ui: &Ui, response: &Response, saved: bool) {
    if !ui.is_rect_visible(response.rect) {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let emphasized =
        response.enabled() && (response.hovered() || response.is_pointer_button_down_on());
    let (fill, ink) = if emphasized {
        (p.primary, p.on_primary)
    } else if saved {
        (p.foreground, p.background)
    } else {
        (p.background, p.foreground)
    };
    let painter = ui
        .painter()
        .with_clip_rect(ui.clip_rect().intersect(response.rect));
    painter.rect(
        response.rect,
        0,
        fill,
        Stroke::new(border::STANDARD, p.foreground),
        StrokeKind::Inside,
    );
    let side = tokens::ICON
        .min(response.rect.width())
        .min(response.rect.height());
    let r = Rect::from_center_size(response.rect.center(), vec2(side, side));
    let points = vec![
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.center_bottom() - vec2(0.0, side / 4.0),
        r.left_bottom(),
        r.left_top(),
    ];
    painter.add(egui::Shape::line(
        points,
        Stroke::new(border::STANDARD, ink),
    ));
    paint_focus_inside(ui, response, p.primary);
    if emphasized {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

pub(super) fn preview(ui: &Ui, response: &Response, label: &str) {
    if !ui.is_rect_visible(response.rect)
        || !response.enabled()
        || !(response.contains_pointer() || focus_is_visible(response))
    {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let height = crate::ControlSize::Small
        .height()
        .min(response.rect.height());
    let rect = Rect::from_min_max(
        pos2(response.rect.left(), response.rect.bottom() - height),
        response.rect.max,
    );
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    painter.rect_filled(rect, 0, p.primary);
    let mut job = Text::label(label)
        .color(TextColor::Custom(p.on_primary))
        .layout_job(
            &Theme::from_context(ui.ctx()),
            (rect.width() - 2.0 * tokens::PADDING).max(0.0),
        );
    job.wrap.max_rows = 1;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    painter.galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        p.on_primary,
    );
}

pub(super) fn title(ui: &mut Ui, title: &str) {
    let p = Theme::from_context(ui.ctx()).palette;
    let width = (ui.available_width() - tokens::ICON - tokens::GAP).max(0.0);
    let mut job = Text::heading(title)
        .size(TextSize::Xl)
        .tracking(Tracking::Normal)
        .color(TextColor::Foreground)
        .layout_job(&Theme::from_context(ui.ctx()), width);
    job.wrap.max_rows = tokens::TITLE_ROWS;
    // This row owns its alignment: a host's Center/Right placement must not
    // move the measured label back into the reserved arrow column.
    let response = ui
        .scope_builder(
            UiBuilder::new().layout(Layout::top_down(Align::Min)),
            |ui| {
                ui.set_min_width(ui.available_width());
                crate::text::add_measured(ui, job)
            },
        )
        .inner;
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Heading);
        node.set_label(title);
        node.set_level(3);
    });
    let right = ui.max_rect().right();
    let r = Rect::from_center_size(
        pos2(
            right - tokens::ICON / 2.0,
            response.rect.top() + tokens::ICON / 2.0,
        ),
        vec2(tokens::ICON, tokens::ICON),
    );
    let painter = ui.painter().with_clip_rect(ui.clip_rect());
    let stroke = Stroke::new(border::STANDARD, p.foreground);
    painter.line_segment([r.left_bottom(), r.right_top()], stroke);
    painter.line_segment([r.left_top(), r.right_top()], stroke);
    painter.line_segment([r.right_top(), r.right_bottom()], stroke);
}
