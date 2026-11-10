//! Real HTTP -> decode -> GPU texture -> accessible image/gallery/viewer.
#![cfg(all(
    feature = "url-images",
    feature = "bundled-fonts",
    not(target_arch = "wasm32")
))]
mod common;
#[path = "common/http/mod.rs"]
mod http;
use dutchy_egui::{
    Accent, Gallery, GalleryItem, Image, ImageAspect, ImageStatus, Lightbox, Mode, OverlayOptions,
    Theme,
};
use egui::{Id, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};
use std::{
    thread,
    time::{Duration, Instant},
};
#[derive(Default)]
struct State {
    status: Option<ImageStatus>,
    selected: Option<u8>,
    opener: Option<Id>,
    retries: usize,
}
fn harness(base: String, theme: Theme, scale: f32, path: &'static str) -> Harness<'static, State> {
    Harness::builder()
        .with_size([820.0, 1400.0])
        .with_pixels_per_point(scale)
        .with_theme(if theme.mode == Mode::Dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        })
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(
            move |ui, state: &mut State| {
                let uri = format!("{base}{path}");
                let image = Image::new("remote", uri.clone(), "Remote sample")
                    .width(320.0)
                    .aspect(ImageAspect::Video)
                    .retryable(true)
                    .show(ui);
                state.status = Some(image.status);
                if image.retry_requested {
                    state.retries += 1;
                    ui.ctx().forget_image(&uri);
                }
                let items = [
                    GalleryItem::new(0u8, format!("{base}/png"), "Remote lake").caption("HTTP PNG"),
                    GalleryItem::new(1u8, format!("{base}/jpeg"), "Remote forest")
                        .caption("HTTP JPEG"),
                ];
                let gallery = Gallery::new("remote-gallery", &items).show(ui);
                if let Some(key) = gallery.open_requested {
                    state.selected = Some(key);
                    state.opener = gallery
                        .images
                        .iter()
                        .find(|(id, _)| *id == key)
                        .and_then(|(_, image)| image.expand.as_ref().map(|r| r.id));
                }
                Lightbox::new("remote-viewer", &items, &mut state.selected)
                    .options(OverlayOptions::default().return_focus(state.opener))
                    .show(ui.ctx());
            },
            State::default(),
        )
}
fn wait(h: &mut Harness<'_, State>, predicate: impl Fn(&State) -> bool) {
    let start = Instant::now();
    loop {
        h.run_steps(2);
        if predicate(h.state()) {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(5));
        thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn http_images_render_and_navigate_at_multiple_themes_and_scales() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (mode, accent, scale, name) in [
        (Mode::Light, Accent::Orange, 1.0, "light-1x"),
        (Mode::Dark, Accent::Orange, 1.5, "dark-1_5x"),
        (Mode::Light, Accent::Purple, 2.0, "purple-2x"),
    ] {
        let server = http::Server::new();
        let mut h = harness(server.base.clone(), Theme::new(accent, mode), scale, "/png");
        wait(&mut h, |state| state.status == Some(ImageStatus::Ready));
        h.run_steps(3);
        h.check_accessibility();
        let rect = h.get_by_role_and_label(Role::Image, "Remote sample").rect();
        let rendered = h.render().unwrap();
        let center = rect.center() * scale;
        assert_eq!(
            rendered.get_pixel(center.x as u32, center.y as u32).0,
            [12, 150, 240, 255]
        );
        rendered
            .save(format!("target/visual-proof/url-images-{name}.png"))
            .unwrap();
        let opener = "Expand Remote lake";
        h.get_by_label(opener).focus();
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        h.run_steps(4);
        assert_eq!(h.state().selected, Some(0));
        h.key_press(egui::Key::ArrowRight);
        h.run_steps(4);
        assert_eq!(h.state().selected, Some(1));
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/url-lightbox-{name}.png"))
            .unwrap();
        h.key_press(egui::Key::Escape);
        h.run_steps(5);
        assert!(h.state().selected.is_none());
        assert!(h.get_by_label(opener).is_focused());
        assert_eq!(server.calls("/png"), 1); // Standalone, gallery and viewer share a texture.
    }
}
#[test]
fn url_error_retry_button_forgets_the_failure_and_recovers() {
    let server = http::Server::new();
    let mut h = harness(server.base.clone(), Theme::default(), 1.0, "/error");
    wait(&mut h, |state| {
        matches!(state.status, Some(ImageStatus::Failed(_)))
    });
    h.get_by_label("Retry image Remote sample").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    wait(&mut h, |state| {
        state.retries == 1 && state.status == Some(ImageStatus::Ready)
    });
    assert_eq!(server.calls("/error"), 2);
    h.check_accessibility();
}

#[test]
fn a_slow_http_response_keeps_a_named_loading_animation_until_the_texture_is_ready() {
    let server = http::Server::new();
    let mut h = harness(server.base.clone(), Theme::default(), 1.0, "/slow");
    server.wait_for("/slow", 1);
    h.run_steps(2);
    assert_eq!(h.state().status, Some(ImageStatus::Loading));
    assert!(
        h.query_by_role_and_label(Role::ProgressIndicator, "Loading Remote sample")
            .is_some()
    );
    wait(&mut h, |state| state.status == Some(ImageStatus::Ready));
    assert!(
        h.query_by_role_and_label(Role::ProgressIndicator, "Loading Remote sample")
            .is_none()
    );
    assert_eq!(server.calls("/slow"), 1);
}
