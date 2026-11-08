use super::state::format_hex;
use crate::{
    ControlSize, Family, Text, TextColor, Theme,
    interaction::paint_focus_inside,
    tokens::{border, color_picker as geometry, space},
};
use egui::{Color32, Rect, Response, Shape, Stroke, StrokeKind, Ui, pos2, vec2};

pub(super) fn swatch(ui: &Ui, rect: Rect, color: Color32) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    if color.a() == 255 {
        painter.rect_filled(rect, 0, color);
        return;
    }
    painter.rect_filled(rect, 0, p.background);
    let tile = geometry::CHECKER;
    let columns = (rect.width() / tile).ceil() as usize;
    let rows = (rect.height() / tile).ceil() as usize;
    for y in 0..rows {
        for x in 0..columns {
            if (x + y) % 2 == 0 {
                painter.rect_filled(
                    Rect::from_min_size(
                        rect.min + vec2(x as f32 * tile, y as f32 * tile),
                        vec2(tile, tile),
                    ),
                    0,
                    p.border,
                );
            }
        }
    }
    painter.rect_filled(rect, 0, color);
}

pub(super) fn trigger(ui: &Ui, response: &Response, rgba: [u8; 4], alpha: bool, size: ControlSize) {
    let rect = response.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    super::super::style::paint_frame(ui, rect, theme);
    if response.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(border::STANDARD), 0, p.muted);
    }
    let inner = rect.shrink(space::SM);
    let side = geometry::SWATCH
        .min(inner.height())
        .min(inner.width())
        .max(0.0);
    let preview = Rect::from_min_size(
        pos2(inner.left(), rect.center().y - side / 2.0),
        vec2(side, side),
    );
    swatch(
        ui,
        preview,
        Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    ui.painter().rect_stroke(
        preview,
        0,
        Stroke::new(border::THIN, p.border),
        StrokeKind::Inside,
    );
    // Match the trailing breathing room of other framed controls. Reserve
    // the label gap from the glyph itself so long hex values cannot crowd it.
    let center = pos2(rect.right() - space::LG - space::XS, rect.center().y);
    let text_left = preview.right() + space::MD;
    let text_rect = Rect::from_min_max(
        pos2(text_left, inner.top()),
        pos2(
            (center.x - space::XS - space::MD).max(text_left),
            inner.bottom(),
        ),
    );
    let mut job = Text::new(format_hex(rgba, alpha))
        .family(Family::Mono)
        .size(size.text_size())
        .color(TextColor::Foreground)
        .layout_job(&theme, text_rect.width().max(0.0));
    job.wrap.max_rows = 1;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.painter()
        .with_clip_rect(text_rect.intersect(ui.clip_rect()))
        .galley(
            pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0),
            galley,
            p.foreground,
        );
    ui.painter().add(Shape::line(
        vec![
            center + vec2(-space::XS, -space::MICRO),
            center + vec2(0.0, space::MICRO),
            center + vec2(space::XS, -space::MICRO),
        ],
        Stroke::new(border::STANDARD, p.foreground),
    ));
    paint_focus_inside(ui, response, p.primary);
}

pub(super) fn preset(ui: &Ui, response: &Response, color: Color32, selected: bool) {
    let p = Theme::from_context(ui.ctx()).palette;
    swatch(ui, response.rect.shrink(border::STANDARD), color);
    let stroke = if response.hovered() || selected {
        p.primary
    } else {
        p.border
    };
    ui.painter().rect_stroke(
        response.rect,
        0,
        Stroke::new(border::STANDARD, stroke),
        StrokeKind::Inside,
    );
    if selected {
        ui.painter().rect_stroke(
            response.rect.shrink(border::STANDARD),
            0,
            Stroke::new(border::STANDARD, p.foreground),
            StrokeKind::Inside,
        );
    }
    paint_focus_inside(ui, response, p.primary);
}
