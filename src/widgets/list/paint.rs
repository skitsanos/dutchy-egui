use super::{ListRow, ListSelectionMode, accessibility};
use crate::{
    Theme,
    interaction::paint_focus_inside,
    tokens::{border, list as geometry},
};
use egui::{Id, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

pub(super) struct Style {
    pub mode: ListSelectionMode,
    pub read_only: bool,
}

pub(super) fn interact(ui: &mut Ui, id: Id, rect: Rect, tab_stop: bool, enabled: bool) -> Response {
    let sense = if tab_stop {
        Sense::click()
    } else {
        Sense::CLICK
    };
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(id.with("surface"))
            .max_rect(rect),
    );
    if !enabled {
        child.disable();
    }
    child.interact(rect, id, sense)
}

pub(super) fn content(
    ui: &mut Ui,
    response: &Response,
    row: ListRow,
    content: impl FnOnce(&mut Ui),
) {
    let rect = response.rect;
    let p = Theme::from_context(ui.ctx()).palette;
    let fill = if row.selected {
        p.background
            .lerp_to_gamma(p.primary, geometry::SELECTED_OPACITY)
    } else if response.hovered() {
        p.background.lerp_to_gamma(p.muted, geometry::HOVER_OPACITY)
    } else {
        p.background
    };
    ui.painter().rect_filled(rect, 0, fill);
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - border::THIN / 2.0,
        Stroke::new(border::THIN, p.border),
    );
    if row.selected {
        let edge = Rect::from_min_size(rect.min, vec2(border::ACCENT, rect.height()));
        ui.painter().rect_filled(edge, 0, p.primary);
    }
    let inner = Rect::from_min_max(
        pos2(
            (rect.left() + geometry::PADDING).min(rect.right()),
            rect.top(),
        ),
        pos2(
            (rect.right() - geometry::PADDING)
                .max(rect.left() + geometry::PADDING)
                .min(rect.right()),
            rect.bottom(),
        ),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(inner));
            content(ui);
        },
    );
    paint_focus_inside(ui, response, p.primary);
}

pub(super) fn metadata(ui: &Ui, response: &Response, label: &str, row: ListRow, style: &Style) {
    accessibility::row(
        response,
        label,
        row.index,
        style.mode,
        row.selected,
        style.read_only,
    );
    if response.has_focus() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    vertical_arrows: true,
                    ..Default::default()
                },
            )
        });
    }
    if response.gained_focus() {
        response.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
    }
}
