//! Shared sharp keyboard-focus outline.

use crate::tokens::border;
use egui::{Color32, Response, Stroke, StrokeKind, Ui};

/// Paint an offset focus outline around a focused, enabled control.
pub fn paint_focus(ui: &Ui, response: &Response, color: Color32) {
    if response.enabled() && response.has_focus() && ui.is_rect_visible(response.rect) {
        ui.painter().rect_stroke(
            response
                .rect
                .expand(border::FOCUS_OFFSET + border::FOCUS_WIDTH),
            0,
            Stroke::new(border::FOCUS_WIDTH, color),
            StrokeKind::Inside,
        );
    }
}
