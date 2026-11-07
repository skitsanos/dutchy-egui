//! Native texture/URI/byte loading, presentation state, semantics and controls.
mod common;
#[path = "common/media.rs"]
mod media;
use dutchy_egui::{Image, ImageAspect, ImageFit, ImageStatus, Theme};
use egui::{Rect, accesskit::Role, load::SizedTexture};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use media::{Outcome, Shared};
use std::sync::{Arc, Mutex};

struct State {
    probe: Shared,
    status: Option<ImageStatus>,
    rect: Option<Rect>,
    expands: usize,
    retries: usize,
    disabled: bool,
    parent_disabled: bool,
    caption: bool,
    renamed: bool,
    apply_retry: bool,
    aspect: ImageAspect,
    fit: ImageFit,
    width: f32,
    height: Option<f32>,
    source: u8,
    offset: f32,
    hidden: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            probe: Arc::new(Mutex::new(media::Probe::default())),
            status: None,
            rect: None,
            expands: 0,
            retries: 0,
            disabled: false,
            parent_disabled: false,
            caption: true,
            renamed: false,
            apply_retry: false,
            aspect: ImageAspect::Original,
            fit: ImageFit::Contain,
            width: 300.0,
            height: None,
            source: 0,
            offset: 0.0,
            hidden: false,
        }
    }
}
fn harness() -> Harness<'static, State> {
    let state = State::default();
    let probe = state.probe.clone();
    Harness::builder()
        .with_size([600.0, 800.0])
        .with_context_setup(move |ctx| {
            common::install(ctx, Theme::default().with_reduced_motion(true));
            media::install(ctx, probe.clone());
        })
        .build_ui_state(
            |ui, state: &mut State| {
                if state.parent_disabled {
                    ui.disable();
                }
                egui::ScrollArea::vertical()
                    .max_height(700.0)
                    .vertical_scroll_offset(state.offset)
                    .show(ui, |ui| {
                        if state.hidden {
                            ui.add_space(1400.0);
                        }
                        let source = match state.source {
                            1 => egui::ImageSource::Bytes {
                                uri: "fixture://bytes.png".into(),
                                bytes: egui::load::Bytes::Static(b"fixture encoded bytes"),
                            },
                            2 => egui::ImageSource::Texture(SizedTexture::new(
                                egui::TextureId::Managed(999),
                                egui::Vec2::ZERO,
                            )),
                            3 => egui::ImageSource::Uri("bytes://no-loader.png".into()),
                            _ => egui::ImageSource::Uri("fixture://photo".into()),
                        };
                        let mut image = Image::new(
                            "picture",
                            source,
                            if state.renamed {
                                "Renamed artwork"
                            } else {
                                "Test artwork"
                            },
                        )
                        .aspect(state.aspect)
                        .fit(state.fit)
                        .width(state.width)
                        .expandable(true)
                        .retryable(true)
                        .enabled(!state.disabled);
                        if let Some(height) = state.height {
                            image = image.height(height);
                        }
                        if state.caption {
                            image = image
                                .caption("A caption that remains ordinary content.")
                                .caption_source("Fixture source");
                        }
                        let result = image.show(ui);
                        state.rect = Some(result.image.rect);
                        state.status = Some(result.status);
                        state.expands += usize::from(result.expand_requested);
                        state.retries += usize::from(result.retry_requested);
                        if state.apply_retry && result.retry_requested {
                            ui.ctx().forget_image("fixture://photo");
                        }
                    });
            },
            state,
        )
}
#[test]
fn intrinsic_image_has_alt_caption_source_and_an_independent_native_expansion_button() {
    let mut h = harness();
    h.run_steps(4);
    assert_eq!(h.state().status, Some(ImageStatus::Ready));
    let image = h.get_by_role_and_label(Role::Image, "Test artwork");
    assert_eq!(image.rect().size(), egui::vec2(300.0, 150.0));
    assert!(h.query_by_label("Fixture source").is_some());
    let caption = h.get_by_role(Role::FigureCaption).rect();
    assert!(
        (caption.top() - image.rect().bottom() - dutchy_egui::tokens::image::CAPTION_GAP).abs()
            < 0.6,
        "{caption:?}"
    );
    assert!(h.get_by_label("Fixture source").rect().right() <= image.rect().right() + 0.5);
    image.click();
    h.run_steps(2);
    assert_eq!(h.state().expands, 0);
    h.get_by_label("Expand Test artwork").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().expands, 1);
    h.check_accessibility();
}
#[test]
fn loading_failure_retry_and_ready_follow_host_policy_without_implicit_forgetting() {
    let mut h = harness();
    h.state().probe.lock().unwrap().outcome = Outcome::Pending;
    h.run_steps(3);
    assert_eq!(h.state().status, Some(ImageStatus::Loading));
    assert!(
        h.get_by_role_and_label(Role::Image, "Test artwork")
            .accesskit_node()
            .data()
            .is_busy()
    );
    assert!(
        !h.get_by_label("Expand Test artwork")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    assert!(h.query_by_label("Retry image Test artwork").is_none());
    h.state().probe.lock().unwrap().outcome = Outcome::Failed;
    h.run_steps(3);
    assert!(matches!(h.state().status, Some(ImageStatus::Failed(_))));
    assert!(
        !h.get_by_role_and_label(Role::Image, "Test artwork")
            .accesskit_node()
            .data()
            .is_busy()
    );
    h.get_by_label("Retry image Test artwork").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().retries, 1);
    assert_eq!(h.state().probe.lock().unwrap().forgets, 0);
    h.state_mut().apply_retry = true;
    h.get_by_label("Retry image Test artwork").click();
    h.run_steps(2);
    assert_eq!(h.state().probe.lock().unwrap().forgets, 1);
    h.state().probe.lock().unwrap().outcome = Outcome::Ready;
    h.run_steps(3);
    assert_eq!(h.state().status, Some(ImageStatus::Ready));
    assert!(h.query_by_label("Retry image Test artwork").is_none());
}
#[test]
fn disabled_images_and_disabled_parents_preserve_readable_images_without_actions() {
    let mut h = harness();
    h.state_mut().disabled = true;
    h.run_steps(3);
    h.get_by_label("Expand Test artwork").click();
    h.run_steps(2);
    assert_eq!(h.state().expands, 0);
    h.state_mut().disabled = false;
    h.state_mut().parent_disabled = true;
    h.run_steps(3);
    assert!(
        h.get_by_label("Expand Test artwork")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    assert!(
        h.query_by_role_and_label(Role::Image, "Test artwork")
            .is_some()
    );
    h.state().probe.lock().unwrap().outcome = Outcome::Failed;
    h.run_steps(3);
    h.get_by_label("Retry image Test artwork").click();
    h.run_steps(2);
    assert_eq!(h.state().retries, 0);
}
#[test]
fn encoded_bytes_use_native_inclusion_and_missing_loaders_or_invalid_textures_fail_explicitly() {
    let mut h = harness();
    h.state_mut().source = 1;
    h.run_steps(3);
    assert_eq!(h.state().status, Some(ImageStatus::Ready));
    let bytes = h.ctx.try_load_bytes("fixture://bytes.png").unwrap();
    assert!(
        matches!(bytes,egui::load::BytesPoll::Ready {bytes,..} if bytes.as_ref()==b"fixture encoded bytes")
    );
    h.state_mut().source = 2;
    h.run_steps(3);
    assert!(matches!(h.state().status, Some(ImageStatus::Failed(_))));
    h.state_mut().source = 3;
    h.run_steps(3);
    assert!(matches!(h.state().status, Some(ImageStatus::Failed(_))));
    assert_eq!(h.state().expands, 0);
}
#[test]
fn captions_renaming_and_geometry_changes_keep_control_identity_and_focus() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_label("Expand Test artwork").focus();
    h.run_steps(2);
    let id = h.get_by_label("Expand Test artwork").accesskit_node().id();
    h.state_mut().caption = false;
    h.state_mut().renamed = true;
    h.state_mut().aspect = ImageAspect::Square;
    h.state_mut().width = 220.0;
    h.run_steps(4);
    let button = h.get_by_label("Expand Renamed artwork");
    assert_eq!(button.accesskit_node().id(), id);
    assert!(button.is_focused());
    assert_eq!(h.state().rect.unwrap().size(), egui::vec2(220.0, 220.0));
    assert!(h.query_by_role(Role::FigureCaption).is_none());
}
#[test]
fn clipped_uri_images_defer_loading_until_revealed_and_sanitized_sizes_stay_finite() {
    let mut h = harness();
    h.state().probe.lock().unwrap().loads = 0;
    h.state_mut().hidden = true;
    h.run_steps(3);
    assert_eq!(h.state().probe.lock().unwrap().loads, 0);
    h.state_mut().offset = 1350.0;
    h.run_steps(4);
    assert!(h.state().probe.lock().unwrap().loads > 0);
    h.state_mut().hidden = false;
    h.state_mut().offset = 0.0;
    h.state_mut().width = f32::NAN;
    h.state_mut().height = Some(f32::INFINITY);
    h.run_steps(3);
    let rect = h.state().rect.unwrap();
    assert!(rect.is_finite());
    assert!(rect.width() <= 584.5);
}

#[test]
fn inverse_card_captions_inherit_text_colors_while_error_content_keeps_its_own_surface() {
    let probe = Arc::new(Mutex::new(media::Probe {
        outcome: Outcome::Failed,
        ..Default::default()
    }));
    let mut h = Harness::builder()
        .with_size([500.0, 400.0])
        .with_context_setup(move |ctx| {
            common::install(ctx, Theme::default().with_reduced_motion(true));
            media::install(ctx, probe.clone());
        })
        .build_ui(|ui| {
            dutchy_egui::Card::new()
                .variant(dutchy_egui::CardVariant::Dark)
                .show(ui, |ui| {
                    Image::new("card-image", "fixture://card", "Card image")
                        .width(320.0)
                        .height(150.0)
                        .caption("Inverse caption")
                        .caption_source("Archive")
                        .show(ui);
                });
        });
    h.run_steps(4);
    let pixels = h.render().unwrap();
    let any_pixel = |rect: Rect, bright: bool| {
        (rect.top().ceil() as u32..rect.bottom().floor() as u32).any(|y| {
            (rect.left().ceil() as u32..rect.right().floor() as u32).any(|x| {
                let rgb = pixels.get_pixel(x, y).0;
                if bright {
                    rgb[0] > 170 && rgb[1] > 170 && rgb[2] > 170
                } else {
                    rgb[0] < 150 && rgb[1] < 150 && rgb[2] < 150
                }
            })
        })
    };
    assert!(any_pixel(h.get_by_label("Inverse caption").rect(), true));
    assert!(any_pixel(h.get_by_label("Archive").rect(), true));
    assert!(any_pixel(
        h.get_by_role_and_label(Role::Image, "Card image")
            .rect()
            .shrink(8.0),
        false
    ));
}

fn galleys<'a>(shape: &'a egui::Shape, out: &mut Vec<&'a egui::Galley>) {
    match shape {
        egui::Shape::Text(text) => out.push(&text.galley),
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| galleys(shape, out)),
        _ => {}
    }
}
#[test]
fn caption_source_uses_wider_tracking() {
    let mut h = harness();
    h.state_mut().caption = true;
    h.run_steps(2);
    let mut painted = Vec::new();
    for clipped in &h.output().shapes {
        galleys(&clipped.shape, &mut painted);
    }
    let spacing = painted
        .iter()
        .find(|galley| galley.job.text.contains("FIXTURE SOURCE"))
        .map(|galley| galley.job.sections[0].format.extra_letter_spacing)
        .expect("the source credit is painted");
    let expected = dutchy_egui::Tracking::Wider.em() * 12.0;
    assert!(
        (spacing - expected).abs() < 1e-3,
        "credit tracking {spacing} should be {expected}"
    );
}

/// Fill alpha of the square control painted inside `area` with the given extent.
fn control_alpha(
    shapes: &[egui::epaint::ClippedShape],
    area: egui::Rect,
    extent: f32,
) -> Option<u8> {
    shapes.iter().find_map(|clipped| match &clipped.shape {
        egui::Shape::Rect(shape)
            if area.contains_rect(shape.rect)
                && (shape.rect.width() - extent).abs() < 0.5
                && (shape.rect.height() - extent).abs() < 0.5 =>
        {
            Some(shape.fill.a())
        }
        _ => None,
    })
}
#[test]
fn standalone_expand_controls_keep_the_eighty_percent_inverse_fill() {
    let mut h = harness();
    h.run_steps(3);
    let figure = h.state().rect.expect("image rect recorded");
    let alpha = control_alpha(&h.output().shapes, figure, 40.0).expect("expand control painted");
    assert_eq!(alpha, (0.8 * 255.0 + 0.5) as u8);
}
