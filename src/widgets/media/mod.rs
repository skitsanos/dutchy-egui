mod caption;
mod card;
mod control;
mod gallery;
mod geometry;
mod lightbox;
mod loading;
mod render;
mod viewer;

use crate::tokens::Palette;
pub use card::{MediaCard, MediaCardResponse};
use egui::{IdSalt, ImageSource, Response, Ui};
pub use gallery::{Gallery, GalleryItem, GalleryResponse};
pub use lightbox::{Lightbox, LightboxResponse};
use std::borrow::Cow;

/// Aspect of the reserved image box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageAspect {
    /// Intrinsic aspect, with a 16:9 fallback while dimensions are unknown.
    #[default]
    Original,
    /// 1:1.
    Square,
    /// 16:9.
    Video,
    /// 4:3.
    Standard,
    /// 16:10.
    Widescreen,
}
/// Image placement without distorting its aspect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// Show the whole image with flat letterboxing when necessary.
    #[default]
    Contain,
    /// Fill the box, cropping equally at opposite edges.
    Cover,
}
/// Current native loader outcome, with diagnostic errors returned to the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageStatus {
    /// Texture is available.
    Ready,
    /// A host-installed loader is working.
    Loading,
    /// Loading/decoding failed; the visible UI uses concise guidance.
    Failed(String),
}
/// Image geometry and independent host action requests.
pub struct ImageResponse {
    /// Figure including its caption.
    pub response: Response,
    /// Named image box, including placeholder/error states.
    pub image: Response,
    /// Optional native expansion control.
    pub expand: Option<Response>,
    /// Optional native retry control.
    pub retry: Option<Response>,
    /// Open a host-owned lightbox.
    pub expand_requested: bool,
    /// Host should retry its loader, e.g. Context::forget_image for a URI.
    pub retry_requested: bool,
    /// Native loading result.
    pub status: ImageStatus,
}
/// Sharp, accessible image with optional caption and independent expand control.
/// Textures work directly; URI/encoded-byte sources require host-installed egui loaders.
#[must_use = "Call show to render the image"]
pub struct Image<'a> {
    id: IdSalt,
    source: ImageSource<'a>,
    alt: Cow<'a, str>,
    caption: Option<Cow<'a, str>>,
    caption_source: Option<Cow<'a, str>>,
    aspect: ImageAspect,
    fit: ImageFit,
    width: Option<f32>,
    height: Option<f32>,
    bordered: bool,
    expandable: bool,
    expand_size: f32,
    expand_inset: f32,
    expand_opacity: f32,
    retryable: bool,
    enabled: bool,
    surface: Option<Palette>,
    reveal_anchor: Option<egui::Pos2>,
    action: Option<Cow<'a, str>>,
}
impl<'a> Image<'a> {
    /// Bind a stable ID, native source, and meaningful alternative text.
    pub fn new(
        id: impl egui::AsIdSalt,
        source: impl Into<ImageSource<'a>>,
        alt: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            source: source.into(),
            alt: alt.into(),
            caption: None,
            caption_source: None,
            aspect: ImageAspect::Original,
            fit: ImageFit::Contain,
            width: None,
            height: None,
            bordered: true,
            expandable: false,
            expand_size: crate::tokens::image::EXPAND_SIZE,
            expand_inset: crate::tokens::image::EXPAND_INSET,
            expand_opacity: crate::tokens::image::EXPAND_OPACITY,
            retryable: false,
            enabled: true,
            surface: None,
            reveal_anchor: None,
            action: None,
        }
    }
    /// Reserve a named aspect; fixed aspects default to a cover crop.
    pub fn aspect(mut self, aspect: ImageAspect) -> Self {
        self.aspect = aspect;
        self.fit = if aspect == ImageAspect::Original {
            ImageFit::Contain
        } else {
            ImageFit::Cover
        };
        self
    }
    /// Override contain/cover placement, without distortion.
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
    /// Preferred width, constrained to available width. Invalid values use defaults.
    pub fn width(mut self, width: f32) -> Self {
        self.width = geometry::dimension(width);
        self
    }
    /// Preferred box height, overriding the aspect-derived height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = geometry::dimension(height);
        self
    }
    /// Supporting caption below the image.
    pub fn caption(mut self, caption: impl Into<Cow<'a, str>>) -> Self {
        self.caption = Some(caption.into());
        self
    }
    /// Uppercase monospaced credit/source, including without a caption.
    pub fn caption_source(mut self, source: impl Into<Cow<'a, str>>) -> Self {
        self.caption_source = Some(source.into());
        self
    }
    /// Show/remove the two-point image outline.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
    /// Show a named expansion button, including during loading or failure.
    pub fn expandable(mut self, expandable: bool) -> Self {
        self.expandable = expandable;
        self
    }
    /// Offer a host-owned retry request on loader failure.
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
    /// Disable controls without enabling a disabled parent UI.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render using the native loader pipeline; no I/O or decoding is installed here.
    pub fn show(self, ui: &mut Ui) -> ImageResponse {
        render::show(self, ui)
    }
}
