use super::{Tabs, TabsVariant, focus, paint};
use crate::{
    Theme,
    interaction::paint_focus,
    tokens::{border, disclosure as geometry, motion},
    widgets::navigation::{self, Axis},
};
use egui::{Frame, Id, Key, Margin, Modifiers, Rect, Response, Ui};

pub(super) struct BarResponse {
    pub tabs: Vec<Response>,
    pub close: Option<usize>,
    pub viewport: Rect,
}

pub(super) fn show<T: Clone + PartialEq>(
    ui: &mut Ui,
    root: Id,
    tabs: &mut Tabs<'_, T>,
) -> BarResponse {
    focus::prepare(ui, root, tabs);
    let tab_stop = focus::tab_stop(ui, root, tabs);
    let mut close = keyboard_close(ui, root, tabs);
    let p = Theme::from_context(ui.ctx()).palette;
    let segmented = tabs.variant == TabsVariant::Segmented;
    let line_shape = ui.painter().add(egui::Shape::Noop);
    let frame = if segmented {
        Frame::NONE
            .fill(p.muted)
            .inner_margin(Margin::same(geometry::SEGMENT_INSET))
    } else {
        Frame::NONE
    };
    let result = frame
        .show(ui, |ui| {
            let widths = widths(ui, tabs);
            egui::ScrollArea::horizontal()
                .id_salt("tab-overflow")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    let height = if segmented {
                        geometry::SEGMENT_HEIGHT
                    } else {
                        geometry::TAB_HEIGHT
                    };
                    ui.allocate_ui_with_layout(
                        egui::vec2(widths.iter().sum(), height),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let mut responses = Vec::with_capacity(tabs.items.len());
                            for (index, (item, width)) in tabs.items.iter().zip(widths).enumerate()
                            {
                                let selected = tabs.selected.as_ref() == Some(&item.value);
                                let (response, close_response) = ui
                                    .add_enabled_ui(item.enabled, |ui| {
                                        paint::tab(
                                            ui,
                                            root,
                                            item,
                                            selected,
                                            tab_stop == Some(index),
                                            width,
                                            tabs.variant,
                                        )
                                    })
                                    .inner;
                                if response.clicked() {
                                    *tabs.selected = Some(item.value.clone());
                                    response.request_focus();
                                    ui.ctx().request_repaint();
                                }
                                if close_response.as_ref().is_some_and(Response::clicked) {
                                    close = Some(index);
                                    if let Some(response) = &close_response {
                                        response.request_focus();
                                    }
                                }
                                if response.has_focus() {
                                    navigation::lock_arrows(ui, response.id, Axis::Horizontal);
                                }
                                if response.gained_focus() || response.clicked() {
                                    let theme = Theme::from_context(ui.ctx());
                                    let animation = if theme.reduced_motion {
                                        egui::style::ScrollAnimation::none()
                                    } else {
                                        egui::style::ScrollAnimation::duration(motion::FAST)
                                    };
                                    response.scroll_to_me_animation(
                                        Some(egui::Align::Center),
                                        animation,
                                    );
                                }
                                responses.push(response);
                            }
                            responses
                        },
                    )
                    .inner
                })
        })
        .inner;
    let mut viewport = result.inner_rect;
    // The pre-layout inner rect can have zero height below a vertical scroll
    // area's initial bounds. A horizontal area grows vertically to its content.
    viewport.max.y = viewport.min.y + result.content_size.y;
    if !segmented && !tabs.items.is_empty() {
        let mut line = viewport;
        line.min.y = line.max.y - border::STANDARD;
        ui.painter()
            .set(line_shape, egui::Shape::rect_filled(line, 0, p.border));
    }
    let clip = ui.clip_rect();
    ui.set_clip_rect(clip.intersect(viewport.expand(border::FOCUS_WIDTH + border::FOCUS_OFFSET)));
    for (item, response) in tabs.items.iter().zip(&result.inner) {
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_selected(tabs.selected.as_ref() == Some(&item.value))
        });
        if tabs.selected.as_ref() != Some(&item.value) {
            paint_focus(ui, response, p.primary);
        }
    }
    ui.set_clip_rect(clip);
    BarResponse {
        tabs: result.inner,
        close,
        viewport,
    }
}

fn widths<T>(ui: &Ui, tabs: &Tabs<'_, T>) -> Vec<f32> {
    let mut widths: Vec<_> = tabs
        .items
        .iter()
        .map(|item| paint::width(ui, item))
        .collect();
    if tabs
        .full_width
        .unwrap_or(tabs.variant == TabsVariant::Segmented)
        && !widths.is_empty()
    {
        let width = widths
            .iter()
            .copied()
            .fold(0.0, f32::max)
            .max(ui.available_width() / widths.len() as f32);
        widths.fill(width);
    }
    widths
}

fn keyboard_close<T>(ui: &Ui, root: Id, tabs: &Tabs<'_, T>) -> Option<usize> {
    if !ui.is_enabled() {
        return None;
    }
    let focused = ui.memory(|memory| memory.focused());
    let current = tabs.items.iter().position(|item| {
        item.enabled
            && item.closable
            && (focused == Some(focus::trigger(root, item.id))
                || focused == Some(focus::close(root, item.id)))
    })?;
    ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Delete))
        .then_some(current)
}
