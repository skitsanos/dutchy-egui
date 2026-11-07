//! Shared focus-visible policy and sharp focus geometry.

use crate::tokens::border;
use egui::{Color32, Id, Pos2, Response, Stroke, StrokeKind, Ui, Vec2};

/// Reveal a focused control once: when focus arrives, or when layout moved it
/// relative to `anchor` (its container's content origin). User scrolling moves
/// the anchor by the same amount, so it never pulls the scroll area back.
pub(crate) fn reveal_focused(ui: &Ui, response: &Response, anchor: Pos2) {
    if !response.has_focus() {
        return;
    }
    let key = Id::unique("dutchy-reveal-anchor");
    let offset = response.rect.min - anchor;
    let moved = ui
        .ctx()
        .data(|data| data.get_temp::<(Id, Vec2)>(key))
        .is_some_and(|(id, previous)| id == response.id && (previous - offset).length() > 0.5);
    ui.ctx()
        .data_mut(|data| data.insert_temp(key, (response.id, offset)));
    if (response.gained_focus() || moved) && !ui.clip_rect().contains_rect(response.rect) {
        response.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
    }
}

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
/// but hide the extra ring; keys, text, IME, clipboard and assistive events
/// restore it. Hover does not change the input mode. The mode persists in
/// temporary memory, so programmatic focus shows a cue only until the first
/// pointer press and again after the next keyboard or assistive event.
pub fn focus_is_visible(response: &Response) -> bool {
    let ctx = &response.ctx;
    let mode_id = egui::Id::unique("dutchy-focus-visible");
    let keyboard = ctx.input(|input| {
        input.raw.events.iter().rev().find_map(|event| match event {
            egui::Event::PointerButton { pressed: true, .. } => Some(false),
            egui::Event::Key { pressed: true, .. }
            | egui::Event::Text(_)
            | egui::Event::Ime(_)
            | egui::Event::Paste(_)
            | egui::Event::Copy
            | egui::Event::Cut
            | egui::Event::AccessKitActionRequest(_) => Some(true),
            _ => None,
        })
    });
    if let Some(keyboard) = keyboard {
        ctx.data_mut(|data| data.insert_temp(mode_id, keyboard));
    }
    let keyboard = ctx.data(|data| data.get_temp::<bool>(mode_id).unwrap_or(true));
    response.enabled() && response.has_focus() && keyboard
}
