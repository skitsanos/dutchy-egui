# URL images

Enable the optional integration:

```toml
[dependencies]
dutchy-egui = { version = "1.0.20", features = ["url-images"] }
```

```rust
use dutchy_egui::{Image, ImageAspect, Theme};

fn setup(ctx: &egui::Context) {
    dutchy_egui::install(ctx, Theme::default());
}
fn photo(ui: &mut egui::Ui) {
    let url = "https://picsum.photos/id/237/960/640.jpg";
    let image = Image::new("puppy", url, "Black puppy on a wooden floor")
        .aspect(ImageAspect::Video)
        .retryable(true)
        .show(ui);
    if image.retry_requested {
        ui.ctx().forget_image(url);
    }
}
```

`Image`, `GalleryItem` and `Lightbox` accept these URLs through their normal
source API. egui's own image widgets can use the installed loader too. Network
access starts only when a visible image needs its URL. Source URLs without file
extensions work: PNG/JPEG signatures determine the codec. Redirects follow the
transport's policy. Non-2xx responses and invalid/unsupported content become
`ImageStatus::Failed`; the product UI presents its existing retry affordance.

The integration is original repository-owned code over ehttp **0.7.3** and image
**0.25.10**, current stable releases verified on 2026-11-06. No widget/helper crate
is added. This feature adds only the HTTP transport and image codec dependencies;
the independent default `markdown` feature adds its parser. Disable all default
and optional features for an egui-only direct runtime dependency set.
The integration handles HTTP(S) textures only; file paths, SVG, byte sources,
authenticated request configuration and additional formats need host loaders.
Give included bytes a non-HTTP URI such as `bytes://logo.png` so they remain
with host byte/codec loaders. PNG/JPEG are accepted as still images; animated-format behavior is not promised.

## Configuration and cache

`dutchy_egui::install` registers default limits once. To change them, call
`image_loaders::install_with_options(ctx, UrlImageOptions { ... })` **before**
`install`. First registration wins. Applications using theme/fonts separately
can call `image_loaders::install(ctx)` directly. Registration performs no fetch.
Install custom matching loaders afterward when they should take priority.

Defaults: 16 MiB encoded body per request, four million decoded pixels, four
simultaneous fetch/decode operations and 16 cached URL/texture-option entries.
Bodies are checked against Content-Length and each streamed chunk. Oversized
images are rejected by dimensions before full decoding; codec allocation limits
also apply, including the renderer's maximum texture dimension before upload.
These bounds are per image/entry, not a single global memory budget;
transport chunks, decoder scratch memory and GPU copies require additional space.

Repeated use of one URL with the same texture options shares a request/texture,
including thumbnails and the lightbox. Different texture options count as
separate entries. Cache entries retain textures, failures or pending results.
Inactive least-recently-used entries can be evicted, protecting the current and
previous egui passes. That protection reads the current viewport's pass counter
for one context-wide cache, so a host that loads URL images in several viewports
may see an entry evicted while another viewport still shows it. If the cache is full of visible entries, an explicit failure
asks the host to increase capacity or show fewer images. There is no disk cache,
background refresh or automatic retry. HTTP cache-control revalidation is not
implemented by this loader.

Retry is an application action: `ctx.forget_image(url)` removes all option
variants for that URL; `ctx.forget_all_images()` clears all image loaders.
Forgotten callbacks cannot reinsert old results. Transfers stop at their next
callback; forget is not immediate socket cancellation. Active transfers retain
concurrency permits until the transport exits, so repeated retry cannot spawn
unbounded workers. Completions request repaint even with reduced motion.

## Platforms

On native platforms ehttp fetches and decodes on its worker thread; uploading the
finished texture happens in egui's UI pass. A decoder panic is caught on that
thread and reported as the failure "Image decoder failed", so retry stays
available; on wasm a panic aborts the page. `response_body_timeout` defaults to
30 seconds for native response-body reads. In ehttp 0.7.3 it does **not** bound
connection establishment/headers and is ignored by browser fetches. A stalled
transport may therefore retain a concurrency slot. Applications needing strict
end-to-end deadlines, transport cancellation or authenticated requests should
supply their own loader.

WASM compiles with browser fetch/streaming. Decode runs in the browser callback;
it is not moved to a Web Worker. URLs must allow CORS, and HTTPS apps must avoid
mixed-content HTTP requests. Browser execution remains separate acceptance work;
the native live-provider check does not establish browser compatibility.

## Showcase

```sh
cargo run --example showcase --features url-images
```

On Images, Galleries or Lightboxes, select **Use Picsum photos**. The switch is
shared across all three pages; offline artwork is the initial source. **Reload
photos** forgets the six collection URLs. Expansion/navigation keeps host-owned
selection and keyboard/focus behavior. The portrait sample demonstrates the same
contain/cover comparisons as the offline artwork. Loading/failure preserves the
image box. Pending images show a rotating square spinner and “Loading image”
label; reduced motion uses a static indicator. This is indeterminate progress,
without an estimated percentage. Retry and lightbox close/navigation remain
available.

The demo uses [Lorem Picsum's specific-image endpoints](https://picsum.photos/)
with photographer credits and links to the originals. IDs and sizes are stable,
so revisiting a component does not pick an unrelated image. No API key is needed,
and no remote photographs are distributed in the crate. Public service uptime
is outside this library's control. Automated HTTP tests use a local fixture,
without calling Picsum. See [verification](verification-url-images.md) and
[DS-022](issues/DS-022.md).
