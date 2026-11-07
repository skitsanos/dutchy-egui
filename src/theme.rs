//! Palette and typography adaptation for egui's stable style API.

use egui::style::{Selection, WidgetVisuals};
use egui::{Context, CornerRadius, Id, Stroke, Style, TextStyle, Visuals, vec2};

use crate::{
    fonts,
    tokens::{Accent, ControlSize, Family, Mode, Palette, Weight, border, motion, space},
};

/// A complete Dutchy theme. Apply only at setup or when settings change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    /// Semantic colors; callers can override individual tokens.
    pub palette: Palette,
    /// Active light/dark mode.
    pub mode: Mode,
    /// Disable motion while retaining static loading indicators.
    pub reduced_motion: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(Accent::Orange, Mode::Light)
    }
}

impl Theme {
    /// Construct a preset.
    pub const fn new(accent: Accent, mode: Mode) -> Self {
        Self {
            palette: Palette::new(accent, mode),
            mode,
            reduced_motion: false,
        }
    }

    /// Opt into a static presentation of animated components.
    pub const fn with_reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced_motion = reduced;
        self
    }

    /// Install style and widget tokens. Does not modify fonts or display scale.
    ///
    /// `Mode` decides the palette, so the style is installed for both egui
    /// themes: a later theme-preference change (a host following the system
    /// theme, a test harness) cannot drop native widgets back to egui defaults.
    pub fn apply(self, ctx: &Context) {
        ctx.set_theme(match self.mode {
            Mode::Light => egui::Theme::Light,
            Mode::Dark => egui::Theme::Dark,
        });
        let style = self.style();
        ctx.set_style_of(egui::Theme::Light, style.clone());
        ctx.set_style_of(egui::Theme::Dark, style);
        ctx.data_mut(|data| data.insert_temp(theme_id(), self));
        ctx.request_repaint();
    }

    /// Read the installed theme; defaults to Dutch Orange on an unconfigured context.
    pub fn from_context(ctx: &Context) -> Self {
        ctx.data(|data| data.get_temp(theme_id()).unwrap_or_default())
    }

    /// Build an egui style without modifying a Context.
    pub fn style(self) -> Style {
        let mut style = Style {
            visuals: self.visuals(),
            ..Default::default()
        };
        style.spacing.item_spacing = vec2(space::SM, space::SM);
        style.spacing.button_padding = vec2(space::LG, space::SM);
        style.spacing.interact_size =
            vec2(ControlSize::Medium.height(), ControlSize::Medium.height());
        style.spacing.indent = space::XXL;
        style.animation_time = if self.reduced_motion {
            0.0
        } else {
            motion::FAST
        };
        style.text_styles = [
            (
                TextStyle::Body,
                fonts::font_id(Family::Body, Weight::Regular, 16.0),
            ),
            (
                TextStyle::Button,
                fonts::font_id(Family::Body, Weight::Bold, 16.0),
            ),
            (
                TextStyle::Heading,
                fonts::font_id(Family::Display, Weight::Bold, 30.0),
            ),
            (
                TextStyle::Small,
                fonts::font_id(Family::Body, Weight::Regular, 12.0),
            ),
            (
                TextStyle::Monospace,
                fonts::font_id(Family::Mono, Weight::Regular, 14.0),
            ),
        ]
        .into();
        style
    }

    fn visuals(self) -> Visuals {
        let p = self.palette;
        let mut v = match self.mode {
            Mode::Light => Visuals::light(),
            Mode::Dark => Visuals::dark(),
        };
        v.window_corner_radius = CornerRadius::ZERO;
        v.menu_corner_radius = CornerRadius::ZERO;
        v.window_shadow = egui::epaint::Shadow::NONE;
        v.popup_shadow = egui::epaint::Shadow::NONE;
        v.window_fill = p.background;
        v.panel_fill = p.background;
        v.window_stroke = Stroke::new(border::STANDARD, p.border);
        v.extreme_bg_color = p.background;
        v.text_edit_bg_color = Some(p.background);
        v.faint_bg_color = p.muted;
        v.code_bg_color = p.muted;
        v.weak_text_color = Some(p.muted_foreground);
        v.hyperlink_color = p.primary;
        v.warn_fg_color = p.warning;
        v.error_fg_color = p.destructive;
        v.selection = Selection {
            bg_fill: p.primary,
            stroke: Stroke::new(border::THIN, p.on_primary),
        };
        v.widgets.noninteractive = widget_visuals(p.background, p.border, p.foreground);
        v.widgets.inactive = widget_visuals(p.muted, p.border, p.foreground);
        v.widgets.hovered = widget_visuals(p.secondary, p.primary, p.foreground);
        v.widgets.active = widget_visuals(p.primary, p.primary, p.on_primary);
        v.widgets.open = widget_visuals(p.secondary, p.primary, p.foreground);
        v.interact_cursor = Some(egui::CursorIcon::PointingHand);
        v
    }
}

pub(crate) fn widget_visuals(
    fill: egui::Color32,
    border: egui::Color32,
    text: egui::Color32,
) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(crate::tokens::border::STANDARD, border),
        corner_radius: CornerRadius::ZERO,
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

fn theme_id() -> Id {
    Id::unique("dutchy::theme")
}
