//! Shared focus-visible policy and sharp focus geometry.

use crate::tokens::border;
use egui::{Color32, Response, Stroke, StrokeKind, Ui};

/// Paint an offset focus outline around a focused, enabled control.
pub fn paint_focus(ui: &Ui, response: &Response, color: Color32) {
    if focus_is_visible(response) && ui.is_rect_visible(response.rect) {
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

/// Paint focus inside a composite header, clear of neighboring fills and borders.
pub fn paint_focus_inside(ui: &Ui, response: &Response, color: Color32) {
    if focus_is_visible(response) && ui.is_rect_visible(response.rect) {
        ui.painter().rect_stroke(
            response
                .rect
                .shrink(border::FOCUS_OFFSET + border::FOCUS_WIDTH),
            0,
            Stroke::new(border::FOCUS_WIDTH, color),
            StrokeKind::Inside,
        );
    }
}

/// Whether focus should have a visible cue. Pointer presses retain native focus
/// but hide the extra ring; keys and assistive actions restore it. Hover does not
/// change the input mode. Programmatic focus defaults to a visible cue.
pub fn focus_is_visible(response: &Response) -> bool {
    let ctx = &response.ctx;
    let mode_id = egui::Id::unique("dutchy-focus-visible");
    let keyboard = ctx.input(|input| {
        input.raw.events.iter().rev().find_map(|event| match event {
            egui::Event::PointerButton { pressed: true, .. } => Some(false),
            egui::Event::Key { pressed: true, .. } | egui::Event::AccessKitActionRequest(_) => {
                Some(true)
            }
            _ => None,
        })
    });
    if let Some(keyboard) = keyboard {
        ctx.data_mut(|data| data.insert_temp(mode_id, keyboard));
    }
    let keyboard = ctx.data(|data| data.get_temp::<bool>(mode_id).unwrap_or(true));
    response.enabled() && response.has_focus() && keyboard
}
