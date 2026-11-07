use super::{FileUpload, FileUploadVariant, icon};
use crate::{
    ControlSize, Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus_inside,
    tokens::{border, space, upload as tokens},
};
use egui::{Rect, Response, Stroke, Ui, pos2, vec2};
use std::sync::Arc;
pub(super) struct Layout {
    pub rect: Rect,
    label: Arc<egui::Galley>,
    detail: Option<Arc<egui::Galley>>,
    content_height: f32,
    compact: bool,
}
pub(super) fn allocate(ui: &mut Ui, config: &FileUpload<'_>) -> Layout {
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    let compact = config.variant == FileUploadVariant::Compact;
    let width = if ui.available_width().is_finite() {
        ui.available_width().max(1.0)
    } else {
        crate::tokens::image::DEFAULT_WIDTH
    };
    let inset = if compact { space::MD } else { tokens::PADDING }.min(width / 4.0);
    let width_text = (width - inset * 2.0).max(1.0);
    // Live instruction: bold 14-point body face, uppercase, 0.05 em tracking.
    let mut job = Text::new(config.label.as_ref())
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .tracking(Tracking::Wider)
        .uppercase(true)
        .color(p.foreground)
        .layout_job(&theme, width_text);
    job.halign = egui::Align::Center;
    let label = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let detail = (!compact).then(|| {
        let text = config.hint.as_ref().map_or_else(
            || config.description.to_string(),
            |hint| format!("{}\n{hint}", config.description),
        );
        let mut job = Text::new(text)
            .size(TextSize::Sm)
            .color(p.muted_foreground)
            .layout_job(&theme, width_text);
        job.halign = egui::Align::Center;
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    });
    let content_height = label.size().y
        + detail.as_ref().map_or(0.0, |text| {
            tokens::ICON_SIZE + space::LG + space::SM + text.size().y
        });
    let height = (content_height + inset * 2.0).max(if compact {
        ControlSize::Large.height()
    } else {
        tokens::DROP_HEIGHT
    });
    let (_, rect) = ui.allocate_space(vec2(width, height));
    Layout {
        rect,
        label,
        detail,
        content_height,
        compact,
    }
}
pub(super) fn show(ui: &Ui, layout: &Layout, response: &Response, emphasized: bool, enabled: bool) {
    let rect = layout.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    if emphasized {
        painter.rect_filled(
            rect,
            0,
            p.muted
                .gamma_multiply(if response.is_pointer_button_down_on() {
                    tokens::PRESSED_TINT
                } else {
                    tokens::HOVER_TINT
                }),
        );
    }
    let stroke = Stroke::new(
        border::STANDARD,
        if emphasized { p.primary } else { p.border },
    );
    let outline = rect.shrink(border::STANDARD / 2.0);
    if layout.compact {
        painter.rect_stroke(outline, 0, stroke, egui::StrokeKind::Middle);
    } else {
        painter.extend(egui::Shape::dashed_line(
            &[
                outline.left_top(),
                outline.right_top(),
                outline.right_bottom(),
                outline.left_bottom(),
                outline.left_top(),
            ],
            stroke,
            tokens::DASH,
            tokens::DASH_GAP,
        ));
    }
    let mut y = rect.center().y - layout.content_height / 2.0;
    if !layout.compact {
        icon::upload(
            ui,
            Rect::from_center_size(
                pos2(rect.center().x, y + tokens::ICON_SIZE / 2.0),
                vec2(tokens::ICON_SIZE, tokens::ICON_SIZE),
            ),
            p.muted_foreground,
        );
        y += tokens::ICON_SIZE + space::LG;
    }
    painter.galley(pos2(rect.center().x, y), layout.label.clone(), p.foreground);
    if let Some(detail) = &layout.detail {
        painter.galley(
            pos2(rect.center().x, y + layout.label.size().y + space::SM),
            detail.clone(),
            p.muted_foreground,
        );
    }
    if enabled {
        paint_focus_inside(ui, response, p.primary);
    }
}
