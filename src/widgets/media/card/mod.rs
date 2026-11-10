mod paint;
mod render;

use super::{ImageAspect, ImageFit, ImageResponse};
use egui::{IdSalt, ImageSource, Response, Ui};
use std::borrow::Cow;

/// Geometry, loader status and independent application-owned action requests.
#[non_exhaustive]
pub struct MediaCardResponse {
    /// Named noninteractive card group.
    pub response: Response,
    /// Existing image pipeline, including status and optional retry request.
    pub image: ImageResponse,
    /// Native primary preview button over the image.
    pub preview: Response,
    /// Native metadata button with category, heading and description.
    pub details: Response,
    /// Optional native saved toggle.
    pub save: Option<Response>,
    /// Preview or metadata requested opening. The host performs that action.
    pub open_requested: bool,
    /// Requested saved state. Ignoring it leaves the control unchanged.
    pub save_requested: Option<bool>,
}

/// Sharp picture card with independent preview, metadata, saved and retry targets.
///
/// Application state stays in the host. Texture/URI/byte sources use the same
/// loader pipeline as Image; no loader, network service or persistence is added.
#[must_use = "Call show to render the media card"]
pub struct MediaCard<'a> {
    id: IdSalt,
    source: ImageSource<'a>,
    alt: Cow<'a, str>,
    title: Cow<'a, str>,
    category: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    action: Cow<'a, str>,
    saved: Option<bool>,
    aspect: ImageAspect,
    fit: ImageFit,
    width: Option<f32>,
    height: Option<f32>,
    enabled: bool,
    retryable: bool,
}

impl<'a> MediaCard<'a> {
    /// Stable identity, source, meaningful image alternative and visible title.
    pub fn new(
        id: impl egui::AsIdSalt,
        source: impl Into<ImageSource<'a>>,
        alt: impl Into<Cow<'a, str>>,
        title: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            source: source.into(),
            alt: alt.into(),
            title: title.into(),
            category: None,
            description: None,
            action: "View artwork".into(),
            saved: None,
            aspect: ImageAspect::Square,
            fit: ImageFit::Cover,
            width: None,
            height: None,
            enabled: true,
            retryable: false,
        }
    }
    /// Small uppercase category above the title.
    pub fn category(mut self, category: impl Into<Cow<'a, str>>) -> Self {
        self.category = Some(category.into());
        self
    }
    /// Supporting text, wrapped and truncated after three rows.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Preview overlay text and accessible action prefix; defaults to View artwork.
    pub fn action_label(mut self, action: impl Into<Cow<'a, str>>) -> Self {
        self.action = action.into();
        self
    }
    /// Show the optional saved toggle bound to this host-supplied state.
    pub fn saved(mut self, saved: bool) -> Self {
        self.saved = Some(saved);
        self
    }
    /// Image box aspect; square by default.
    pub fn aspect(mut self, aspect: ImageAspect) -> Self {
        self.aspect = aspect;
        self
    }
    /// Contain or cover without distorting the image.
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
    /// Preferred outer width, constrained to available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = super::geometry::dimension(width);
        self
    }
    /// Preferred image height, overriding the aspect-derived value.
    pub fn image_height(mut self, height: f32) -> Self {
        self.height = super::geometry::dimension(height);
        self
    }
    /// Offer a separate retry action on failure. The host performs retry policy.
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
    /// Disable all actions without enabling a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render the card and return requests without mutating host state.
    pub fn show(self, ui: &mut Ui) -> MediaCardResponse {
        render::show(self, ui)
    }
}
