use super::{
    Image, ImageResponse, ImageStatus, caption,
    control::{self, Icon},
    geometry, loading,
};
use crate::{
    ControlSize, Text, TextSize, Theme,
    tokens::{border, image as tokens, space},
};
use egui::{
    Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo,
    load::{TextureLoadResult, TexturePoll},
    pos2, vec2,
};

pub(super) fn show(config: Image<'_>, ui: &mut Ui) -> ImageResponse {
    let root = ui.make_persistent_id(("dutchy-image", config.id));
    let anchor = config.reveal_anchor.unwrap_or(ui.max_rect().min);
    let p = config
        .surface
        .unwrap_or_else(|| Theme::from_context(ui.ctx()).palette);
    let result = ui.scope_builder(
        UiBuilder::new()
            .scope_id(root)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            if !config.enabled {
                ui.disable();
            }
            let width = geometry::width(ui.available_width(), config.width);
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 0.0;
            let saved = ui
                .ctx()
                .data(|data| data.get_temp(root.with("intrinsic-size")));
            let intrinsic = config
                .source
                .texture_size()
                .filter(|size| geometry::valid(*size))
                .or(saved);
            let hint = vec2(
                width,
                config
                    .height
                    .unwrap_or(width / geometry::aspect(config.aspect, intrinsic)),
            );
            let candidate = Rect::from_min_size(ui.cursor().min, hint);
            let load = if ui.is_rect_visible(candidate) || config.source.texture_size().is_some() {
                egui::Image::new(config.source.clone())
                    .fit_to_exact_size(hint)
                    .load_for_size(ui.ctx(), hint)
            } else {
                Ok(TexturePoll::Pending { size: intrinsic })
            };
            let intrinsic = load
                .as_ref()
                .ok()
                .and_then(TexturePoll::size)
                .filter(|size| geometry::valid(*size))
                .or(intrinsic);
            if let Some(size) = intrinsic {
                ui.ctx()
                    .data_mut(|data| data.insert_temp(root.with("intrinsic-size"), size));
            }
            let height = config
                .height
                .unwrap_or(width / geometry::aspect(config.aspect, intrinsic))
                .min(tokens::MAX_DIMENSION);
            let (_, rect) = ui.allocate_space(vec2(width, height));
            let image = ui.interact(
                rect,
                root.with("image"),
                if config.action.is_some() {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            if image.clicked() {
                image.request_focus();
            }
            let status = status(&load);
            image.widget_info(|| {
                WidgetInfo::labeled(
                    if config.action.is_some() {
                        egui::accesskit::Role::Button
                    } else {
                        egui::accesskit::Role::Image
                    },
                    ui.is_enabled(),
                    config.action.as_deref().unwrap_or(config.alt.as_ref()),
                )
            });
            image
                .ctx
                .accesskit_node_builder(image.id, |node| match &status {
                    ImageStatus::Loading => {
                        node.set_busy();
                        node.set_description("Loading image");
                    }
                    ImageStatus::Failed(_) => node.set_description("Image unavailable"),
                    ImageStatus::Ready => {}
                });
            let inset = if config.bordered {
                border::STANDARD
            } else {
                0.0
            };
            let inner = rect.shrink(inset.min(rect.width().min(rect.height()) / 2.0));
            ui.painter().rect_filled(
                rect,
                0,
                if config.surface.is_some() {
                    p.background
                } else {
                    p.muted
                },
            );
            let mut retry = None;
            match (&status, &load) {
                (ImageStatus::Ready, Ok(TexturePoll::Ready { texture })) => {
                    let (destination, uv) = geometry::placement(inner, texture.size, config.fit);
                    ui.painter()
                        .with_clip_rect(ui.clip_rect().intersect(inner))
                        .image(texture.id, destination, uv, egui::Color32::WHITE);
                }
                (ImageStatus::Loading, _) => loading::show(ui, root, inner, config.alt.as_ref(), p),
                _ => {
                    retry = failure(ui, root, inner, &config, p);
                }
            }
            if config.bordered {
                ui.painter().rect_stroke(
                    rect,
                    0,
                    Stroke::new(border::STANDARD, p.border),
                    StrokeKind::Inside,
                );
            }
            let expand = config.expandable.then(|| {
                let extent = config.expand_size.min(inner.width()).min(inner.height());
                let inset = config
                    .expand_inset
                    .min(((inner.width().min(inner.height()) - extent) / 2.0).max(0.0));
                let button = Rect::from_min_size(
                    pos2(inner.right() - inset - extent, inner.top() + inset),
                    egui::Vec2::splat(extent),
                );
                control::icon(
                    ui,
                    root.with("expand"),
                    button,
                    &format!("Expand {}", config.alt),
                    Icon::Expand,
                    config.expand_opacity,
                    p,
                )
            });
            for response in expand.iter().chain(retry.iter()) {
                crate::interaction::reveal_focused(ui, response, anchor);
            }
            let caption_color = if config.surface.is_some() {
                p.muted_foreground
            } else {
                ui.visuals()
                    .override_text_color
                    .unwrap_or(p.muted_foreground)
            };
            caption::show(
                ui,
                config.caption.as_deref(),
                config.caption_source.as_deref(),
                caption_color,
            );
            (image, status, expand, retry)
        },
    );
    result
        .response
        .ctx
        .accesskit_node_builder(result.response.id, |node| {
            node.set_role(egui::accesskit::Role::Figure);
            node.set_label(config.alt.to_string());
        });
    geometry::bounds(&result.response);
    ImageResponse {
        response: result.response,
        image: result.inner.0,
        status: result.inner.1,
        expand_requested: result.inner.2.as_ref().is_some_and(Response::clicked),
        retry_requested: result.inner.3.as_ref().is_some_and(Response::clicked),
        expand: result.inner.2,
        retry: result.inner.3,
    }
}
fn status(load: &TextureLoadResult) -> ImageStatus {
    match load {
        Ok(TexturePoll::Ready { texture }) if geometry::valid(texture.size) => ImageStatus::Ready,
        Ok(TexturePoll::Ready { .. }) => ImageStatus::Failed("Invalid image dimensions".into()),
        Ok(TexturePoll::Pending { .. }) => ImageStatus::Loading,
        Err(error) => ImageStatus::Failed(error.to_string()),
    }
}
fn failure(
    ui: &mut Ui,
    root: Id,
    rect: Rect,
    config: &Image<'_>,
    p: crate::tokens::Palette,
) -> Option<Response> {
    let target = ControlSize::Small.height().min(rect.height());
    if !config.retryable || rect.height() > target * 2.0 + space::LG {
        let job = Text::new("Image unavailable")
            .size(TextSize::Sm)
            .color(p.muted_foreground)
            .layout_job(&Theme::from_context(ui.ctx()), rect.width());
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let offset = if config.retryable {
            target / 2.0 + space::XS
        } else {
            0.0
        };
        ui.painter()
            .with_clip_rect(ui.clip_rect().intersect(rect))
            .galley(
                pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0 - offset,
                ),
                galley,
                p.muted_foreground,
            );
    }
    config.retryable.then(|| {
        let width = tokens::RETRY_WIDTH.min(rect.width());
        let offset = if rect.height() > target * 2.0 + space::LG {
            target / 2.0 + space::XS
        } else {
            0.0
        };
        let button = Rect::from_center_size(rect.center() + vec2(0.0, offset), vec2(width, target));
        control::retry(
            ui,
            root.with("retry"),
            button,
            &format!("Retry image {}", config.alt),
            p,
        )
    })
}
