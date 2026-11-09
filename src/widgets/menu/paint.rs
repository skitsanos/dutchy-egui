use super::{MenuItem, MenuItemVariant};
use crate::{
    Family, Icon, Text, TextSize, Theme, Weight,
    tokens::{border, menu as geometry},
};
use egui::{Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2};

pub(crate) struct RowState {
    pub role: egui::accesskit::Role,
    pub active: bool,
    pub selected: bool,
    pub focusable: bool,
    pub check_slot: bool,
    pub pointer: bool,
    pub branch_open: bool,
}

pub(crate) fn row<T>(ui: &mut Ui, id: Id, item: &MenuItem<'_, T>, state: RowState) -> Response {
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    let width = ui.available_width().max(0.0);
    let check_width = if state.check_slot {
        geometry::ICON_SIZE + geometry::ITEM_GAP
    } else {
        0.0
    };
    let branch_width = if item.children.is_some() {
        geometry::ICON_SIZE + geometry::ITEM_GAP
    } else {
        0.0
    };
    let content = (width - geometry::PADDING_X * 2.0 - check_width - branch_width).max(0.0);
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
    metadata(&response, item, &state);
    let highlighted = ui.is_enabled()
        && (state.active
            || (state.focusable
                && if state.pointer {
                    response.hovered()
                } else {
                    response.has_focus()
                }));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    // Under pointer ownership the keyboard-focused row stays visible as quiet
    // context, so Enter never activates an unmarked row.
    let focus_behind_pointer = state.pointer && !highlighted && response.has_focus();
    let (fill, foreground) = colors(
        p,
        highlighted,
        state.selected || state.branch_open || focus_behind_pointer,
        item.variant,
    );
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    painter.rect_filled(rect, 0, fill);
    let arrow_ink = if state.branch_open && !highlighted {
        p.primary
    } else {
        foreground
    };
    decorations(&painter, rect, item, foreground, arrow_ink);
    if let Some(icon) = item.icon {
        icon(
            &painter,
            Rect::from_center_size(
                pos2(
                    rect.left() + geometry::PADDING_X + check_width + geometry::ICON_SIZE / 2.0,
                    rect.center().y,
                ),
                vec2(geometry::ICON_SIZE, geometry::ICON_SIZE),
            ),
            foreground,
        );
    }
    let text_left = rect.left() + geometry::PADDING_X + check_width + icon_width;
    painter
        .with_clip_rect(Rect::from_min_size(
            pos2(text_left, rect.top()),
            vec2(text_width, rect.height()),
        ))
        .galley_with_override_text_color(
            pos2(text_left, rect.center().y - galley.size().y / 2.0),
            galley,
            foreground,
        );
    if let Some(hint) = hint {
        let position = pos2(
            rect.right() - geometry::PADDING_X - branch_width - hint.size().x,
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

fn colors(
    p: crate::tokens::Palette,
    highlighted: bool,
    selected: bool,
    variant: MenuItemVariant,
) -> (egui::Color32, egui::Color32) {
    let destructive = variant == MenuItemVariant::Destructive;
    let fill = if highlighted {
        if destructive {
            p.destructive
        } else {
            p.primary
        }
    } else if selected {
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
    (fill, foreground)
}

fn metadata<T>(response: &Response, item: &MenuItem<'_, T>, state: &RowState) {
    let role = if state.role == egui::accesskit::Role::MenuItem
        && item.children.is_none()
        && item.checked.is_some()
    {
        egui::accesskit::Role::MenuItemCheckBox
    } else {
        state.role
    };
    response.widget_info(|| WidgetInfo::labeled(role, response.enabled(), item.label.as_ref()));
    response.ctx.accesskit_node_builder(response.id, |node| {
        if state.role == egui::accesskit::Role::ListBoxOption {
            node.set_selected(state.selected);
        }
        if role == egui::accesskit::Role::MenuItemCheckBox {
            node.set_toggled(if item.checked == Some(true) {
                egui::accesskit::Toggled::True
            } else {
                egui::accesskit::Toggled::False
            });
        }
        if let Some(hint) = &item.shortcut {
            node.set_description(format!("Shortcut: {hint}"));
        }
    });
}

fn decorations<T>(
    painter: &egui::Painter,
    rect: Rect,
    item: &MenuItem<'_, T>,
    foreground: egui::Color32,
    arrow_ink: egui::Color32,
) {
    if let Some(checked) = item.checked.filter(|_| item.children.is_none()) {
        let check = Rect::from_center_size(
            pos2(
                rect.left() + geometry::PADDING_X + geometry::ICON_SIZE / 2.0,
                rect.center().y,
            ),
            vec2(geometry::CHECK_SIZE, geometry::CHECK_SIZE),
        );
        painter.rect_stroke(
            check,
            0,
            Stroke::new(border::THIN, foreground),
            StrokeKind::Inside,
        );
        if checked {
            Icon::Check.paint(painter, check.shrink(border::STANDARD), foreground);
        }
    }
    if item.children.is_some() {
        let center = pos2(
            rect.right() - geometry::PADDING_X - geometry::ICON_SIZE / 2.0,
            rect.center().y,
        );
        let offset = geometry::CHECK_SIZE / 4.0;
        painter.line_segment(
            [
                center + vec2(-offset / 2.0, -offset),
                center + vec2(offset / 2.0, 0.0),
            ],
            Stroke::new(border::STANDARD, arrow_ink),
        );
        painter.line_segment(
            [
                center + vec2(offset / 2.0, 0.0),
                center + vec2(-offset / 2.0, offset),
            ],
            Stroke::new(border::STANDARD, arrow_ink),
        );
    }
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
