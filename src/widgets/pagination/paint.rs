use crate::{
    Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus,
    tokens::{border, navigation as geometry},
    widgets::nav_icon,
};
use egui::{Color32, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2};
#[derive(Clone, Copy)]
pub(super) enum Kind {
    Page { number: usize, current: bool },
    Direction { next: bool, compact: bool },
}
impl Kind {
    fn label(self) -> String {
        match self {
            Self::Page { number, .. } => format!("Page {number}"),
            Self::Direction { next: true, .. } => "Next page".into(),
            Self::Direction { next: false, .. } => "Previous page".into(),
        }
    }
    fn visible(self) -> String {
        match self {
            Self::Page { number, .. } => number.to_string(),
            Self::Direction {
                compact: true,
                next: true,
            } => "NEXT".into(),
            Self::Direction {
                compact: true,
                next: false,
            } => "PREVIOUS".into(),
            _ => String::new(),
        }
    }
    fn compact(self) -> bool {
        matches!(self, Self::Direction { compact: true, .. })
    }
    fn current(self) -> bool {
        matches!(self, Self::Page { current: true, .. })
    }
}
fn text(ui: &Ui, value: &str, bold: bool) -> std::sync::Arc<egui::Galley> {
    let job = Text::new(value)
        .size(TextSize::Sm)
        .weight(if bold { Weight::Bold } else { Weight::Regular })
        .tracking(Tracking::Wide)
        .layout_job(&Theme::from_context(ui.ctx()), f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}
pub(super) fn control(ui: &mut Ui, id: Id, kind: Kind, enabled: bool) -> Response {
    let galley = text(ui, &kind.visible(), true);
    let width = if kind.compact() {
        galley.size().x + geometry::ICON_SIZE + geometry::PAGE_GAP + geometry::PAGE_PADDING * 2.0
    } else {
        geometry::PAGE_SIZE.max(galley.size().x + geometry::PAGE_PADDING * 2.0)
    }
    .min(ui.max_rect().width());
    ui.allocate_ui_with_layout(
        vec2(width, geometry::PAGE_SIZE),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            if !enabled {
                ui.disable();
            }
            let (_, rect) = ui.allocate_space(vec2(width, geometry::PAGE_SIZE));
            let response = ui.interact(rect, id, Sense::click());
            response.widget_info(|| {
                WidgetInfo::labeled(egui::accesskit::Role::Button, ui.is_enabled(), kind.label())
            });
            if kind.current() {
                ui.ctx().accesskit_node_builder(id, |node| {
                    node.set_aria_current(egui::accesskit::AriaCurrent::Page)
                });
            }
            draw(ui, &response, kind, galley);
            paint_focus(ui, &response, Theme::from_context(ui.ctx()).palette.primary);
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        },
    )
    .inner
}
fn draw(ui: &Ui, response: &Response, kind: Kind, galley: std::sync::Arc<egui::Galley>) {
    let rect = response.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let hover = response.hovered() || response.has_focus();
    // Live page numbers inherit the foreground; direction controls rest muted.
    let color = if kind.current() {
        p.on_primary
    } else if hover || matches!(kind, Kind::Page { .. }) {
        p.foreground
    } else {
        p.muted_foreground
    };
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    if !kind.compact() {
        let stroke = if kind.current() {
            p.primary
        } else if hover {
            p.foreground
        } else {
            p.border
        };
        painter.rect(
            rect,
            0,
            if kind.current() {
                p.primary
            } else {
                Color32::TRANSPARENT
            },
            Stroke::new(border::STANDARD, stroke),
            StrokeKind::Inside,
        );
    }
    let center = if let Kind::Direction { next, compact } = kind {
        direction(&painter, rect, color, next, compact)
    } else {
        rect.center()
    };
    painter.galley_with_override_text_color(center - galley.size() / 2.0, galley, color);
}
fn direction(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    next: bool,
    compact: bool,
) -> egui::Pos2 {
    let icon_center = if !compact {
        rect.center()
    } else if next {
        pos2(
            rect.right() - geometry::PAGE_PADDING - geometry::ICON_SIZE / 2.0,
            rect.center().y,
        )
    } else {
        pos2(
            rect.left() + geometry::PAGE_PADDING + geometry::ICON_SIZE / 2.0,
            rect.center().y,
        )
    };
    nav_icon::chevron(
        painter,
        Rect::from_center_size(icon_center, vec2(geometry::ICON_SIZE, geometry::ICON_SIZE)),
        color,
        next,
    );
    if !compact {
        return rect.center();
    }
    let offset = (geometry::ICON_SIZE + geometry::PAGE_GAP) / 2.0;
    rect.center() + vec2(if next { -offset } else { offset }, 0.0)
}
pub(super) fn ellipsis(ui: &mut Ui) {
    let (_, rect) = ui.allocate_space(vec2(geometry::PAGE_SIZE, geometry::PAGE_SIZE));
    let galley = text(ui, "…", false);
    ui.painter().galley_with_override_text_color(
        rect.center() - galley.size() / 2.0,
        galley,
        Theme::from_context(ui.ctx()).palette.muted_foreground,
    );
}

/// Muted summary; page numbers, when given, are bold foreground sections
/// (live "Page <strong>3</strong> of <strong>12</strong>").
pub(super) fn summary(ui: &mut Ui, label: &str, numbers: Option<(usize, usize)>) {
    let theme = Theme::from_context(ui.ctx());
    let width = ui.max_rect().width();
    let job = match numbers {
        Some((page, total)) => {
            let mut job = Text::new("Page ")
                .size(TextSize::Sm)
                .color(crate::TextColor::Muted)
                .layout_job(&theme, width);
            let base = job.sections[0].format.clone();
            let strong = Text::new(" ")
                .size(TextSize::Sm)
                .weight(crate::Weight::Bold)
                .color(crate::TextColor::Foreground)
                .layout_job(&theme, width)
                .sections[0]
                .format
                .clone();
            job.append(&page.to_string(), 0.0, strong.clone());
            job.append(" of ", 0.0, base);
            job.append(&total.to_string(), 0.0, strong);
            job
        }
        None => Text::new(label)
            .size(TextSize::Sm)
            .color(crate::TextColor::Muted)
            .layout_job(&theme, width),
    };
    let text = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let size = vec2(text.size().x, geometry::PAGE_SIZE.max(text.size().y));
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        response.widget_info(|| {
            WidgetInfo::labeled(egui::accesskit::Role::Label, ui.is_enabled(), label)
        });
        ui.painter().galley(
            rect.center() - text.size() / 2.0,
            text,
            theme.palette.muted_foreground,
        );
    });
}
