mod grid;
mod style;
mod trend;

use crate::{
    Family, Text, TextColor, TextSize, Theme, Tracking, Weight,
    tokens::{border, metrics, space},
};
use egui::{Frame, Id, IdSalt, Margin, Response, Ui, UiBuilder, Widget};
pub use grid::{StatsGrid, StatsLayout};
use std::borrow::Cow;
pub use trend::{Trend, TrendDirection, TrendSentiment};

/// Statistic tile surface, matching the web StatsGrid component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatVariant {
    /// Background surface.
    #[default]
    Light,
    /// Muted surface.
    Muted,
    /// Inverted foreground surface.
    Dark,
    /// Primary surface.
    Primary,
}

impl StatVariant {
    /// Supported tile surfaces.
    pub const ALL: [Self; 4] = [Self::Light, Self::Muted, Self::Dark, Self::Primary];
    /// Human-readable surface name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Muted => "Muted",
            Self::Dark => "Dark",
            Self::Primary => "Primary",
        }
    }
}

/// Statistic type scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatSize {
    /// Responsive 36/48-point value.
    #[default]
    Default,
    /// 30-point value.
    Small,
}

/// Formatted KPI data. The host computes and formats values and trends.
#[derive(Clone, Debug)]
#[must_use = "Add the stat to a Ui or a StatsGrid"]
pub struct Stat<'a> {
    label: Cow<'a, str>,
    value: Cow<'a, str>,
    id: IdSalt,
    variant: StatVariant,
    trend: Option<Trend<'a>>,
    description: Option<Cow<'a, str>>,
    size: StatSize,
    accented: bool,
}

impl<'a> Stat<'a> {
    /// Bordered standalone stat with a label and already formatted value.
    pub fn new(label: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        Self {
            id: IdSalt::new(label.as_ref()),
            label,
            value: value.into(),
            variant: StatVariant::Light,
            trend: None,
            description: None,
            size: StatSize::Default,
            accented: false,
        }
    }
    /// Stable salt when labels repeat or change.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = IdSalt::new(id);
        self
    }
    /// Tile colors.
    pub fn variant(mut self, variant: StatVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Explicit change direction/sentiment.
    pub fn trend(mut self, trend: Trend<'a>) -> Self {
        self.trend = Some(trend);
        self
    }
    /// Additional context, such as a previous value or reporting period.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Standalone type scale. StatsGrid supplies its shared size instead.
    pub fn size(mut self, size: StatSize) -> Self {
        self.size = size;
        self
    }
    /// Left accent instead of the standalone full border; also honored in grids.
    pub fn accented(mut self, accented: bool) -> Self {
        self.accented = accented;
        self
    }

    fn tile(
        &self,
        ui: &mut Ui,
        id: Id,
        size: StatSize,
        compact: bool,
        bordered: bool,
    ) -> style::Tile {
        let painter = ui.painter().clone();
        let shape = painter.add(egui::Shape::Noop);
        let theme = Theme::from_context(ui.ctx());
        let mut fill = theme.palette.background;
        let mut margin = Margin::same(if compact { space::LG } else { space::XXL } as i8);
        if bordered && !self.accented {
            margin += border::ACCENT as i8;
        } else if self.accented {
            margin.left += border::ACCENT as i8;
        }
        let result = ui.scope_builder(
            UiBuilder::new()
                .scope_id(id)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                fill = style::colors(ui, self.variant);
                Frame::NONE.inner_margin(margin).show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.with_layout(
                        egui::Layout::top_down(if compact {
                            egui::Align::Center
                        } else {
                            egui::Align::Min
                        }),
                        |ui| self.body(ui, size, compact),
                    );
                });
            },
        );
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| {
                node.set_role(egui::accesskit::Role::Group);
                node.set_label(self.label.to_string());
                node.set_value(self.value.to_string());
                let mut context = self.description.as_deref().unwrap_or("").to_string();
                if let Some(trend) = &self.trend {
                    if !context.is_empty() {
                        if !matches!(context.chars().last(), Some('.' | '!' | '?')) {
                            context.push('.');
                        }
                        context.push(' ');
                    }
                    context.push_str(&trend.summary());
                }
                if !context.is_empty() {
                    node.set_description(context);
                }
            });
        style::Tile {
            rect: result.response.rect,
            response: result.response,
            painter,
            shape,
            fill,
            edge: if self.accented {
                theme.palette.primary
            } else {
                theme.palette.border
            },
            accent: self.accented,
            bordered,
        }
    }

    fn body(&self, ui: &mut Ui, size: StatSize, compact: bool) {
        let value_size = if compact {
            TextSize::Xl2
        } else if size == StatSize::Small {
            TextSize::Xl3
        } else if ui.available_width() >= metrics::LARGE_VALUE_WIDTH {
            TextSize::Xl5
        } else {
            TextSize::Xl4
        };
        ui.add(
            Text::new(self.value.as_ref())
                .family(Family::Display)
                .size(value_size)
                .weight(Weight::Bold)
                .tracking(Tracking::Tight),
        );
        ui.add(
            Text::new(self.label.as_ref())
                .size(if compact { TextSize::Xs } else { TextSize::Sm })
                .weight(if compact {
                    Weight::Bold
                } else {
                    Weight::default()
                })
                .tracking(Tracking::Wider)
                .uppercase(true)
                .color(TextColor::Muted),
        );
        if let Some(trend) = &self.trend {
            trend.show(ui);
        }
        if let Some(description) = &self.description {
            ui.add(
                Text::new(description.as_ref())
                    .size(TextSize::Sm)
                    .color(TextColor::Muted),
            );
        }
    }
}

impl Widget for Stat<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let tile = self.tile(
            ui,
            ui.make_persistent_id(("dutchy-stat", self.id)),
            self.size,
            false,
            true,
        );
        tile.finish(tile.rect.bottom());
        tile.response
    }
}
