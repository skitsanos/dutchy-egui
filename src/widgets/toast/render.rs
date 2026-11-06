use super::{
    ToastDismissal, ToastResponse, Toasts, ToastsResponse, memory::Memory, paint, queue::Entry,
};
use crate::{
    Theme,
    tokens::{motion, toast as geometry},
};
use egui::{Context, Id, LayerId, Order, Response, Ui};
use std::time::Duration;
pub(super) fn show<T>(stack: Toasts<'_, T>, ctx: &Context) -> ToastsResponse<T> {
    let root = Id::unique(("dutchy-toasts", stack.id));
    // Below native modal/popup layers, above ordinary host panels/windows.
    let layer = LayerId::new(Order::Middle, root);
    let now = ctx.input(|input| input.time);
    let delta = stack.queue.delta(now);
    let mut memory = Memory::load(ctx, root);
    let suspended =
        ctx.memory(|m| m.top_modal_layer().is_some()) || !ctx.input(|input| input.focused);
    if suspended || stack.queue.is_empty() {
        stack.queue.pause();
        if !suspended {
            memory.finish(ctx, root, Vec::new(), stack.return_focus);
        }
        return ToastsResponse {
            response: None,
            visible: Vec::new(),
            pending: stack.queue.len(),
            events: Vec::new(),
        };
    }
    memory.capture(ctx, layer);
    let bounds = ctx.content_rect().shrink(geometry::GUTTER);
    let width = geometry::WIDTH.min(bounds.width().max(0.0));
    let anchor = stack.position.anchor();
    let mut requests = Vec::new();
    let area = egui::Area::new(root)
        .order(Order::Middle)
        .sense(egui::Sense::hover())
        .anchor(anchor, egui::Vec2::ZERO)
        .constrain_to(bounds)
        .default_width(width)
        .fade_in(false)
        .accessible_name(stack.name)
        .role(egui::accesskit::Role::Group)
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.set_max_height(bounds.height().max(0.0));
            egui::ScrollArea::vertical()
                .id_salt("notifications-overflow")
                .auto_shrink([false, true])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let mut rows = Vec::new();
                    for (index, entry) in stack.queue.entries.iter_mut().enumerate() {
                        if index >= stack.max_visible {
                            entry.running = false;
                            continue;
                        }
                        if index > 0 {
                            ui.add_space(geometry::GAP);
                        }
                        let opacity = entrance(ui, entry, now);
                        let response = paint::row(ui, root, entry.id, &entry.toast, opacity);
                        let running = !stack.paused
                            && !ui.is_sizing_pass()
                            && ui.clip_rect().contains_rect(response.response.rect)
                            && !response.response.contains_pointer()
                            && !response.controls().any(|control| {
                                control.has_focus() || control.is_pointer_button_down_on()
                            });
                        let expired = entry.tick(delta, running);
                        if response.action.as_ref().is_some_and(Response::clicked) {
                            requests.push((entry.id, None));
                        } else if response.close.as_ref().is_some_and(Response::clicked) {
                            requests.push((entry.id, Some(ToastDismissal::CloseButton)));
                        } else if expired {
                            requests.push((entry.id, Some(ToastDismissal::Expired)));
                        }
                        schedule(ctx, entry, running);
                        rows.push(response);
                    }
                    reveal_focus(ui, &rows);
                    rows
                })
                .inner
        });
    ctx.move_to_top(layer);
    let events = stack.queue.finish(requests);
    if !events.is_empty() {
        ctx.request_repaint();
    }
    memory.register(area.inner.iter().flat_map(|row| row.controls().cloned()));
    let remaining = area
        .inner
        .iter()
        .filter(|row| stack.queue.entries.iter().any(|entry| entry.id == row.id))
        .flat_map(|row| row.controls().map(|r| r.id))
        .collect();
    memory.finish(ctx, root, remaining, stack.return_focus);
    let pending = stack.queue.len().saturating_sub(
        area.inner
            .iter()
            .filter(|row| stack.queue.entries.iter().any(|entry| entry.id == row.id))
            .count(),
    );
    ToastsResponse {
        response: Some(area.response),
        visible: area.inner,
        pending,
        events,
    }
}
fn entrance<T>(ui: &Ui, entry: &mut Entry<T>, now: f64) -> f32 {
    if ui.is_sizing_pass() {
        return 1.0;
    }
    let start = *entry.entrance.get_or_insert(now);
    if Theme::from_context(ui.ctx()).reduced_motion {
        return 1.0;
    }
    let amount = ((now - start) / f64::from(motion::SLOW)).clamp(0.0, 1.0) as f32;
    if amount < 1.0 {
        ui.ctx().request_repaint_after(motion::FRAME_INTERVAL);
    }
    amount
}
fn schedule<T>(ctx: &Context, entry: &Entry<T>, running: bool) {
    if running && let Some(remaining) = entry.remaining {
        ctx.request_repaint_after(Duration::try_from_secs_f64(remaining).unwrap_or(Duration::MAX));
    }
}
fn reveal_focus(ui: &Ui, rows: &[ToastResponse]) {
    if let Some(response) = rows
        .iter()
        .flat_map(|row| row.controls())
        .find(|r| r.has_focus() && !ui.clip_rect().contains_rect(r.rect))
    {
        response.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
    }
}
