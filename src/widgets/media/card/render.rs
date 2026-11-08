use super::{MediaCard, MediaCardResponse, paint};
use crate::{
    Card, CardPadding, CardVariant, Image, ImageStatus, Text, TextColor, TextSize, Theme,
    interaction::{paint_focus_inside, reveal_focused},
    tokens::{border, media_card as geometry},
};
use egui::{
    Id, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo, accesskit::Role, pos2, vec2,
};

pub(super) fn show(config: MediaCard<'_>, ui: &mut Ui) -> MediaCardResponse {
    let root = ui.make_persistent_id(("dutchy-media-card", config.id));
    let anchor = ui.max_rect().min;
    let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
        if !config.enabled {
            ui.disable();
        }
        let width = super::super::geometry::width(ui.available_width(), config.width);
        ui.set_width(width);
        Card::new()
            .variant(CardVariant::Bordered)
            .padding(CardPadding::None)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut image = Image::new("artwork", config.source.clone(), config.alt.as_ref())
                    .aspect(config.aspect)
                    .fit(config.fit)
                    .bordered(false)
                    .retryable(config.retryable);
                image.height = config.height;
                image.action = Some(format!("{}: {}", config.action, config.title).into());
                image.reveal_anchor = Some(anchor);
                let image = image.show(ui);
                let preview = image
                    .image
                    .clone()
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let save = config.saved.map(|saved| {
                    let rect = image.image.rect;
                    let side = geometry::SAVE_TARGET.min(rect.width()).min(rect.height());
                    let inset = geometry::SAVE_INSET
                        .min(((rect.width().min(rect.height()) - side) / 2.0).max(0.0));
                    let rect = egui::Rect::from_min_size(
                        pos2(rect.right() - inset - side, rect.top() + inset),
                        vec2(side, side),
                    );
                    let response = ui.interact(rect, root.with("save"), Sense::click());
                    let verb = if saved { "Unsave" } else { "Save" };
                    response.widget_info(|| {
                        WidgetInfo::selected(
                            Role::Button,
                            ui.is_enabled(),
                            saved,
                            format!("{verb} {}", config.title),
                        )
                    });
                    if response.clicked() {
                        response.request_focus();
                    }
                    paint::save(ui, &response, saved);
                    response
                });
                if image.status == ImageStatus::Ready {
                    paint::preview(ui, &preview, &config.action);
                }
                paint_focus_inside(ui, &preview, Theme::from_context(ui.ctx()).palette.primary);
                let details = details(ui, root, &config);
                for response in [&preview, &details].into_iter().chain(save.iter()) {
                    reveal_focused(ui, response, anchor);
                }
                (image, preview, details, save)
            })
            .inner
    });
    let (image, preview, details, save) = result.inner;
    result.response.widget_info(|| {
        WidgetInfo::labeled(
            Role::Group,
            result.response.enabled(),
            format!("Media card {}", config.title),
        )
    });
    let p = Theme::from_context(ui.ctx()).palette;
    if preview.hovered() || details.hovered() || save.as_ref().is_some_and(Response::hovered) {
        ui.painter().rect_stroke(
            result.response.rect,
            0,
            Stroke::new(border::STANDARD, p.primary),
            StrokeKind::Inside,
        );
    }
    MediaCardResponse {
        response: result.response,
        open_requested: preview.clicked() || details.clicked(),
        save_requested: save
            .as_ref()
            .filter(|s| s.clicked())
            .map(|_| !config.saved.unwrap_or(false)),
        image,
        preview,
        details,
        save,
    }
}

fn details(ui: &mut Ui, root: Id, config: &MediaCard<'_>) -> Response {
    let p = Theme::from_context(ui.ctx()).palette;
    let result = ui.scope_builder(
        UiBuilder::new()
            .scope_id(root.with("details"))
            .sense(Sense::click()),
        |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            let hovered = ui.is_enabled() && ui.response().contains_pointer();
            egui::Frame::NONE
                .fill(if hovered { p.muted } else { p.background })
                .inner_margin(egui::Margin::same(geometry::PADDING as i8))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = geometry::GAP;
                    if let Some(category) = &config.category {
                        ui.add(Text::label(category.as_ref()).color(TextColor::Muted));
                    }
                    paint::title(ui, &config.title);
                    if let Some(description) = &config.description {
                        let mut job = Text::new(description.as_ref())
                            .size(TextSize::Sm)
                            .color(TextColor::Muted)
                            .layout_job(&Theme::from_context(ui.ctx()), ui.available_width());
                        job.wrap.max_rows = geometry::DESCRIPTION_ROWS;
                        ui.add(egui::Label::new(job).wrap());
                    }
                });
        },
    );
    let response = result
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| {
        WidgetInfo::labeled(
            Role::Button,
            ui.is_enabled(),
            format!("Open {}", config.title),
        )
    });
    if response.clicked() {
        response.request_focus();
    }
    paint_focus_inside(ui, &response, p.primary);
    response
}
