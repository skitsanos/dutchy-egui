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
///
/// Start from [`UrlImageOptions::default`] and chain the setters; fields can
/// also be read or assigned. New limits may be added in minor releases.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
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

impl UrlImageOptions {
    /// Maximum encoded response body retained per request.
    pub fn max_download_bytes(mut self, bytes: usize) -> Self {
        self.max_download_bytes = bytes;
        self
    }

    /// Maximum decoded width times height.
    pub fn max_pixels(mut self, pixels: u64) -> Self {
        self.max_pixels = pixels;
        self
    }

    /// Maximum cached entries; values below one become one.
    pub fn max_entries(mut self, entries: usize) -> Self {
        self.max_entries = entries;
        self
    }

    /// Maximum simultaneous fetch/decode operations; values below one become one.
    pub fn max_concurrent(mut self, operations: usize) -> Self {
        self.max_concurrent = operations;
        self
    }

    /// Native response-body read timeout.
    pub fn response_body_timeout(mut self, timeout: Duration) -> Self {
        self.response_body_timeout = timeout;
        self
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
