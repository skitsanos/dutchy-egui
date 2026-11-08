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

/// Expose transformed text as written: assistive technology derives a label
/// node's name from its value, so both fields carry the host's string.
pub(crate) fn name_as_written(response: &Response, text: &str) {
    let text = text.to_string();
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_label(text.clone());
        node.set_value(text);
    });
}

impl Widget for Text<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        if matches!(self.color, TextColor::Foreground | TextColor::Muted)
            && let Some(color) = ui.visuals().override_text_color
        {
            self.color = TextColor::Custom(color);
        }
        // Uppercase is a visual treatment; assistive technology gets the text as written.
        let name = self.uppercase.then(|| self.text.to_string());
        let response = add_ragged(ui, self.layout_job(&theme, ui.available_width()));
        if let Some(name) = &name {
            name_as_written(&response, name);
        }
        response
    }
}

/// Add wrapped text without inheriting the layout's horizontal justification.
///
/// egui's `Label` copies `Layout::horizontal_justify` into its layout job, so
/// wrapped copy inside `Ui::columns`, `add_sized` boxes or other justified
/// layouts stretches every interior line to the full width. In those layouts
/// the text is laid out here with justification off and handed to `Label` as a
/// finished galley; the allocated rect, placement and name are unchanged.
pub(crate) fn add_ragged(ui: &mut Ui, mut job: LayoutJob) -> Response {
    if !ui.layout().horizontal_justify() {
        return ui.add(Label::new(job).wrap());
    }
    job.wrap.max_width = ui.available_width();
    add_measured(ui, job)
}

/// Add text laid out at the job's own wrap width.
///
/// egui's `Label` substitutes the available width for a wrapped job's
/// `wrap.max_width`; a finished galley keeps the narrower measure, so a row
/// that reserves space beside the text (such as a card title next to its
/// arrow) stays clear of it. Justification is never inherited.
pub(crate) fn add_measured(ui: &mut Ui, mut job: LayoutJob) -> Response {
    job.justify = false;
    job.halign = ui.layout().horizontal_placement();
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.add(Label::new(galley))
}
