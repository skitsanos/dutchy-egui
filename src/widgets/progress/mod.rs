mod paint;

use crate::{
    Family, ProgressSize, Text, TextColor, TextSize, Theme, Tracking, Weight,
    tokens::{loading, space},
};
use egui::accesskit::Role;
use egui::{IdSalt, Response, Sense, Ui, UiBuilder, Widget, WidgetInfo, vec2};
use std::borrow::Cow;

/// Fill treatment. Give the operation a meaningful label as well as a color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressVariant {
    /// Brand accent.
    #[default]
    Primary,
    /// Successful completion.
    Success,
    /// Failed/destructive operation.
    Destructive,
    /// Main foreground.
    Foreground,
}

impl ProgressVariant {
    /// All supported fills.
    pub const ALL: [Self; 4] = [
        Self::Primary,
        Self::Success,
        Self::Destructive,
        Self::Foreground,
    ];
    /// Human-readable fill name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Success => "Success",
            Self::Destructive => "Destructive",
            Self::Foreground => "Foreground",
        }
    }
    fn color(self, theme: Theme) -> egui::Color32 {
        match self {
            Self::Primary => theme.palette.primary,
            Self::Success => theme.palette.success,
            Self::Destructive => theme.palette.destructive,
            Self::Foreground => theme.palette.foreground,
        }
    }
}

/// Sharp progress track. Values, completion, and async work remain host-owned.
#[must_use = "Add this progress indicator to a Ui"]
pub struct Progress<'a> {
    value: Option<f64>,
    max: f64,
    label: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    id: Option<IdSalt>,
    size: ProgressSize,
    variant: ProgressVariant,
    width: Option<f32>,
    show_value: bool,
    animate: bool,
}

impl<'a> Progress<'a> {
    /// Determinate progress in 0..=100 units by default.
    pub fn new(value: f64) -> Self {
        Self {
            value: Some(value),
            max: loading::PROGRESS_MAX,
            label: None,
            name: None,
            id: None,
            size: ProgressSize::Medium,
            variant: ProgressVariant::Primary,
            width: None,
            show_value: false,
            animate: true,
        }
    }
    /// Unknown progress: omit a numeric value and show a traveling segment.
    pub fn indeterminate() -> Self {
        Self {
            value: None,
            ..Self::new(0.0)
        }
    }
    /// Maximum units; non-finite/non-positive maxima fall back to 100.
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }
    /// Visible label above the track; also used as its accessible name.
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Accessible name, overriding the label without adding visible text.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Stable identity, recommended for changing labels or inserted siblings.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = Some(IdSalt::new(id));
        self
    }
    /// Track height tier.
    pub fn size(mut self, size: ProgressSize) -> Self {
        self.size = size;
        self
    }
    /// Semantic fill.
    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Show rounded percentage, or "In progress" for unknown progress.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }
    /// Preferred width, constrained to the column.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(super::loading_motion::dimension(width));
        self
    }
    /// Pause indeterminate motion. Determinate progress always updates immediately.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    fn range(&self) -> (f64, Option<f64>) {
        let max = if self.max.is_finite() && self.max > 0.0 {
            self.max
        } else {
            loading::PROGRESS_MAX
        };
        let value = self.value.map(|value| {
            if value.is_nan() {
                0.0
            } else {
                value.clamp(0.0, max)
            }
        });
        (max, value)
    }

    fn header(&self, ui: &mut Ui, percent: Option<f64>) {
        if self.label.is_none() && !self.show_value {
            return;
        }
        let theme = Theme::from_context(ui.ctx());
        let value = self.show_value.then(|| {
            percent.map_or("In progress".to_string(), |percent| {
                format!("{percent:.0}%")
            })
        });
        let job = value.map(|value| {
            Text::new(value)
                .family(Family::Mono)
                .size(TextSize::Xs)
                .color(
                    ui.visuals()
                        .override_text_color
                        .map_or(TextColor::Muted, TextColor::Custom),
                )
                .layout_job(&theme, ui.available_width())
        });
        let value_width = job.as_ref().map_or(0.0, |job| {
            ui.fonts_mut(|fonts| fonts.layout_job(job.clone()).size().x)
        });
        let label_width = (ui.available_width() - value_width - space::SM).max(0.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::SM;
            ui.allocate_ui_with_layout(
                vec2(label_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_width(label_width);
                    if let Some(label) = &self.label {
                        ui.add(
                            Text::new(label.as_ref())
                                .size(TextSize::Xs)
                                .weight(Weight::Bold)
                                .tracking(Tracking::Wider)
                                .uppercase(true),
                        );
                    }
                },
            );
            if let Some(job) = job {
                ui.add(egui::Label::new(job).wrap());
            }
        });
        ui.add_space(space::SM);
    }

    fn show(self, ui: &mut Ui) -> Response {
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width())
            .max(0.0);
        ui.set_width(width);
        let (max, value) = self.range();
        self.header(ui, value.map(|value| 100.0 * (value / max)));
        let (rect, response) =
            ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());
        let name = self
            .name
            .as_deref()
            .or(self.label.as_deref())
            .unwrap_or(if value.is_some() {
                "Progress"
            } else {
                "Loading"
            });
        response
            .widget_info(|| WidgetInfo::labeled(Role::ProgressIndicator, ui.is_enabled(), name));
        response.ctx.accesskit_node_builder(response.id, |node| {
            if let Some(value) = value {
                node.set_numeric_value(value);
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(max);
                node.set_value(format!("{:.0}% complete", 100.0 * (value / max)));
            } else {
                node.set_busy();
                node.set_value("In progress");
            }
        });
        paint::track(
            ui,
            rect,
            value.map(|value| (value / max) as f32),
            self.variant,
            self.animate,
        );
        response
    }
}

impl Widget for Progress<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let builder = self.id.map_or_else(UiBuilder::new, |id| {
            UiBuilder::new().scope_id(ui.make_persistent_id(("dutchy-progress", id)))
        });
        ui.scope_builder(builder, |ui| self.show(ui)).inner
    }
}
