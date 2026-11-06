//! Deterministic HTTP integration: no public network calls in the test suite.
#![cfg(all(feature = "url-images", not(target_arch = "wasm32")))]
#[path = "common/http/mod.rs"]
mod http;
use dutchy_egui::image_loaders::{self, UrlImageOptions};
use egui::{
    Context, TextureOptions,
    load::{SizeHint, SizedTexture, TextureLoadResult, TexturePoll},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn ctx(options: UrlImageOptions) -> Context {
    let ctx = Context::default();
    image_loaders::install_with_options(&ctx, options);
    ctx
}
fn load(ctx: &Context, uri: &str) -> TextureLoadResult {
    ctx.try_load_texture(uri, TextureOptions::LINEAR, SizeHint::default())
}
fn finish(ctx: &Context, uri: &str) -> Result<SizedTexture, String> {
    let start = Instant::now();
    loop {
        match load(ctx, uri) {
            Ok(TexturePoll::Ready { texture }) => return Ok(texture),
            Err(error) => return Err(error.to_string()),
            _ => {
                assert!(
                    start.elapsed() < Duration::from_secs(5),
                    "image did not finish: {uri}"
                );
                thread::sleep(Duration::from_millis(2));
            }
        }
    }
}
fn passes(ctx: &Context, n: usize) {
    for _ in 0..n {
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
    }
}
#[test]
fn png_jpeg_and_redirects_decode_without_extensions_and_cache_textures() {
    let server = http::Server::new();
    let ctx = ctx(Default::default());
    for path in ["/png", "/jpeg", "/redirect"] {
        let uri = server.url(path);
        assert!(matches!(load(&ctx, &uri), Ok(TexturePoll::Pending { .. })));
        let texture = finish(&ctx, &uri).unwrap();
        assert_eq!(texture.size, egui::vec2(32.0, 16.0));
        for _ in 0..10 {
            assert_eq!(finish(&ctx, &uri).unwrap().id, texture.id);
        }
        assert_eq!(server.calls(path), 1);
    }
    assert_eq!(server.calls("/png"), 2); // The redirect follows a second GET.
}
#[test]
fn failures_are_cached_and_retry_is_explicit() {
    let server = http::Server::new();
    let ctx = ctx(Default::default());
    let uri = server.url("/error");
    assert!(finish(&ctx, &uri).unwrap_err().contains("503"));
    for _ in 0..5 {
        assert!(finish(&ctx, &uri).is_err());
    }
    assert_eq!(server.calls("/error"), 1);
    ctx.forget_image(&uri);
    assert!(finish(&ctx, &uri).is_ok());
    assert_eq!(server.calls("/error"), 2);
    assert!(finish(&ctx, &server.url("/invalid")).is_err());
}
#[test]
fn byte_pixel_and_timeout_limits_fail_cleanly() {
    let server = http::Server::new();
    let tiny = ctx(UrlImageOptions {
        max_download_bytes: 64,
        ..Default::default()
    });
    for path in ["/large-header", "/large-stream"] {
        assert!(
            finish(&tiny, &server.url(path))
                .unwrap_err()
                .contains("byte limit")
        );
    }
    let pixels = ctx(UrlImageOptions {
        max_pixels: 100,
        ..Default::default()
    });
    assert!(
        finish(&pixels, &server.url("/png"))
            .unwrap_err()
            .contains("pixel limit")
    );
    let dimensions = ctx(Default::default());
    assert!(finish(&dimensions, &server.url("/wide")).is_err());
    let timeout = ctx(UrlImageOptions {
        response_body_timeout: Duration::from_millis(40),
        ..Default::default()
    });
    assert!(finish(&timeout, &server.url("/timeout")).is_err());
}
#[test]
fn pending_is_nonblocking_deduplicated_and_completion_requests_repaint() {
    let server = http::Server::new();
    let ctx = ctx(Default::default());
    let repaints = Arc::new(AtomicUsize::new(0));
    let observed = repaints.clone();
    ctx.set_repaint_observer(move |_| {
        observed.fetch_add(1, Ordering::AcqRel);
    });
    let uri = server.url("/slow");
    let start = Instant::now();
    for _ in 0..20 {
        assert!(matches!(load(&ctx, &uri), Ok(TexturePoll::Pending { .. })));
    }
    assert!(start.elapsed() < Duration::from_millis(250));
    server.wait_for("/slow", 1);
    passes(&ctx, 2);
    let before = repaints.load(Ordering::Acquire);
    assert!(finish(&ctx, &uri).is_ok());
    assert!(repaints.load(Ordering::Acquire) > before);
    assert_eq!(server.calls("/slow"), 1);
}
#[test]
fn forgetting_pending_work_does_not_resurrect_old_textures_or_exceed_concurrency() {
    let server = http::Server::new();
    let ctx = ctx(UrlImageOptions {
        max_concurrent: 1,
        ..Default::default()
    });
    let uri = server.url("/slow");
    load(&ctx, &uri).unwrap();
    server.wait_for("/slow", 1);
    ctx.forget_image(&uri);
    // The cancelled transfer still holds its permit until its transport exits.
    assert!(matches!(load(&ctx, &uri), Ok(TexturePoll::Pending { .. })));
    assert_eq!(server.calls("/slow"), 1);
    let new = finish(&ctx, &uri).unwrap();
    assert_eq!(server.calls("/slow"), 2);
    thread::sleep(Duration::from_millis(20));
    assert_eq!(finish(&ctx, &uri).unwrap().id, new.id);
    ctx.forget_all_images();
    assert_ne!(finish(&ctx, &uri).unwrap().id, new.id);
}
#[test]
fn queued_requests_resume_after_a_concurrency_slot_is_released() {
    let server = http::Server::new();
    let ctx = ctx(UrlImageOptions {
        max_concurrent: 1,
        ..Default::default()
    });
    let slow = server.url("/slow");
    load(&ctx, &slow).unwrap();
    server.wait_for("/slow", 1);
    let other = server.url("/jpeg");
    assert!(matches!(
        load(&ctx, &other),
        Ok(TexturePoll::Pending { .. })
    ));
    assert_eq!(server.calls("/jpeg"), 0);
    finish(&ctx, &slow).unwrap();
    finish(&ctx, &other).unwrap();
    assert_eq!(server.calls("/jpeg"), 1);
}
#[test]
fn bounded_cache_retains_visible_textures_and_evicts_inactive_entries() {
    let server = http::Server::new();
    let ctx = ctx(UrlImageOptions {
        max_entries: 1,
        ..Default::default()
    });
    let a = server.url("/png");
    let b = server.url("/jpeg");
    let old = finish(&ctx, &a).unwrap();
    assert!(finish(&ctx, &b).unwrap_err().contains("cache is full"));
    assert_eq!(finish(&ctx, &a).unwrap().id, old.id);
    passes(&ctx, 3);
    finish(&ctx, &b).unwrap();
    passes(&ctx, 3);
    assert_ne!(finish(&ctx, &a).unwrap().id, old.id);
    assert_eq!(server.calls("/png"), 2);
}
#[test]
fn installation_is_idempotent_and_does_not_handle_unrelated_schemes() {
    let ctx = ctx(UrlImageOptions {
        max_pixels: 1,
        ..Default::default()
    });
    image_loaders::install(&ctx);
    image_loaders::install(&ctx);
    let server = http::Server::new();
    assert!(finish(&ctx, &server.url("/png")).is_err());
    assert!(load(&ctx, "file://unrelated.png").is_err());
}
