use super::geometry::Slots;
use crate::{
    ButtonVariant, ControlSize, Icon, IconButton, Input, Text, TextColor, Theme,
    interaction::paint_focus_inside,
    tokens::{border, inline_edit as tokens},
};
use egui::{
    Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo, accesskit::Role, vec2,
};

#[derive(Clone, Copy)]
pub(super) struct Options<'a> {
    pub size: ControlSize,
    pub name: &'a str,
    pub read_only: bool,
    pub limit: Option<usize>,
}

pub(super) fn idle(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    value: &str,
    placeholder: &str,
    options: Options<'_>,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(
            Role::Button,
            ui.is_enabled(),
            format!("Rename {}", options.name),
        )
    });
    ui.ctx().accesskit_node_builder(id, |node| {
        // An empty name is announced as the placeholder it shows.
        node.set_value(if value.is_empty() { placeholder } else { value });
        node.set_description(if options.read_only {
            "Read-only name."
        } else {
            "Enter, Space, F2 or click to rename. Enter saves; Escape cancels."
        });
        if options.read_only {
            node.set_read_only();
        }
    });
    let p = Theme::from_context(ui.ctx()).palette;
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let hot = response.hovered() && !options.read_only && ui.is_enabled();
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    if hot {
        painter.rect(
            rect,
            0,
            p.muted,
            Stroke::new(border::STANDARD, p.border),
            StrokeKind::Inside,
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let padding = tokens::PADDING.min(rect.width() / 4.0);
    let icon = !options.read_only && rect.width() >= 2.0 * padding + 2.0 * tokens::ICON;
    let text_width =
        (rect.width() - 2.0 * padding - if icon { tokens::ICON + padding } else { 0.0 }).max(0.0);
    let color = if !ui.is_enabled() || options.read_only || value.is_empty() {
        p.muted_foreground
    } else {
        ui.visuals().override_text_color.unwrap_or(p.foreground)
    };
    let mut job = Text::new(if value.is_empty() { placeholder } else { value })
        .size(options.size.text_size())
        .color(TextColor::Custom(color))
        .layout_job(&Theme::from_context(ui.ctx()), text_width);
    job.wrap.max_rows = 1;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    painter.galley(
        egui::pos2(
            rect.left() + padding,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        color,
    );
    if icon {
        let r = Rect::from_center_size(
            egui::pos2(rect.right() - padding - tokens::ICON / 2.0, rect.center().y),
            vec2(tokens::ICON, tokens::ICON),
        );
        pencil(
            &painter,
            r,
            if hot { p.primary } else { p.muted_foreground },
        );
    }
    paint_focus_inside(ui, &response, p.primary);
    response
}

pub(super) fn editor(
    ui: &mut Ui,
    slot: Rect,
    draft: &mut String,
    options: Options<'_>,
) -> Response {
    super::super::style::paint_frame(ui, slot, Theme::from_context(ui.ctx()));
    let padding = tokens::PADDING.min(slot.width() / 4.0);
    let inner = slot.shrink2(vec2(padding, border::STANDARD.min(slot.height() / 2.0)));
    let mut child = ui.new_child(
        UiBuilder::new()
            .scope_id(ui.scope_id().with("editor-slot"))
            .max_rect(inner),
    );
    child.set_clip_rect(ui.clip_rect().intersect(inner));
    let mut response = child.add(
        Input::new("editor", draft)
            .accessible_name(options.name)
            .size(options.size)
            .read_only(options.read_only)
            .char_limit(options.limit.unwrap_or(usize::MAX))
            .width(inner.width())
            .without_frame(inner.height())
            .without_return_key(),
    );
    // Give the frame's padding a hit target without covering the native editor.
    for (key, edge) in [
        (
            "leading",
            Rect::from_min_max(slot.min, egui::pos2(inner.left(), slot.bottom())),
        ),
        (
            "trailing",
            Rect::from_min_max(egui::pos2(inner.right(), slot.top()), slot.max),
        ),
    ] {
        if ui
            .interact(edge, response.id.with(key), Sense::CLICK)
            .clicked()
        {
            response.request_focus();
        }
    }
    response.rect = slot;
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_description(
            "Editing name. Enter submits; Escape cancels. Moving focus keeps the draft.",
        )
    });
    super::super::style::finish(ui, &response);
    response
}

pub(super) fn actions(ui: &mut Ui, slots: &Slots, options: Options<'_>) -> [Response; 2] {
    let action = |ui: &mut Ui, slot: Rect, key, verb, icon, enabled, variant| {
        let mut child = ui.new_child(
            UiBuilder::new()
                .scope_id(ui.scope_id().with(key))
                .max_rect(slot),
        );
        child.set_clip_rect(ui.clip_rect().intersect(slot));
        child.add(
            IconButton::new(key, format!("{verb} {}", options.name), icon)
                .size(options.size)
                .enabled(enabled)
                .variant(variant),
        )
    };
    [
        action(
            ui,
            slots.save,
            "save",
            "Save",
            Icon::Check,
            !options.read_only,
            ButtonVariant::Primary,
        ),
        action(
            ui,
            slots.cancel,
            "cancel",
            "Cancel",
            Icon::Close,
            true,
            ButtonVariant::Ghost,
        ),
    ]
}

pub(super) fn action_ids(ui: &Ui) -> [Id; 2] {
    ["save", "cancel"].map(|key| {
        ui.scope_id()
            .with(key)
            .with(("dutchy-icon-button", egui::IdSalt::new(key)))
    })
}

pub(super) fn editor_id(ui: &Ui) -> Id {
    ui.scope_id()
        .with("editor-slot")
        .with(egui::IdSalt::new("editor"))
}

fn pencil(p: &egui::Painter, rect: Rect, color: egui::Color32) {
    let point = |x, y| rect.min + vec2(rect.width() * x, rect.height() * y);
    let stroke = Stroke::new(border::STANDARD, color);
    p.add(egui::Shape::line(
        vec![
            point(0.15, 0.85),
            point(0.2, 0.6),
            point(0.7, 0.1),
            point(0.9, 0.3),
            point(0.4, 0.8),
            point(0.15, 0.85),
        ],
        stroke,
    ));
    p.line_segment([point(0.6, 0.2), point(0.8, 0.4)], stroke);
}
