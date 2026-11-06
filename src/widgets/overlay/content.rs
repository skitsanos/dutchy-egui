use super::render::Config;
use crate::{
    Text, TextSize, Theme, Tracking,
    tokens::{ControlSize, border, overlay as geometry, space},
    widgets::close,
};
use egui::{Id, Response, Sense, Ui, WidgetInfo, vec2};
pub(super) struct Contents<R> {
    pub inner: R,
    pub close: bool,
    pub focus: Id,
    pub sizing: bool,
}
pub(super) fn show<R, F>(
    ui: &mut Ui,
    root: Id,
    config: &Config<'_>,
    width: f32,
    height: f32,
    body: impl FnOnce(&mut Ui) -> R,
    footer: Option<impl FnOnce(&mut Ui) -> F>,
) -> Contents<(R, Option<F>)> {
    ui.set_width(width);
    ui.set_max_height(height);
    if config.placement.is_drawer() {
        ui.set_min_height(height);
    }
    ui.spacing_mut().item_spacing.y = 0.0;
    let start = ui.cursor().top();
    let (close, focus) = header(ui, root, config);
    divider(ui);
    let remaining = (height - (ui.cursor().top() - start)).max(0.0);
    let scrolling = egui::ScrollArea::vertical()
        .id_salt("overlay-content")
        .max_height(remaining)
        .min_scrolled_height(0.0)
        .auto_shrink([false, !config.placement.is_drawer()])
        .show(ui, |ui| {
            let body = egui::Frame::NONE
                .inner_margin(egui::Margin::same(geometry::BODY_PADDING))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = space::SM;
                    body(ui)
                })
                .inner;
            let footer = footer.map(|footer| {
                divider(ui);
                egui::Frame::NONE
                    .inner_margin(margin())
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = space::SM;
                        footer(ui)
                    })
                    .inner
            });
            reveal_focused_content(ui);
            (body, footer)
        });
    Contents {
        inner: scrolling.inner,
        close,
        focus,
        sizing: ui.is_sizing_pass(),
    }
}
fn margin() -> egui::Margin {
    egui::Margin {
        left: geometry::PADDING_X,
        right: geometry::PADDING_X,
        top: geometry::PADDING_Y,
        bottom: geometry::PADDING_Y,
    }
}
fn header(ui: &mut Ui, root: Id, config: &Config<'_>) -> (bool, Id) {
    egui::Frame::NONE
        .inner_margin(margin())
        .show(ui, |ui| {
            let size = ControlSize::Small.height();
            let reserved = if config.options.show_close_button {
                size + geometry::TITLE_GAP
            } else {
                0.0
            };
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = geometry::TITLE_GAP;
                let width = (ui.available_width() - reserved).max(0.0);
                let heading = ui
                    .allocate_ui_with_layout(
                        vec2(width, size),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_width(width);
                            let job = Text::heading(config.title.as_ref())
                                .size(TextSize::Lg)
                                .tracking(Tracking::Tight)
                                .layout_job(&Theme::from_context(ui.ctx()), width);
                            let sense = if config.options.show_close_button {
                                Sense::hover()
                            } else {
                                Sense::focusable_noninteractive()
                            };
                            let response = ui.add(egui::Label::new(job).sense(sense));
                            ui.ctx().accesskit_node_builder(response.id, |node| {
                                node.set_role(egui::accesskit::Role::Heading);
                                node.set_level(2);
                                node.set_label(config.title.to_string());
                            });
                            response
                        },
                    )
                    .inner;
                if config.options.show_close_button {
                    let (_, rect) = ui.allocate_space(vec2(size, size));
                    let response: Response = ui.interact(rect, root.with("close"), Sense::click());
                    response.widget_info(|| {
                        WidgetInfo::labeled(
                            egui::accesskit::Role::Button,
                            ui.is_enabled(),
                            format!("Close {}", config.title),
                        )
                    });
                    let p = Theme::from_context(ui.ctx()).palette;
                    close::paint(
                        ui,
                        &response,
                        geometry::CLOSE_ICON,
                        close::Colors {
                            idle: p.muted_foreground,
                            foreground: p.foreground,
                            focus: p.primary,
                        },
                    );
                    (response.clicked(), response.id)
                } else {
                    (false, heading.id)
                }
            })
            .inner
        })
        .inner
}
fn divider(ui: &mut Ui) {
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), border::STANDARD));
    ui.painter()
        .rect_filled(rect, 0, Theme::from_context(ui.ctx()).palette.border);
}

fn reveal_focused_content(ui: &Ui) {
    if let Some(response) = ui
        .memory(|memory| memory.focused())
        .and_then(|id| ui.ctx().read_response(id))
        && response.layer_id == ui.layer_id()
        && response.gained_focus()
        && ui.min_rect().contains_rect(response.rect)
    {
        response.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
    }
}
