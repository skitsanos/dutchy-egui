use crate::widgets::spinner::{SpinnerSize, paint_spinner};
use crate::{
    ButtonVariant, ControlSize, Icon, Text, Theme, Tooltip, Tracking, Weight,
    interaction::{paint_focus, paint_focus_inside},
    tokens::actions as tokens,
};
use egui::{
    Id, Response, Sense, StrokeKind, Ui, UiBuilder, WidgetInfo,
    accesskit::{Role, Toggled},
    pos2, vec2,
};

pub(crate) struct Action<'a> {
    pub id: Id,
    pub label: &'a str,
    pub icon: Option<Icon>,
    pub icon_only: bool,
    pub pressed: Option<bool>,
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub enabled: bool,
    pub loading: bool,
    pub tooltip: Option<&'a str>,
    pub inside_focus: bool,
    pub menu: bool,
    /// Accessibility role, registered once per frame.
    pub role: Role,
}

pub(crate) fn show(
    ui: &mut Ui,
    mut action: Action<'_>,
    sense: Sense,
    bound: Option<&mut bool>,
) -> Response {
    let theme = Theme::from_context(ui.ctx());
    let icon_only = action.icon_only && action.icon.is_some();
    let galley = (!icon_only).then(|| {
        let mut job = Text::new(action.label)
            .size(action.size.text_size())
            .weight(Weight::Bold)
            .tracking(Tracking::Wide)
            .uppercase(true)
            .layout_job(&theme, f32::INFINITY);
        job.wrap.max_rows = 1;
        ui.fonts_mut(|f| f.layout_job(job))
    });
    let icon_width = action
        .icon
        .map_or(0.0, |_| tokens::icon(action.size) + tokens::ICON_GAP);
    let natural = if icon_only {
        action.size.height()
    } else {
        galley.as_ref().map_or(0.0, |g| g.size().x) + 2.0 * action.size.padding() + icon_width
    } + if action.menu && !icon_only {
        tokens::icon(action.size)
    } else {
        0.0
    };
    let (rect, _) = ui.allocate_exact_size(
        vec2(
            natural.min(ui.max_rect().width()).max(0.0),
            action.size.height(),
        ),
        Sense::hover(),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .scope_id(action.id.with("surface"))
            .max_rect(rect),
    );
    if !action.enabled || action.loading {
        child.disable();
    }
    let ui = &mut child;
    focus_request(ui, action.id);
    let sense = if ui.memory(|m| m.has_focus(action.id)) {
        Sense::click()
    } else {
        sense
    };
    let mut response = ui.interact(rect, action.id, sense);
    if response.clicked() {
        response.request_focus();
        if let Some(value) = bound {
            *value = !*value;
            action.pressed = Some(*value);
            response.mark_changed();
        }
    }
    response.widget_info(|| WidgetInfo::labeled(action.role, ui.is_enabled(), action.label));
    response.ctx.accesskit_node_builder(response.id, |node| {
        if ui.is_enabled() {
            node.add_action(egui::accesskit::Action::Focus);
        }
        if let Some(pressed) = action.pressed {
            node.set_toggled(if pressed {
                Toggled::True
            } else {
                Toggled::False
            });
        }
        if action.loading {
            node.set_busy();
        }
    });
    paint(ui, &response, &action, galley, icon_only);
    if let Some(help) = action.tooltip.or(icon_only.then_some(action.label)) {
        Tooltip::new(help).show(&response);
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

// Roving composites omit inactive items from native Tab registration. They
// still accept an explicit assistive-technology focus request by stable ID.
fn focus_request(ui: &Ui, id: Id) {
    if ui.is_enabled()
        && ui.input(|i| {
            i.events.iter().any(|event| {
                matches!(event, egui::Event::AccessKitActionRequest(request)
            if request.action == egui::accesskit::Action::Focus
                && request.target_tree == egui::accesskit::TreeId::ROOT
                && request.target_node == id.accesskit_id())
            })
        })
    {
        ui.memory_mut(|m| m.request_focus(id));
    }
}

fn paint(
    ui: &Ui,
    response: &Response,
    action: &Action<'_>,
    galley: Option<std::sync::Arc<egui::Galley>>,
    icon_only: bool,
) {
    if !ui.is_rect_visible(response.rect) {
        return;
    }
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    let visuals = visuals(ui, response, action, theme);
    let painter = ui
        .painter()
        .with_clip_rect(ui.clip_rect().intersect(response.rect));
    painter.rect(
        response.rect,
        0,
        visuals.bg_fill,
        visuals.bg_stroke,
        StrokeKind::Inside,
    );
    paint_pressed_marker(&painter, response.rect, action, p.primary);
    let ink = visuals.fg_stroke.color;
    if action.menu && !icon_only && !action.loading {
        let offset = tokens::icon(action.size) / 4.0;
        let center = pos2(
            response.rect.right() - action.size.padding() - offset,
            response.rect.center().y,
        );
        painter.line_segment(
            [
                center + vec2(-offset, -offset / 2.0),
                center + vec2(0.0, offset / 2.0),
            ],
            egui::Stroke::new(crate::tokens::border::STANDARD, ink),
        );
        painter.line_segment(
            [
                center + vec2(0.0, offset / 2.0),
                center + vec2(offset, -offset / 2.0),
            ],
            egui::Stroke::new(crate::tokens::border::STANDARD, ink),
        );
    }
    let extent = tokens::icon(action.size)
        .min(response.rect.width())
        .min(response.rect.height());
    if action.loading {
        let size = match action.size {
            ControlSize::Small => SpinnerSize::Small,
            ControlSize::Medium => SpinnerSize::Medium,
            ControlSize::Large => SpinnerSize::Large,
        };
        paint_spinner(ui, response.rect.center(), size, ink, !theme.reduced_motion);
    } else {
        if let Some(icon) = action.icon {
            let center = if icon_only {
                response.rect.center()
            } else {
                pos2(
                    response.rect.left() + action.size.padding() + extent / 2.0,
                    response.rect.center().y,
                )
            };
            icon.paint(
                &painter,
                egui::Rect::from_center_size(center, vec2(extent, extent)),
                ink,
            );
        }
        if let Some(galley) = galley {
            let left = response.rect.left()
                + action.size.padding()
                + action.icon.map_or(0.0, |_| extent + tokens::ICON_GAP);
            painter.galley_with_override_text_color(
                pos2(left, response.rect.center().y - galley.size().y / 2.0),
                galley,
                ink,
            );
        }
    }
    let focus = if action.pressed == Some(true) {
        p.background
    } else {
        p.primary
    };
    if action.inside_focus {
        paint_focus_inside(ui, response, focus);
    } else {
        paint_focus(ui, response, p.primary);
    }
}

fn paint_pressed_marker(
    painter: &egui::Painter,
    rect: egui::Rect,
    action: &Action<'_>,
    color: egui::Color32,
) {
    if action.variant != ButtonVariant::Secondary || action.pressed != Some(true) {
        return;
    }
    let inset = crate::tokens::border::STANDARD.min(rect.width() / 2.0);
    let rect = egui::Rect::from_min_max(
        pos2(
            rect.left() + inset,
            rect.bottom() - tokens::PRESSED_MARKER.min(rect.height()),
        ),
        pos2(rect.right() - inset, rect.bottom()),
    );
    painter.rect_filled(rect, 0, color);
}

fn visuals(
    ui: &Ui,
    response: &Response,
    action: &Action<'_>,
    theme: Theme,
) -> egui::style::WidgetVisuals {
    let p = theme.palette;
    let hovered = response.hovered() || response.is_pointer_button_down_on();
    let pressed = action.pressed == Some(true);
    let variant = if pressed {
        ButtonVariant::Secondary
    } else {
        action.variant
    };
    let mut visuals = surface(theme, action, variant, hovered);
    if !hovered && matches!(variant, ButtonVariant::Ghost | ButtonVariant::Outline) {
        let ink = ui.visuals().override_text_color.unwrap_or(p.foreground);
        visuals.fg_stroke.color = ink;
        if variant == ButtonVariant::Outline {
            visuals.bg_stroke.color = ink;
        }
    }
    if !response.enabled() {
        visuals.bg_fill = if pressed { p.foreground } else { p.muted };
        visuals.bg_stroke = egui::Stroke::new(crate::tokens::border::STANDARD, p.border);
        visuals.fg_stroke.color = if pressed {
            p.background
        } else {
            p.muted_foreground
        };
    } else if response.is_pointer_button_down_on() {
        visuals.bg_fill = visuals.bg_fill.gamma_multiply(tokens::PRESSED_TINT);
    }
    visuals
}

/// A toggle owns two surfaces and the inverse one means "on": an unpressed
/// Outline toggle hovers with the muted fill and its stroke instead of
/// borrowing the pressed surface, and a pressed toggle keeps the tinted inverse
/// fill of Secondary instead of switching to the accent.
fn surface(
    theme: Theme,
    action: &Action<'_>,
    variant: ButtonVariant,
    hovered: bool,
) -> egui::style::WidgetVisuals {
    if action.pressed == Some(false) && hovered && variant == ButtonVariant::Outline {
        let mut visuals = ButtonVariant::Ghost.visuals(theme, true);
        visuals.bg_stroke =
            egui::Stroke::new(crate::tokens::border::STANDARD, theme.palette.foreground);
        return visuals;
    }
    variant.visuals(theme, hovered)
}
