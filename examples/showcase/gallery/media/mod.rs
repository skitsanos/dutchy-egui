pub mod art;
mod pending;
#[cfg(feature = "url-images")]
mod remote;
use super::section;
use dutchy_egui::{
    Button, ButtonVariant, Checkbox, ColumnWidth, ControlSize, Gallery, GalleryItem, Image,
    ImageAspect, ImageFit, Lightbox, OverlayOptions, ResponsiveGrid, Spacing, Text, TextColor,
};
use egui::{Context, Id, TextureHandle, Ui, load::SizedTexture};

#[derive(Default)]
pub struct MediaState {
    textures: Vec<TextureHandle>,
    pub selected: Option<u8>,
    opener: Option<Id>,
    pub wrap: bool,
    pub reversed: bool,
    pub disabled: bool,
    pub loaded: bool,
    pub retries: usize,
    #[cfg(feature = "url-images")]
    pub online: bool,
}
impl MediaState {
    fn ensure(&mut self, ctx: &Context) {
        if self.textures.is_empty() {
            self.textures = art::textures(ctx);
            ctx.add_texture_loader(std::sync::Arc::new(pending::Pending));
        }
    }
    fn items(&self) -> Vec<GalleryItem<'static, u8>> {
        let mut items = self.sources();
        if self.reversed {
            items.reverse();
        }
        items
    }
    fn sources(&self) -> Vec<GalleryItem<'static, u8>> {
        #[cfg(feature = "url-images")]
        if self.online {
            return remote::items();
        }
        self.textures
            .iter()
            .enumerate()
            .map(|(index, texture)| {
                GalleryItem::new(
                    index as u8,
                    SizedTexture::from_handle(texture),
                    art::NAMES[index],
                )
                .caption(format!(
                    "{} — an original Dutchy raster illustration.",
                    art::NAMES[index]
                ))
                .caption_source("Dutchy / original artwork")
                .enabled(index != 3)
            })
            .collect()
    }
    fn source_controls(&mut self, ui: &mut Ui) {
        #[cfg(feature = "url-images")]
        remote::controls(ui, &mut self.online);
        #[cfg(not(feature = "url-images"))]
        ui.add(
            Text::new("For online photos, run the showcase with --features url-images.")
                .color(TextColor::Muted),
        );
    }
    fn retry(
        &mut self,
        ctx: &Context,
        item: &GalleryItem<'_, u8>,
        response: &dutchy_egui::ImageResponse,
    ) {
        if response.retry_requested {
            self.retries += 1;
            if let Some(uri) = item.source.uri() {
                ctx.forget_image(uri);
            }
        }
    }
    fn opened(&mut self, key: u8, response: &dutchy_egui::ImageResponse) {
        if response.expand_requested {
            self.selected = Some(key);
            self.opener = response.expand.as_ref().map(|response| response.id);
        }
    }
    fn gallery(&mut self, ui: &mut Ui, id: &str, captions: bool) {
        let items = self.items();
        let result = Gallery::new(id, &items)
            .accessible_name(if captions {
                "Captioned artwork"
            } else {
                "Joined artwork"
            })
            .gap(if captions {
                Spacing::Xxl
            } else {
                Spacing::Micro
            })
            .captions(captions)
            .bordered(!captions)
            .enabled(!self.disabled)
            .show(ui);
        if let Some(key) = result.open_requested {
            self.selected = Some(key);
            self.opener = result
                .images
                .iter()
                .find(|(id, _)| *id == key)
                .and_then(|(_, image)| image.expand.as_ref().map(|response| response.id));
        }
        for (key, response) in &result.images {
            if let Some(item) = items.iter().find(|item| item.key == *key) {
                self.retry(ui.ctx(), item, response);
            }
        }
    }
    pub fn images(&mut self, ui: &mut Ui) {
        self.ensure(ui.ctx());
        ui.add(Text::heading("Images. With intention."));
        ui.add(Text::new(
            "Sharp frames, reserved aspect ratios, clear captions and native loader outcomes.",
        ));
        self.source_controls(ui);
        section(ui, "01", "Intrinsic image / caption and source", |ui| {
            let items = self.items();
            let item = items.iter().find(|item| item.key == 0).unwrap();
            let response = Image::new("feature", item.source.clone(), item.alt.as_ref())
                .width(640.0)
                .caption(item.caption.as_deref().unwrap_or_default())
                .caption_source(item.caption_source.as_deref().unwrap_or_default())
                .expandable(true)
                .retryable(true)
                .show(ui);
            self.opened(item.key, &response);
            self.retry(ui.ctx(), item, &response);
        });
        section(ui, "02", "Cover crops / contain placement", |ui| {
            let items = self.items();
            let portrait = items.iter().find(|item| item.key == 1).unwrap();
            let variants = [
                ("Square crop", ImageAspect::Square, ImageFit::Cover),
                ("Video crop", ImageAspect::Video, ImageFit::Cover),
                ("4:3 contained", ImageAspect::Standard, ImageFit::Contain),
                ("16:10 crop", ImageAspect::Widescreen, ImageFit::Cover),
            ];
            ResponsiveGrid::new("aspect-grid")
                .columns(2)
                .min_column_width(ColumnWidth::Standard)
                .gap(Spacing::Xxl)
                .show(
                    ui,
                    &variants,
                    |variant| variant.0,
                    |ui, (label, aspect, fit)| {
                        let response = Image::new("aspect", portrait.source.clone(), *label)
                            .aspect(*aspect)
                            .fit(*fit)
                            .caption(*label)
                            .retryable(true)
                            .show(ui);
                        self.retry(ui.ctx(), portrait, &response);
                    },
                );
        });
        section(ui, "03", "Host loading / explicit retry", |ui| {
            ui.add(Checkbox::new(
                "loaded",
                &mut self.loaded,
                "Show loaded image",
            ));
            ResponsiveGrid::new("loader-grid")
                .columns(2)
                .min_column_width(ColumnWidth::Standard)
                .gap(Spacing::Xxl)
                .show(
                    ui,
                    &[0, 1],
                    |index| *index,
                    |ui, index| {
                        if *index == 0 {
                            let source = if self.loaded {
                                egui::ImageSource::Texture(SizedTexture::from_handle(
                                    &self.textures[2],
                                ))
                            } else {
                                egui::ImageSource::Uri("showcase://pending".into())
                            };
                            Image::new("pending", source, "Host-controlled loading artwork")
                                .aspect(ImageAspect::Video)
                                .caption("Toggle the host control to replace the pending source.")
                                .show(ui);
                        } else {
                            let response = Image::new(
                                "missing",
                                "bytes://missing-showcase-image.png",
                                "Unavailable artwork",
                            )
                            .aspect(ImageAspect::Video)
                            .retryable(true)
                            .caption("A missing source stays explicit. Retry returns to the host.")
                            .show(ui);
                            if response.retry_requested {
                                self.retries += 1;
                                ui.ctx().forget_image("bytes://missing-showcase-image.png");
                            }
                        }
                    },
                );
            ui.add(
                Text::label(format!("Host retry requests: {}", self.retries))
                    .color(TextColor::Muted),
            );
        });
    }
    pub fn galleries(&mut self, ui: &mut Ui) {
        self.ensure(ui.ctx());
        ui.add(Text::heading("A collection. Clearly framed."));
        ui.add(Text::new("Stable item keys preserve controls through reordering and responsive layout. The fourth image is disabled."));
        self.source_controls(ui);
        ui.horizontal_wrapped(|ui| {
            ui.add(Checkbox::new(
                "reverse",
                &mut self.reversed,
                "Reverse artwork order",
            ));
            ui.add(Checkbox::new(
                "disable-gallery",
                &mut self.disabled,
                "Disable galleries",
            ));
        });
        section(ui, "01", "Joined thumbnails / host-owned opening", |ui| {
            self.gallery(ui, "joined", false)
        });
        section(ui, "02", "Spaced tiles / captions and source", |ui| {
            self.gallery(ui, "captioned", true)
        });
    }
    pub fn lightboxes(&mut self, ui: &mut Ui) {
        self.ensure(ui.ctx());
        ui.add(Text::heading("See the whole image."));
        ui.add(Text::new("A dark, constrained viewer. Native modality blocks the workspace and returns focus to its opener."));
        self.source_controls(ui);
        section(
            ui,
            "01",
            "Keyboard / native controls / host selection",
            |ui| {
                ui.add(Checkbox::new(
                    "wrap",
                    &mut self.wrap,
                    "Wrap image navigation",
                ));
                ui.horizontal_wrapped(|ui| {
                    for (key, label) in [
                        (0, "Open first artwork"),
                        (99, "Open unavailable selection"),
                    ] {
                        let response = ui.add(
                            Button::new(label)
                                .variant(ButtonVariant::Outline)
                                .size(ControlSize::Small),
                        );
                        if response.clicked() {
                            self.selected = Some(key);
                            self.opener = Some(response.id);
                        }
                    }
                });
                ui.add(Text::new("Left/Right navigate enabled images; Home/End reach collection endpoints. Escape, Close or the backdrop dismiss. The image surface itself does not dismiss."));
                ui.add(
                    Text::label(format!("Host selected key: {:?}", self.selected))
                        .color(TextColor::Muted),
                );
            },
        );
        section(ui, "02", "Expand a thumbnail", |ui| {
            self.gallery(ui, "viewer-gallery", false)
        });
    }
    pub fn render(&mut self, ctx: &Context) {
        let items = self.items();
        let response = Lightbox::new("artwork-viewer", &items, &mut self.selected)
            .wrap_navigation(self.wrap)
            .options(OverlayOptions {
                return_focus: self.opener,
                ..Default::default()
            })
            .show(ctx);
        if let Some(response) = response
            && let Some(image) = &response.image
            && let Some(item) = items
                .iter()
                .find(|item| Some(&item.key) == response.key.as_ref())
        {
            self.retry(ctx, item, image);
        }
    }
}
