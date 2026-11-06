//! Optional asynchronous HTTP(S) PNG/JPEG loading (`url-images` feature).
//!
//! [`crate::install`] registers this loader automatically. Install custom limits
//! first to override the defaults. Existing application loaders remain usable;
//! the most recently registered matching egui loader takes priority.
//!
//! Requests and failures are cached by URL and texture options. Retry explicitly
//! with [`egui::Context::forget_image`]. Browser requests follow CORS rules.
mod cache;
mod decode;
mod fetch;

use std::{sync::Arc, time::Duration};

/// Resource limits for the optional URL image loader.
///
/// These are per context. Cache entries include pending requests and distinct
/// texture-option variants. CPU decode runs on the HTTP worker on native and
/// on the browser callback on WASM. Animated images are treated as still images.
#[derive(Clone, Copy, Debug)]
pub struct UrlImageOptions {
    /// Maximum encoded response body retained per request (default 16 MiB).
    pub max_download_bytes: usize,
    /// Maximum decoded width times height (default 4 million pixels).
    pub max_pixels: u64,
    /// Maximum cached entries (default 16). Values below one become one.
    /// Inactive least-recently-used entries are evicted; visible entries stay.
    pub max_entries: usize,
    /// Maximum simultaneous fetch/decode operations (default four).
    /// Values below one become one, including forgotten requests still exiting.
    pub max_concurrent: usize,
    /// Native response-body read timeout (default 30 seconds).
    /// ehttp does not apply it to connection/headers or browser fetches.
    pub response_body_timeout: Duration,
}

impl Default for UrlImageOptions {
    fn default() -> Self {
        Self {
            max_download_bytes: 16 * 1024 * 1024,
            max_pixels: 4_000_000,
            max_entries: 16,
            max_concurrent: 4,
            response_body_timeout: Duration::from_secs(30),
        }
    }
}

/// Register the URL loader once with default limits. Makes no network requests.
pub fn install(ctx: &egui::Context) {
    install_with_options(ctx, UrlImageOptions::default());
}

/// Register with custom limits. The first installation in a context wins.
///
/// Call before [`crate::install`] to override automatic registration. Only
/// HTTP(S) is handled; texture, byte, file and other URI schemes are untouched.
pub fn install_with_options(ctx: &egui::Context, options: UrlImageOptions) {
    if !ctx.is_loader_installed(cache::ID) {
        ctx.add_texture_loader(Arc::new(cache::Loader::new(options)));
    }
}
