use crate::{
    Text, TextSize, Theme, Tracking, Weight,
    tokens::{border, close as geometry, space},
    widgets::close,
};
use egui::accesskit::Role;
use egui::{
    Color32, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget, WidgetInfo, pos2, vec2,
};
use std::borrow::Cow;

/// Status and category badge treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Foreground block.
    #[default]
    Default,
    /// Brand accent.
    Primary,
    /// Successful status.
    Success,
    /// Error/destructive status.
    Destructive,
    /// Warning status.
    Warning,
    /// Informational status.
    Info,
    /// Strong border and transparent fill.
    Outline,
    /// Subtle background and text.
    Muted,
}

impl BadgeVariant {
    /// All supported variants in display order.
    pub const ALL: [Self; 8] = [
        Self::Default,
        Self::Primary,
        Self::Success,
        Self::Destructive,
        Self::Warning,
        Self::Info,
        Self::Outline,
        Self::Muted,
    ];

    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Primary => "Primary",
            Self::Success => "Success",
            Self::Destructive => "Destructive",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Outline => "Outline",
            Self::Muted => "Muted",
        }
    }

    fn colors(self, theme: Theme) -> (Color32, Color32, Stroke) {
        let p = theme.palette;
        let (fill, text) = match self {
            Self::Default => (p.foreground, p.background),
            Self::Primary => (p.primary, p.on_primary),
            Self::Success => (p.success, p.on_status),
            Self::Destructive => (p.destructive, p.on_status),
            Self::Warning => (p.warning, p.on_status),
            Self::Info => (p.info, p.on_status),
            Self::Outline => (Color32::TRANSPARENT, p.foreground),
            Self::Muted => (p.muted, p.muted_foreground),
        };
        let stroke = if self == Self::Outline {
            Stroke::new(border::STANDARD, p.foreground)
        } else {
            Stroke::NONE
        };
        (fill, text, stroke)
    }
}

/// Compact uppercase badge. For a closable badge, `Response::clicked` requests removal.
///
/// Only the close affordance activates; clicking the label does not dismiss it.
#[derive(Clone, Debug)]
#[must_use = "Add this badge to a Ui"]
pub struct Badge<'a> {
    label: Cow<'a, str>,
    variant: BadgeVariant,
    closable: bool,
}

impl<'a> Badge<'a> {
    /// Default foreground-colored badge.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            closable: false,
        }
    }
    /// Select a semantic treatment.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Add a named, keyboard-accessible dismiss control.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

impl Widget for Badge<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        let (fill, color, stroke) = self.variant.colors(theme);
        let job = Text::new(self.label.clone())
            .size(TextSize::Xs)
            .weight(Weight::Bold)
            .tracking(Tracking::Wider)
            .uppercase(true)
            .color(color)
            .layout_job(&theme, f32::INFINITY);
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let close_width = if self.closable { space::XXL } else { 0.0 };
        let desired = vec2(
            galley.size().x + 2.0 * space::MD + close_width,
            (galley.size().y + 2.0 * space::XS).max(space::XXL),
        );
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
        response
            .widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), self.label.as_ref()));
        if ui.is_rect_visible(rect) {
            ui.painter().rect(rect, 0, fill, stroke, StrokeKind::Inside);
            ui.painter().galley(
                pos2(
                    rect.left() + space::MD,
                    rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                color,
            );
        }
        if !self.closable {
            return response;
        }
        let close_rect = Rect::from_min_max(
            pos2(rect.right() - space::MD - space::LG, rect.top()),
            pos2(rect.right() - space::XS, rect.bottom()),
        );
        let close = ui.interact(close_rect, response.id.with("dismiss"), Sense::click());
        close.widget_info(|| {
            WidgetInfo::labeled(
                Role::Button,
                ui.is_enabled(),
                format!("Dismiss {}", self.label),
            )
        });
        close::paint(
            ui,
            &close,
            geometry::BADGE_ICON,
            close::Colors {
                idle: color.gamma_multiply(geometry::IDLE_OPACITY),
                foreground: color,
                focus: theme.palette.primary,
            },
        );
        response.union(close)
    }
}
