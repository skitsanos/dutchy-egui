//! Images, fitting, gallery controls and fullscreen viewer surfaces.
#![cfg(feature = "bundled-fonts")]
#[path = "../examples/showcase/gallery/media/art.rs"]
mod art;
mod common;
#[path = "../examples/showcase/gallery/media/pending.rs"]
mod pending;
use dutchy_egui::{
    Accent, ColumnWidth, Gallery, GalleryItem, Image, ImageAspect, ImageFit, Lightbox, Mode,
    ResponsiveGrid, Spacing, Text, Theme,
};
use egui::{TextureHandle, accesskit::Role, load::SizedTexture};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

#[derive(Default)]
struct State {
    textures: Vec<TextureHandle>,
    selected: Option<u8>,
    long_caption: bool,
    rects: Vec<egui::Rect>,
}
fn items(state: &State) -> Vec<GalleryItem<'static, u8>> {
    state
        .textures
        .iter()
        .enumerate()
        .map(|(index, texture)| {
            GalleryItem::new(
                index as u8,
                SizedTexture::from_handle(texture),
                art::NAMES[index],
            )
            .caption(if state.long_caption {
                "Long caption with enough detail to wrap over several lines. ".repeat(16)
            } else {
                format!("{} — original Dutchy artwork.", art::NAMES[index])
            })
            .caption_source("Dutchy / original")
            .enabled(index != 3)
        })
        .collect()
}
fn matrix(ui: &mut egui::Ui, state: &mut State) {
    if state.textures.is_empty() {
        state.textures = art::textures(ui.ctx());
        ui.ctx()
            .add_texture_loader(std::sync::Arc::new(pending::Pending));
    }
    ui.add(Text::heading("Image. Form. Story."));
    let variants = [
        (
            "Natural canal houses",
            0,
            ImageAspect::Original,
            ImageFit::Contain,
        ),
        ("Square cover", 1, ImageAspect::Square, ImageFit::Cover),
        ("Square contain", 1, ImageAspect::Square, ImageFit::Contain),
        ("Wide bridge", 3, ImageAspect::Video, ImageFit::Cover),
    ];
    ResponsiveGrid::new("matrix")
        .columns(3)
        .min_column_width(ColumnWidth::Compact)
        .gap(Spacing::Xxl)
        .show(
            ui,
            &variants,
            |variant| variant.0,
            |ui, (label, index, aspect, fit)| {
                Image::new(
                    "picture",
                    SizedTexture::from_handle(&state.textures[*index]),
                    *label,
                )
                .aspect(*aspect)
                .fit(*fit)
                .caption(*label)
                .caption_source("Original")
                .expandable(true)
                .show(ui)
            },
        );
    ui.add_space(24.0);
    ResponsiveGrid::new("states")
        .columns(2)
        .min_column_width(ColumnWidth::Compact)
        .gap(Spacing::Xxl)
        .show(
            ui,
            &[false, true],
            |error| *error,
            |ui, error| {
                Image::new(
                    "state",
                    if *error {
                        "bytes://not-installed.png"
                    } else {
                        "showcase://pending"
                    },
                    if *error {
                        "Missing artwork"
                    } else {
                        "Pending artwork"
                    },
                )
                .aspect(ImageAspect::Video)
                .expandable(true)
                .retryable(true)
                .caption(if *error {
                    "The host decides how to retry."
                } else {
                    "A named, reserved loading surface."
                })
                .show(ui)
            },
        );
    ui.add_space(24.0);
    let items = items(state);
    Gallery::new("collection", &items)
        .accessible_name("Artwork collection")
        .show(ui);
}
fn harness(theme: Theme, scale: f32, size: [f32; 2], viewer: bool) -> Harness<'static, State> {
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(if theme.mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui_state(
            move |ui, state: &mut State| {
                if viewer {
                    if state.textures.is_empty() {
                        state.textures = art::textures(ui.ctx());
                        state.selected = Some(0);
                    }
                    ui.label("Workspace behind the viewer");
                    let items = items(state);
                    Lightbox::new("matrix-viewer", &items, &mut state.selected).show(ui.ctx());
                } else {
                    matrix(ui, state);
                }
            },
            State::default(),
        )
}
#[test]
fn image_gallery_and_viewer_surfaces_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = harness(theme, scale, [900.0, 1800.0], false);
        h.run_steps(5);
        h.get_by_label("Expand Square cover").focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(format!("images-{name}"));
        results.extend_harness(&mut h);
        let mut viewer = harness(theme, scale, [1024.0, 720.0], true);
        viewer.run_steps(5);
        viewer.check_accessibility();
        viewer.snapshot(format!("lightbox-{name}"));
        results.extend_harness(&mut viewer);
    }
    results.unwrap();
}
#[test]
fn expand_control_has_flat_hover_feedback_and_a_stable_target() {
    let mut h = harness(
        Theme::default().with_reduced_motion(true),
        1.0,
        [900.0, 1800.0],
        false,
    );
    h.run_steps(4);
    let before = h.render().unwrap();
    let rect = h.get_by_label("Expand Square cover").rect();
    h.get_by_label("Expand Square cover").hover();
    h.run_steps(2);
    assert_eq!(h.get_by_label("Expand Square cover").rect(), rect);
    assert_ne!(before, h.render().unwrap());
    h.snapshot("image-expand-hover");
}
#[test]
fn narrow_images_and_long_viewer_captions_remain_constrained() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    let mut h = harness(
        Theme::default().with_reduced_motion(true),
        1.0,
        [280.0, 3600.0],
        false,
    );
    h.run_steps(5);
    for image in h.query_all(egui_kittest::kittest::By::new().role(Role::Image)) {
        let r = image.rect();
        assert!(r.left() >= 8.0 && r.right() <= 272.5, "{r:?}");
    }
    h.render()
        .unwrap()
        .save("target/visual-proof/images-matrix-narrow.png")
        .unwrap();
    let mut viewer = harness(
        Theme::default().with_reduced_motion(true),
        1.0,
        [280.0, 600.0],
        true,
    );
    viewer.state_mut().long_caption = true;
    viewer.run_steps(5);
    let r = viewer
        .get_by_role_and_label(Role::Dialog, "Image viewer")
        .rect();
    assert!(
        r.left() >= 23.5 && r.right() <= 256.5 && r.bottom() <= 576.5,
        "{r:?}"
    );
    assert!(viewer.get_by_label("Close Image viewer").is_focused());
    viewer.check_accessibility();
    viewer
        .render()
        .unwrap()
        .save("target/visual-proof/lightbox-narrow.png")
        .unwrap();
    for _ in 0..3 {
        viewer.key_press(egui::Key::PageDown);
        viewer.run_steps(3);
    }
    assert!(viewer.get_by_label("Dutchy / original").rect().bottom() <= 576.5);
    assert!(viewer.get_by_label("Close Image viewer").is_focused());
    viewer
        .render()
        .unwrap()
        .save("target/visual-proof/lightbox-caption-scrolled.png")
        .unwrap();
}
#[test]
fn contain_letterboxes_and_cover_crops_actual_texture_pixels_without_distortion() {
    let mut texture = None;
    let mut h = Harness::builder()
        .with_size([500.0, 270.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(move |ui| {
            let texture = texture.get_or_insert_with(|| {
                let mut image = egui::ColorImage::filled([160, 80], egui::Color32::GREEN);
                for y in 0..80 {
                    for x in 0..40 {
                        image.pixels[y * 160 + x] = egui::Color32::RED;
                        image.pixels[y * 160 + x + 120] = egui::Color32::BLUE;
                    }
                }
                ui.ctx()
                    .load_texture("Crop proof", image, egui::TextureOptions::NEAREST)
            });
            ui.horizontal_top(|ui| {
                Image::new("cover", SizedTexture::from_handle(texture), "Cover pixels")
                    .width(200.0)
                    .height(200.0)
                    .fit(ImageFit::Cover)
                    .show(ui);
                Image::new(
                    "contain",
                    SizedTexture::from_handle(texture),
                    "Contain pixels",
                )
                .width(200.0)
                .height(200.0)
                .fit(ImageFit::Contain)
                .show(ui);
            });
        });
    h.run_steps(4);
    let image = h.render().unwrap();
    let cover = h.get_by_role_and_label(Role::Image, "Cover pixels").rect();
    let contain = h
        .get_by_role_and_label(Role::Image, "Contain pixels")
        .rect();
    let pixel = |x: f32, y: f32| image.get_pixel(x as u32, y as u32).0;
    assert_eq!(
        pixel(cover.left() + 20.0, cover.top() + 20.0),
        [0, 255, 0, 255]
    );
    assert_eq!(
        pixel(contain.left() + 20.0, contain.center().y),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel(contain.right() - 20.0, contain.center().y),
        [0, 0, 255, 255]
    );
    assert_ne!(
        pixel(contain.center().x, contain.top() + 20.0),
        [0, 255, 0, 255]
    );
}

/// Colors of the pixel midway between the first two tiles and of the page.
fn gallery_gap_pixels(spaced: bool) -> ([u8; 3], [u8; 3]) {
    let mut h = Harness::builder()
        .with_size([520.0, 360.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            move |ui, state: &mut State| {
                if state.textures.is_empty() {
                    state.textures = art::textures(ui.ctx());
                }
                // A painted strip above the gallery samples the page surface.
                ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 16.0),
                    egui::Sense::hover(),
                );
                let items = items(state);
                let mut gallery = Gallery::new("pair", &items[..2]).columns(2);
                if spaced {
                    gallery = gallery.gap(Spacing::Xxl).bordered(false);
                }
                state.rects = gallery
                    .show(ui)
                    .images
                    .into_iter()
                    .map(|(_, image)| image.image.rect)
                    .collect();
            },
            State::default(),
        );
    h.run_steps(3);
    let pixels = h.render().unwrap();
    let [a, b] = [h.state().rects[0], h.state().rects[1]];
    let gap = pixels.get_pixel(((a.right() + b.left()) / 2.0) as u32, a.center().y as u32);
    let page = pixels.get_pixel(a.center().x as u32, 12);
    ([gap[0], gap[1], gap[2]], [page[0], page[1], page[2]])
}
#[test]
fn spaced_gallery_leaves_the_page_background_between_tiles() {
    let (gap, page) = gallery_gap_pixels(true);
    assert_eq!(
        gap, page,
        "a spaced gallery paints no border band between tiles"
    );
}
#[test]
fn joined_gallery_fills_the_micro_gap_with_the_border_color() {
    let (gap, page) = gallery_gap_pixels(false);
    assert_ne!(
        gap, page,
        "the joined gallery keeps its border-colored hairline gap"
    );
}
