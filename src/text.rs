//! Typography builder shared by labels and components.

use crate::{
    Theme, fonts,
    tokens::{Family, Leading, TextSize, Tracking, Weight},
};
use egui::{
    Color32, Label, Response, Ui, Widget,
    text::{LayoutJob, TextFormat},
};
use std::borrow::Cow;

/// Semantic text color, resolved against the current theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextColor {
    /// Primary text.
    #[default]
    Foreground,
    /// Secondary text.
    Muted,
    /// Brand accent.
    Primary,
    /// Text on foreground-colored blocks.
    Background,
    /// Error/destructive status.
    Destructive,
    /// Successful status.
    Success,
    /// Explicit application color.
    Custom(Color32),
}

impl TextColor {
    /// Resolve a semantic color.
    pub const fn resolve(self, theme: &Theme) -> Color32 {
        let p = theme.palette;
        match self {
            Self::Foreground => p.foreground,
            Self::Muted => p.muted_foreground,
            Self::Primary => p.primary,
            Self::Background => p.background,
            Self::Destructive => p.destructive,
            Self::Success => p.success,
            Self::Custom(color) => color,
        }
    }
}

impl From<Color32> for TextColor {
    fn from(color: Color32) -> Self {
        Self::Custom(color)
    }
}

/// Composable Dutchy typography. Defaults to 16-point body text.
#[derive(Clone, Debug)]
#[must_use = "Add this text to a Ui, or turn it into a LayoutJob"]
pub struct Text<'a> {
    text: Cow<'a, str>,
    family: Family,
    size: TextSize,
    weight: Weight,
    tracking: Tracking,
    leading: Option<Leading>,
    color: TextColor,
    uppercase: bool,
    italic: bool,
}

impl<'a> Text<'a> {
    /// Body text.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            family: Family::Body,
            size: TextSize::Base,
            weight: Weight::Regular,
            tracking: Tracking::Normal,
            leading: None,
            color: TextColor::Foreground,
            uppercase: false,
            italic: false,
        }
    }

    /// Bold uppercase Space Grotesk with tight tracking.
    pub fn heading(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text)
            .family(Family::Display)
            .size(TextSize::Xl5)
            .weight(Weight::Bold)
            .tracking(Tracking::Tighter)
            .uppercase(true)
    }

    /// Uppercase technical label.
    pub fn label(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text)
            .family(Family::Mono)
            .size(TextSize::Xs)
            .weight(Weight::Bold)
            .tracking(Tracking::Widest)
            .uppercase(true)
    }

    /// Typeface role.
    pub fn family(mut self, family: Family) -> Self {
        self.family = family;
        self
    }
    /// Type scale.
    pub fn size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }
    /// Static font face weight.
    pub fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }
    /// Letter spacing in em units.
    pub fn tracking(mut self, tracking: Tracking) -> Self {
        self.tracking = tracking;
        self
    }
    /// Override the scale's default line height.
    pub fn leading(mut self, leading: Leading) -> Self {
        self.leading = Some(leading);
        self
    }
    /// Semantic or explicit color.
    pub fn color(mut self, color: impl Into<TextColor>) -> Self {
        self.color = color.into();
        self
    }
    /// Apply Unicode uppercase conversion to this label only.
    pub fn uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }
    /// Apply egui's italic treatment; bundled faces are upright.
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Build a reusable layout job. `wrap_width` is in logical points.
    pub fn layout_job(&self, theme: &Theme, wrap_width: f32) -> LayoutJob {
        let size = self.size.points();
        let content = if self.uppercase {
            self.text.to_uppercase()
        } else {
            self.text.to_string()
        };
        let mut job = LayoutJob::simple_format(
            content,
            TextFormat {
                font_id: fonts::font_id(self.family, self.weight, size),
                extra_letter_spacing: self.tracking.em() * size,
                line_height: Some(
                    size * self
                        .leading
                        .map_or(self.size.line_height(), Leading::factor),
                ),
                color: self.color.resolve(theme),
                italics: self.italic,
                ..Default::default()
            },
        );
        job.wrap.max_width = wrap_width.max(0.0);
        job
    }
}

impl Widget for Text<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        if matches!(self.color, TextColor::Foreground | TextColor::Muted)
            && let Some(color) = ui.visuals().override_text_color
        {
            self.color = TextColor::Custom(color);
        }
        ui.add(Label::new(self.layout_job(&theme, ui.available_width())).wrap())
    }
}
