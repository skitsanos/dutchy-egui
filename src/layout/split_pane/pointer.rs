use super::geometry::{Axis, Bounds};
use egui::{Event, Id, PointerButton, Pos2, Rect, Response, Ui};

pub(super) fn cancel(ui: &Ui, id: Id) {
    ui.ctx().data_mut(|d| d.remove::<Grab>(id.with("grab")));
}
#[derive(Clone, Copy)]
struct Grab {
    frame: u64,
    offset: f32,
    axis: Axis,
}
pub(super) fn read(
    ui: &Ui,
    response: &Response,
    rect: Rect,
    axis: Axis,
    bounds: Bounds,
    value: f32,
) -> f32 {
    if let Some(value) = batched(ui, response, axis, bounds, value) {
        return value;
    }
    let key = response.id.with("grab");
    let frame = ui.ctx().cumulative_frame_nr();
    let mut grab = ui.ctx().data(|d| d.get_temp::<Grab>(key));
    if grab.is_some_and(|g| g.axis != axis) {
        ui.ctx().data_mut(|d| d.remove::<Grab>(key));
        grab = None;
    }
    if response.drag_started_by(egui::PointerButton::Primary)
        && grab.is_none_or(|g| g.frame != frame)
        && let Some(pos) = response.interact_pointer_pos()
    {
        let press = pos - response.total_drag_delta().unwrap_or_default();
        let next = Grab {
            frame,
            axis,
            offset: axis.coordinate(press) - axis.coordinate(rect.min) - value,
        };
        ui.ctx().data_mut(|d| d.insert_temp(key, next));
        grab = Some(next);
    }
    let mut next = value;
    if (response.dragged_by(egui::PointerButton::Primary)
        || response.drag_stopped_by(egui::PointerButton::Primary))
        && let (Some(grab), Some(pos)) = (grab, response.interact_pointer_pos())
    {
        next = bounds.clamp(axis.coordinate(pos) - axis.coordinate(rect.min) - grab.offset);
    }
    if !response.dragged() {
        ui.ctx().data_mut(|d| d.remove::<Grab>(key));
    }
    next
}

#[derive(Clone, Copy)]
struct FrameValue {
    frame: u64,
    value: f32,
}

// egui does not classify a press and release in the same frame as a drag.
// Honor a complete primary gesture only in our reserved divider, on its layer.
fn batched(ui: &Ui, response: &Response, axis: Axis, bounds: Bounds, value: f32) -> Option<f32> {
    let key = response.id.with("batched-drag");
    let frame = ui.ctx().cumulative_frame_nr();
    if let Some(last) = ui.ctx().data(|d| d.get_temp::<FrameValue>(key))
        && last.frame == frame
    {
        return Some(bounds.clamp(last.value));
    }
    if response.dragged() || response.drag_started() || response.drag_stopped() {
        return None;
    }
    let (press, release) = primary_pair(ui)?;
    if ui.ctx().layer_id_at(press) != Some(response.layer_id) {
        return None;
    }
    let local = |pos| {
        ui.ctx()
            .layer_transform_from_global(response.layer_id)
            .map_or(pos, |t| t * pos)
    };
    let press = local(press);
    if !response
        .interact_rect
        .intersect(ui.clip_rect())
        .contains(press)
    {
        return None;
    }
    let delta = axis.coordinate(local(release)) - axis.coordinate(press);
    if delta == 0.0 {
        return None;
    }
    let next = bounds.clamp(value + delta);
    ui.ctx()
        .data_mut(|d| d.insert_temp(key, FrameValue { frame, value: next }));
    response.request_focus();
    Some(next)
}
fn primary_pair(ui: &Ui) -> Option<(Pos2, Pos2)> {
    ui.input(|input| {
        let mut start = None;
        for event in &input.raw.events {
            if let Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                ..
            } = event
            {
                if *pressed {
                    start = Some(*pos);
                } else if let Some(start) = start.take() {
                    return Some((start, *pos));
                }
            }
        }
        None
    })
}
