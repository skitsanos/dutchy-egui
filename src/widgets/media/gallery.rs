use super::{Image, ImageAspect, ImageResponse};
use crate::{ColumnWidth, EmptyState, ResponsiveGrid, Spacing, Theme, tokens::border};
use egui::{IdSalt, ImageSource, Response, Ui, UiBuilder};
use std::borrow::Cow;

/// One stable gallery item, shared with the lightbox. Keys must remain immutable.
#[derive(Clone)]
#[non_exhaustive]
pub struct GalleryItem<'a, K> {
    /// Host key, unique within this gallery.
    pub key: K,
    /// Texture, URI, or encoded bytes accepted by native egui loaders.
    pub source: ImageSource<'a>,
    /// Meaningful alternative text, also used to name expansion controls.
    pub alt: Cow<'a, str>,
    /// Optional caption.
    pub caption: Option<Cow<'a, str>>,
    /// Optional source/credit.
    pub caption_source: Option<Cow<'a, str>>,
    /// Whether opening/navigation to this item is enabled.
    pub enabled: bool,
}
impl<'a, K> GalleryItem<'a, K> {
    /// Construct an enabled item. The host retains texture handles/loaders.
    pub fn new(key: K, source: impl Into<ImageSource<'a>>, alt: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            source: source.into(),
            alt: alt.into(),
            caption: None,
            caption_source: None,
            enabled: true,
        }
    }
    /// Supporting caption, shown in the lightbox and optionally in thumbnails.
    pub fn caption(mut self, caption: impl Into<Cow<'a, str>>) -> Self {
        self.caption = Some(caption.into());
        self
    }
    /// Source/credit text.
    pub fn caption_source(mut self, source: impl Into<Cow<'a, str>>) -> Self {
        self.caption_source = Some(source.into());
        self
    }
    /// Disable opening/navigation without enabling disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub(super) fn image(&self) -> Image<'_> {
        let mut image =
            Image::new("item", self.source.clone(), self.alt.as_ref()).enabled(self.enabled);
        if let Some(caption) = &self.caption {
            image = image.caption(caption.as_ref());
        }
        if let Some(source) = &self.caption_source {
            image = image.caption_source(source.as_ref());
        }
        image
    }
}
/// Gallery geometry, thumbnails and a host-owned expansion request.
#[non_exhaustive]
pub struct GalleryResponse<K> {
    /// Named gallery region.
    pub response: Response,
    /// Key requested for viewing; no selection or lightbox is owned internally.
    pub open_requested: Option<K>,
    /// Thumbnails in supplied order, retaining their independent native responses.
    pub images: Vec<(K, ImageResponse)>,
}
/// Responsive, keyed thumbnail grid. All supplied items are laid out each pass.
#[must_use = "Call show to render the gallery"]
pub struct Gallery<'a, K> {
    id: IdSalt,
    items: &'a [GalleryItem<'a, K>],
    name: Cow<'a, str>,
    columns: usize,
    minimum: ColumnWidth,
    gap: Spacing,
    aspect: ImageAspect,
    captions: bool,
    enabled: bool,
    bordered: bool,
}
impl<'a, K: egui::AsIdSalt + Clone> Gallery<'a, K> {
    /// Joined, bordered square thumbnails; at most three compact-width columns.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [GalleryItem<'a, K>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            name: "Image gallery".into(),
            columns: 3,
            minimum: ColumnWidth::Compact,
            gap: Spacing::Micro,
            aspect: ImageAspect::Square,
            captions: false,
            enabled: true,
            bordered: true,
        }
    }
    /// Gallery region name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Maximum responsive columns; zero is treated as one.
    pub fn columns(mut self, count: usize) -> Self {
        self.columns = count.max(1);
        self
    }
    /// Minimum desired tile width; a single column may be narrower.
    pub fn min_column_width(mut self, width: ColumnWidth) -> Self {
        self.minimum = width;
        self
    }
    /// Token gap between tiles.
    pub fn gap(mut self, gap: Spacing) -> Self {
        self.gap = gap;
        self
    }
    /// Thumbnail aspect; the lightbox still contains the complete source image.
    pub fn aspect(mut self, aspect: ImageAspect) -> Self {
        self.aspect = aspect;
        self
    }
    /// Include each thumbnail's caption/source; defaults to false.
    pub fn captions(mut self, captions: bool) -> Self {
        self.captions = captions;
        self
    }
    /// Disable all opening controls.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Show/remove the outer two-point border.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
    /// Return expansion requests; the host supplies Lightbox selection and focus.
    pub fn show(self, ui: &mut Ui) -> GalleryResponse<K> {
        let root = ui.make_persistent_id(("dutchy-gallery", self.id));
        let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            if self.items.is_empty() {
                EmptyState::new("empty", "No images")
                    .description("Add images to build this gallery.")
                    .show(ui);
                return Vec::new();
            }
            let p = Theme::from_context(ui.ctx()).palette;
            // Joined tiles share one border-colored surface that shows through
            // the micro gap; spaced tiles sit on the page and border themselves.
            let joined = self.gap == Spacing::Micro;
            let frame = if joined {
                let width = if self.bordered {
                    border::STANDARD as i8
                } else {
                    0
                };
                egui::Frame::NONE
                    .fill(p.border)
                    .inner_margin(egui::Margin::same(width))
            } else {
                egui::Frame::NONE
            };
            frame
                .show(ui, |ui| {
                    let anchor = ui.max_rect().min;
                    ResponsiveGrid::new("images")
                        .columns(self.columns)
                        .min_column_width(self.minimum)
                        .gap(self.gap)
                        .show(
                            ui,
                            self.items,
                            |item| &item.key,
                            |ui, item| {
                                egui::Frame::NONE
                                    .fill(p.background)
                                    .show(ui, |ui| {
                                        let mut image = item
                                            .image()
                                            .aspect(self.aspect)
                                            .expandable(true)
                                            .bordered(!joined && self.bordered);
                                        image.expand_size =
                                            crate::tokens::image::GALLERY_EXPAND_SIZE;
                                        image.expand_inset =
                                            crate::tokens::image::GALLERY_EXPAND_INSET;
                                        image.expand_opacity =
                                            crate::tokens::image::GALLERY_EXPAND_OPACITY;
                                        image.reveal_anchor = Some(anchor);
                                        if !self.captions {
                                            image.caption = None;
                                            image.caption_source = None;
                                        }
                                        (item.key.clone(), image.show(ui))
                                    })
                                    .inner
                            },
                        )
                        .inner
                })
                .inner
        });
        result.response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::accesskit::Role::Group,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        let open_requested = result
            .inner
            .iter()
            .find(|(_, image)| image.expand_requested)
            .map(|(key, _)| key.clone());
        GalleryResponse {
            response: result.response,
            open_requested,
            images: result.inner,
        }
    }
}
