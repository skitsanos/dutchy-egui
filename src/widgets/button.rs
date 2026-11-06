use std::borrow::Cow;

use super::spinner::{SpinnerSize, paint_spinner};
use crate::{
    ControlSize, Text, Theme, Tracking, Weight, interaction::paint_focus, theme::widget_visuals,
    tokens::space,
};
use egui::{Color32, Response, Stroke, Ui, Widget, pos2, vec2};

/// Button emphasis and intent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Brand-colored action.
    #[default]
    Primary,
    /// Foreground block with inverse text.
    Secondary,
    /// Strong outline, inverted on hover.
    Outline,
    /// Transparent action with a subtle hover fill.
    Ghost,
    /// Destructive action.
    Destructive,
}

impl ButtonVariant {
    /// All supported variants, in display order.
    pub const ALL: [Self; 5] = [
        Self::Primary,
        Self::Secondary,
        Self::Outline,
        Self::Ghost,
        Self::Destructive,
    ];

    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Outline => "Outline",
            Self::Ghost => "Ghost",
            Self::Destructive => "Destructive",
        }
    }

    fn visuals(self, theme: Theme, hovered: bool) -> egui::style::WidgetVisuals {
        let p = theme.palette;
        let (fill, text, stroke) = match self {
            Self::Primary => (p.primary, p.on_primary, Color32::TRANSPARENT),
            Self::Secondary => (p.foreground, p.background, Color32::TRANSPARENT),
            Self::Destructive => (p.destructive, p.on_status, Color32::TRANSPARENT),
            Self::Outline if hovered => (p.foreground, p.background, p.foreground),
            Self::Outline => (Color32::TRANSPARENT, p.foreground, p.foreground),
            Self::Ghost if hovered => (p.muted, p.foreground, Color32::TRANSPARENT),
            Self::Ghost => (Color32::TRANSPARENT, p.foreground, Color32::TRANSPARENT),
        };
        let fill = if hovered && matches!(self, Self::Primary | Self::Secondary | Self::Destructive)
        {
            fill.gamma_multiply(0.9)
        } else {
            fill
        };
        let mut visuals = widget_visuals(fill, stroke, text);
        if stroke == Color32::TRANSPARENT {
            visuals.bg_stroke = Stroke::NONE;
        }
        visuals
    }
}

/// Bold uppercase button retaining egui's native pointer/keyboard semantics.
#[derive(Clone, Debug)]
#[must_use = "Add this button to a Ui"]
pub struct Button<'a> {
    label: Cow<'a, str>,
    variant: ButtonVariant,
    size: ControlSize,
    enabled: bool,
    loading: bool,
    full_width: bool,
}

impl<'a> Button<'a> {
    /// Primary medium-size action.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            size: ControlSize::Medium,
            enabled: true,
            loading: false,
            full_width: false,
        }
    }

    /// Set emphasis/intent.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Shared control size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disabled controls never activate. Parent disabled state is preserved.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Show a square spinner and disable activation while busy.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Fill available horizontal space.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        let job = self.layout_job(theme);
        let text_width = ui.fonts_mut(|fonts| fonts.layout_job(job.clone()).size().x);
        let width = if self.full_width {
            ui.available_width()
        } else {
            (text_width + 2.0 * self.size.padding()).min(ui.max_rect().width())
        };
        // Allocate before entering the style scope so wrapping parents move
        // the entire control to the next row instead of clipping a child scope.
        ui.allocate_ui_with_layout(
            vec2(width, self.size.height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| self.show(ui, theme, job, text_width, width),
        )
        .inner
    }
}

impl Button<'_> {
    fn layout_job(&self, theme: Theme) -> egui::text::LayoutJob {
        let mut job = Text::new(self.label.clone())
            .size(self.size.text_size())
            .weight(Weight::Bold)
            .tracking(Tracking::Wide)
            .uppercase(true)
            .color(Color32::PLACEHOLDER)
            .layout_job(&theme, f32::INFINITY);
        if self.loading {
            job.sections[0].leading_space = SpinnerSize::Small.extent() + space::SM;
        }
        job
    }

    fn show(
        self,
        ui: &mut Ui,
        theme: Theme,
        job: egui::text::LayoutJob,
        text_width: f32,
        width: f32,
    ) -> Response {
        let normal = self.variant.visuals(theme, false);
        let hover = self.variant.visuals(theme, true);
        let style = ui.style_mut();
        style.visuals.override_text_color = None;
        style.visuals.widgets.noninteractive = normal;
        style.visuals.widgets.inactive = normal;
        style.visuals.widgets.hovered = hover;
        style.visuals.widgets.active = hover;
        style.visuals.widgets.open = hover;
        style.spacing.button_padding = vec2(self.size.padding(), 0.0);
        style.spacing.interact_size.y = self.size.height();
        if !self.enabled || self.loading {
            ui.disable();
        }

        let response = ui.add(
            egui::Button::new(job)
                .truncate()
                .min_size(vec2(width, self.size.height()))
                .corner_radius(0),
        );
        if self.loading {
            response
                .ctx
                .accesskit_node_builder(response.id, |node| node.set_busy());
            let width =
                text_width.min((response.rect.width() - 2.0 * self.size.padding()).max(0.0));
            let center = pos2(
                response.rect.center().x - width / 2.0 + SpinnerSize::Small.extent() / 2.0,
                response.rect.center().y,
            );
            paint_spinner(
                ui,
                center,
                SpinnerSize::Small,
                normal.fg_stroke.color,
                !theme.reduced_motion,
            );
        }
        let focus = match self.variant {
            ButtonVariant::Secondary => theme.palette.foreground,
            ButtonVariant::Destructive => theme.palette.destructive,
            _ => theme.palette.primary,
        };
        paint_focus(ui, &response, focus);
        response
    }
}
