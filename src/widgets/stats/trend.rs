use crate::{
    Text, TextSize, Theme, Weight,
    tokens::{border, metrics},
};
use egui::{Color32, Sense, Stroke, Ui, vec2};
use std::borrow::Cow;

/// Change direction, independent of whether that change is desirable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrendDirection {
    /// Increasing metric.
    Up,
    /// Decreasing metric.
    Down,
    /// No directional change.
    Flat,
}

impl TrendDirection {
    /// Text alternative to the decorative arrow.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Flat => "Unchanged",
        }
    }
}

/// Application-supplied interpretation; no numeric signs are inferred.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrendSentiment {
    /// Desirable change.
    Positive,
    /// Undesirable change.
    Negative,
    /// No judgment; default.
    #[default]
    Neutral,
}

impl TrendSentiment {
    /// Accessible interpretation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Positive => "Positive",
            Self::Negative => "Negative",
            Self::Neutral => "Neutral",
        }
    }
    fn color(self, theme: Theme) -> Color32 {
        match self {
            Self::Positive => theme.palette.success,
            Self::Negative => theme.palette.destructive,
            Self::Neutral => theme.palette.muted_foreground,
        }
    }
}

/// Formatted change text with explicit direction, sentiment, and comparison.
#[derive(Clone, Debug)]
pub struct Trend<'a> {
    value: Cow<'a, str>,
    direction: TrendDirection,
    sentiment: TrendSentiment,
    comparison: Option<Cow<'a, str>>,
}

impl<'a> Trend<'a> {
    /// Neutral trend; the host formats the value, including its sign and units.
    pub fn new(value: impl Into<Cow<'a, str>>, direction: TrendDirection) -> Self {
        Self {
            value: value.into(),
            direction,
            sentiment: TrendSentiment::Neutral,
            comparison: None,
        }
    }
    /// Set whether the change is desirable.
    pub fn sentiment(mut self, sentiment: TrendSentiment) -> Self {
        self.sentiment = sentiment;
        self
    }
    /// Context such as "vs last month" or "fewer failed requests".
    pub fn comparison(mut self, comparison: impl Into<Cow<'a, str>>) -> Self {
        self.comparison = Some(comparison.into());
        self
    }

    pub(super) fn summary(&self) -> String {
        let comparison = self
            .comparison
            .as_deref()
            .map_or(String::new(), |text| format!(", {text}"));
        format!(
            "{} {}, {}{comparison}",
            self.direction.name(),
            self.value,
            self.sentiment.name()
        )
    }

    pub(super) fn show(&self, ui: &mut Ui) {
        let theme = Theme::from_context(ui.ctx());
        let color = if self.sentiment == TrendSentiment::Neutral {
            ui.visuals()
                .override_text_color
                .unwrap_or(self.sentiment.color(theme))
        } else {
            self.sentiment.color(theme)
        };
        let text_width = (ui.available_width() - metrics::TREND_ICON - metrics::TREND_GAP).max(0.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = metrics::TREND_GAP;
            let (rect, decorative) = ui.allocate_exact_size(
                vec2(metrics::TREND_ICON, metrics::TREND_ICON),
                Sense::hover(),
            );
            decorative
                .ctx
                .accesskit_node_builder(decorative.id, |node| node.set_hidden());
            let point = |x, y| rect.min + vec2(x * rect.width(), y * rect.height());
            let stroke = Stroke::new(border::STANDARD, color);
            match self.direction {
                TrendDirection::Flat => {
                    ui.painter()
                        .line_segment([point(0.1, 0.5), point(0.9, 0.5)], stroke);
                }
                direction => {
                    let (a, b) = if direction == TrendDirection::Up {
                        (0.8, 0.2)
                    } else {
                        (0.2, 0.8)
                    };
                    ui.painter()
                        .line_segment([point(0.5, a), point(0.5, b)], stroke);
                    ui.painter().add(egui::Shape::line(
                        vec![point(0.2, 0.5), point(0.5, b), point(0.8, 0.5)],
                        stroke,
                    ));
                }
            }
            ui.vertical(|ui| {
                ui.set_width(text_width);
                ui.add(
                    Text::new(format!("{} {}", self.direction.name(), self.value))
                        .size(TextSize::Sm)
                        .weight(Weight::Bold)
                        .color(color),
                );
                if let Some(comparison) = &self.comparison {
                    ui.add(
                        Text::new(comparison.as_ref())
                            .size(TextSize::Sm)
                            .color(crate::TextColor::Muted),
                    );
                }
            });
        });
    }
}
