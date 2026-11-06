use super::{MenuItem, MenuItemVariant};
use crate::{
    Family, Text, TextSize, Theme, Weight,
    interaction::focus_is_visible,
    tokens::{border, menu as geometry},
};
use egui::{Id, Rect, Response, Sense, Stroke, Ui, WidgetInfo, pos2, vec2};

pub(crate) struct RowState {
    pub role: egui::accesskit::Role,
    pub active: bool,
    pub selected: bool,
    pub focusable: bool,
}

pub(crate) fn row<T>(ui: &mut Ui, id: Id, item: &MenuItem<'_, T>, state: RowState) -> Response {
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    let width = ui.available_width().max(0.0);
    let content = (width - geometry::PADDING_X * 2.0).max(0.0);
    let icon_width = if item.icon.is_some() {
        geometry::ICON_SIZE + geometry::ITEM_GAP
    } else {
        0.0
    };
    let hint = item.shortcut.as_deref().map(|hint| {
        let mut job = Text::new(hint)
            .family(Family::Mono)
            .size(TextSize::Xs)
            .layout_job(&theme, content / 2.0);
        job.wrap.max_rows = 1;
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    });
    let hint_width = hint
        .as_ref()
        .map_or(0.0, |galley| galley.size().x + geometry::ITEM_GAP);
    let text_width = (content - icon_width - hint_width).max(0.0);
    let job = Text::new(item.label.as_ref())
        .size(TextSize::Sm)
        .weight(if state.selected {
            Weight::Bold
        } else {
            Weight::Regular
        })
        .layout_job(&theme, text_width);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let height = geometry::ROW_HEIGHT.max(galley.size().y + geometry::PADDING_Y * 2.0);
    let (_, rect) = ui.allocate_space(vec2(width, height));
    let sense = if state.focusable {
        Sense::click()
    } else {
        Sense::CLICK
    };
    let response = ui.interact(rect, id, sense);
    response.widget_info(|| WidgetInfo::labeled(state.role, ui.is_enabled(), item.label.as_ref()));
    ui.ctx().accesskit_node_builder(id, |node| {
        if state.role == egui::accesskit::Role::ListBoxOption {
            node.set_selected(state.selected);
        }
        if let Some(hint) = &item.shortcut {
            node.set_description(format!("Shortcut: {hint}"));
        }
    });
    let highlighted = ui.is_enabled()
        && (state.active
            || (state.focusable && (response.hovered() || focus_is_visible(&response))));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let destructive = item.variant == MenuItemVariant::Destructive;
    let fill = if highlighted {
        if destructive {
            p.destructive
        } else {
            p.primary
        }
    } else if state.selected {
        p.muted
    } else {
        p.background
    };
    let foreground = if highlighted {
        if destructive {
            p.on_status
        } else {
            p.on_primary
        }
    } else if destructive {
        p.destructive
    } else {
        p.foreground
    };
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    painter.rect_filled(rect, 0, fill);
    if let Some(icon) = item.icon {
        icon(
            &painter,
            Rect::from_center_size(
                pos2(
                    rect.left() + geometry::PADDING_X + geometry::ICON_SIZE / 2.0,
                    rect.center().y,
                ),
                vec2(geometry::ICON_SIZE, geometry::ICON_SIZE),
            ),
            foreground,
        );
    }
    painter.galley_with_override_text_color(
        pos2(
            rect.left() + geometry::PADDING_X + icon_width,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        foreground,
    );
    if let Some(hint) = hint {
        let position = pos2(
            rect.right() - geometry::PADDING_X - hint.size().x,
            rect.center().y - hint.size().y / 2.0,
        );
        painter.galley_with_override_text_color(
            position,
            hint,
            if highlighted {
                foreground
            } else {
                p.muted_foreground
            },
        );
    }
    response
}

pub(crate) fn divider(ui: &mut Ui, strong: bool) {
    let height = if strong {
        border::STANDARD
    } else {
        border::THIN
    };
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), height));
    ui.painter()
        .rect_filled(rect, 0, Theme::from_context(ui.ctx()).palette.border);
}

pub(crate) fn frame(theme: Theme) -> egui::Frame {
    egui::Frame::NONE
        .fill(theme.palette.background)
        .stroke(Stroke::new(border::STANDARD, theme.palette.border))
}
