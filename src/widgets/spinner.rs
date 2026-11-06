use crate::{
    Theme,
    tokens::{border, motion},
};
use egui::accesskit::Role;
use egui::{Color32, Pos2, Response, Sense, Stroke, Ui, Widget, WidgetInfo, vec2};

/// Square-spinner side length before rotation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerSize {
    /// 12-point square.
    Small,
    /// 16-point square.
    #[default]
    Medium,
    /// 20-point square.
    Large,
}

impl SpinnerSize {
    /// Unrotated side length.
    pub const fn side(self) -> f32 {
        match self {
            Self::Small => 12.0,
            Self::Medium => 16.0,
            Self::Large => 20.0,
        }
    }

    /// Space reserved for the fully rotated square and stroke.
    pub fn extent(self) -> f32 {
        self.side() * std::f32::consts::SQRT_2 + border::STANDARD
    }
}

/// Three-sided rotating square. Repaints only while visible and animated.
#[derive(Clone, Copy, Debug, Default)]
#[must_use = "Add this spinner to a Ui"]
pub struct Spinner {
    size: SpinnerSize,
    color: Option<Color32>,
    paused: bool,
}

impl Spinner {
    /// Medium spinner inheriting the current UI text color.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the square side length.
    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }
    /// Set an explicit color.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
    /// Disable rotation, independently of the theme's reduced-motion preference.
    pub fn animate(mut self, animate: bool) -> Self {
        self.paused = !animate;
        self
    }
}

impl Widget for Spinner {
    fn ui(self, ui: &mut Ui) -> Response {
        let extent = self.size.extent();
        let (rect, response) = ui.allocate_exact_size(vec2(extent, extent), Sense::hover());
        response.widget_info(|| {
            WidgetInfo::labeled(Role::ProgressIndicator, ui.is_enabled(), "Loading")
        });
        response
            .ctx
            .accesskit_node_builder(response.id, |node| node.set_busy());
        let theme = Theme::from_context(ui.ctx());
        paint_spinner(
            ui,
            rect.center(),
            self.size,
            self.color.unwrap_or(ui.visuals().text_color()),
            !self.paused && !theme.reduced_motion,
        );
        response
    }
}

pub(super) fn paint_spinner(
    ui: &Ui,
    center: Pos2,
    size: SpinnerSize,
    color: Color32,
    animate: bool,
) {
    let visible = egui::Rect::from_center_size(center, vec2(size.extent(), size.extent()));
    if !ui.is_rect_visible(visible) {
        return;
    }
    let angle = if animate {
        ui.ctx().request_repaint_after(motion::FRAME_INTERVAL);
        ui.input(|input| (input.time.rem_euclid(motion::SPIN) / motion::SPIN) as f32)
            * std::f32::consts::TAU
    } else {
        0.0
    };
    let rotation = egui::emath::Rot2::from_angle(angle);
    let half = size.side() / 2.0;
    let points = [
        vec2(half, -half),
        vec2(-half, -half),
        vec2(-half, half),
        vec2(half, half),
    ]
    .map(|point| center + rotation * point)
    .to_vec();
    ui.painter().add(egui::Shape::line(
        points,
        Stroke::new(border::STANDARD, color),
    ));
}
