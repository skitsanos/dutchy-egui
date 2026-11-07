use super::{
    GalleryItem, ImageFit, ImageResponse,
    control::{self, Icon},
};
use crate::tokens::Palette;
use crate::{
    Family, Mode, OverlayOptions, Text, TextSize, Theme,
    tokens::{image as geometry, space},
};
use egui::{Id, Rect, Response, Sense, Ui, vec2};

pub(super) struct Contents {
    pub close: Option<Response>,
    pub previous: Option<Response>,
    pub next: Option<Response>,
    pub image: Option<ImageResponse>,
    pub focus: Id,
    pub sizing: bool,
}
pub(super) fn palette(theme: Theme) -> Palette {
    let mut p = Palette::new(crate::Accent::Orange, Mode::Dark);
    p.background = geometry::VIEWER_BACKGROUND;
    p.foreground = geometry::VIEWER_FOREGROUND;
    p.primary = theme.palette.primary;
    p.on_primary = theme.palette.on_primary;
    p
}
pub(super) struct View<'a, 'b, K> {
    pub name: &'a str,
    pub items: &'a [GalleryItem<'b, K>],
    pub index: Option<usize>,
    pub previous: Option<usize>,
    pub next: Option<usize>,
    pub size: egui::Vec2,
    pub options: &'a OverlayOptions,
    pub palette: Palette,
}
pub(super) fn show<K>(ui: &mut Ui, root: Id, view: View<'_, '_, K>) -> Contents {
    let View {
        name,
        items,
        index,
        previous,
        next,
        size,
        options,
        palette: p,
    } = view;
    ui.set_width(size.x);
    ui.set_height(size.y);
    ui.spacing_mut().item_spacing = egui::Vec2::splat(space::SM);
    ui.visuals_mut().override_text_color = Some(p.foreground);
    let start = ui.cursor().top();
    let (close, previous, next, focus) = header(
        ui,
        Header {
            root,
            name,
            index,
            count: items.len(),
            previous: previous.is_some(),
            next: next.is_some(),
            options,
            p,
        },
    );
    let remaining = (size.y - (ui.cursor().top() - start)).max(0.0);
    let stage = (remaining - geometry::CAPTION_RESERVE).max(geometry::MIN_STAGE.min(remaining));
    let image = egui::ScrollArea::vertical()
        .id_salt("viewer-content")
        .max_height(remaining)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            page_scroll(ui, remaining);
            if let Some(index) = index {
                let mut image = items[index]
                    .image()
                    .width(ui.available_width())
                    .height(stage)
                    .fit(ImageFit::Contain)
                    .bordered(false)
                    .retryable(true);
                image.surface = Some(p);
                Some(image.show(ui))
            } else {
                ui.add(
                    Text::heading("Image unavailable")
                        .size(TextSize::Lg)
                        .color(p.foreground),
                );
                ui.add(
                    Text::new("The selected image is no longer available in this collection.")
                        .size(TextSize::Sm)
                        .color(p.muted_foreground),
                );
                None
            }
        })
        .inner;
    Contents {
        close,
        previous,
        next,
        image,
        focus,
        sizing: ui.is_sizing_pass(),
    }
}
fn page_scroll(ui: &Ui, height: f32) {
    if ui.memory(|memory| memory.top_modal_layer()) != Some(ui.layer_id()) {
        return;
    }
    let direction = ui.input_mut(|input| {
        if input.consume_key(egui::Modifiers::NONE, egui::Key::PageDown) {
            -1.0
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::PageUp) {
            1.0
        } else {
            0.0
        }
    });
    if direction != 0.0 {
        ui.scroll_with_delta_animation(
            vec2(0.0, direction * height),
            egui::style::ScrollAnimation::none(),
        );
    }
}
struct Header<'a> {
    root: Id,
    name: &'a str,
    index: Option<usize>,
    count: usize,
    previous: bool,
    next: bool,
    options: &'a OverlayOptions,
    p: Palette,
}
fn header(
    ui: &mut Ui,
    config: Header<'_>,
) -> (Option<Response>, Option<Response>, Option<Response>, Id) {
    let Header {
        root,
        name,
        index,
        count,
        previous,
        next,
        options,
        p,
    } = config;
    let origin = ui.cursor().min;
    let extent = geometry::VIEWER_CONTROL.min(ui.available_width());
    let reserve = if options.show_close_button {
        extent + space::SM.min((ui.available_width() - extent).max(0.0))
    } else {
        0.0
    };
    let width = (ui.available_width() - reserve).max(0.0);
    let controls = ui.allocate_ui_with_layout(
        vec2(width, extent),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(width);
            let job = Text::heading(name)
                .size(TextSize::Sm)
                .color(p.foreground)
                .layout_job(&Theme::from_context(ui.ctx()), width);
            let heading = ui.add(egui::Label::new(job).sense(if options.show_close_button {
                Sense::hover()
            } else {
                Sense::focusable_noninteractive()
            }));
            heading.ctx.accesskit_node_builder(heading.id, |node| {
                node.set_role(egui::accesskit::Role::Heading);
                node.set_label(name.to_string());
                node.set_level(2);
            });
            if !options.show_close_button {
                crate::interaction::paint_focus_inside(ui, &heading, p.primary);
            }
            let navigation = ui
                .horizontal_wrapped(|ui| {
                    let mut buttons = Vec::new();
                    if count > 1 {
                        for (salt, label, icon, enabled) in [
                            ("previous", "Previous image", Icon::Previous, previous),
                            ("next", "Next image", Icon::Next, next),
                        ] {
                            let (_, rect) =
                                ui.allocate_space(egui::Vec2::splat(geometry::VIEWER_CONTROL));
                            buttons.push(ui.add_enabled(enabled, |ui: &mut Ui| {
                                control::icon(
                                    ui,
                                    root.with(salt),
                                    rect,
                                    label,
                                    icon,
                                    geometry::EXPAND_OPACITY,
                                    p,
                                )
                            }));
                        }
                    }
                    let counter = ui.add(
                        Text::new(index.map_or_else(
                            || "Image unavailable".into(),
                            |index| format!("{} / {}", index + 1, count),
                        ))
                        .family(Family::Mono)
                        .size(TextSize::Xs)
                        .color(p.foreground),
                    );
                    counter.ctx.accesskit_node_builder(counter.id, |node| {
                        node.set_role(egui::accesskit::Role::Status);
                        node.set_label(index.map_or_else(
                            || "Image unavailable".into(),
                            |index| format!("Image {} of {}", index + 1, count),
                        ));
                        node.set_live(egui::accesskit::Live::Polite);
                    });
                    buttons
                })
                .inner;
            (heading.id, navigation)
        },
    );
    let close = options.show_close_button.then(|| {
        let rect = Rect::from_min_size(
            egui::pos2(origin.x + ui.available_width() - extent, origin.y),
            egui::Vec2::splat(extent),
        );
        control::icon(
            ui,
            root.with("close"),
            rect,
            &format!("Close {name}"),
            Icon::Close,
            geometry::EXPAND_OPACITY,
            p,
        )
    });
    let focus = close
        .as_ref()
        .map_or(controls.inner.0, |response| response.id);
    let mut navigation = controls.inner.1.into_iter();
    (close, navigation.next(), navigation.next(), focus)
}
