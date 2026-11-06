use crate::tokens::{loading, motion};
use egui::Ui;

pub(super) fn phase(ui: &Ui, period: f64) -> f32 {
    ui.ctx().request_repaint_after(motion::FRAME_INTERVAL);
    ui.input(|input| (input.time.rem_euclid(period) / period) as f32)
}

pub(super) fn wave(phase: f32) -> f32 {
    (1.0 - (phase * std::f32::consts::TAU).cos()) / 2.0
}

pub(super) fn opacity(ui: &Ui, animate: bool) -> f32 {
    if animate {
        1.0 - wave(phase(ui, motion::SKELETON_PULSE)) * (1.0 - loading::SKELETON_MIN_OPACITY)
    } else {
        1.0
    }
}

pub(super) fn dimension(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
