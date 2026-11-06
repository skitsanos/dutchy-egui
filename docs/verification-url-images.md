# URL image and loading feedback verification — 2026-11-06

Package **1.0.13**, local branch `ticket/images-gallery-lightbox`. This extends
[initial media verification](verification-media.md); 1.0.13 remains uncommitted
and unpushed. No registry publication was performed.

## Dependencies and implementation

The crates.io API and official documentation confirmed egui 0.37.0, ehttp 0.7.3
and image 0.25.10 as current stable releases. `url-images` enables only the two
new optional direct dependencies; `cargo tree -e normal --no-default-features
--depth 1` confirms egui alone in the default runtime dependency graph. The
integration is original code. No widget/helper implementation was copied.

A context-owned texture loader accepts HTTP(S), performs streamed asynchronous
fetch/decode, then uploads in the UI pass. Cache keys include URL and texture
options. Encoded byte, pixel, renderer-dimension, entry-count and concurrency
limits are enforced, including limits while inspecting image headers. Decoded
buffers are released after upload. Forgotten callbacks cannot reinsert results.
See [URL decisions and platform limits](url-images.md).

## HTTP and interaction evidence

Eight local-server integration tests cover extensionless PNG/JPEG, HTTP
redirects, texture/request reuse, cached non-2xx/invalid-content failures,
explicit forget/retry, advertised and streamed byte caps, pixel/texture limits,
native body timeout, nonblocking pending polls, completion repaint requests,
cancelled transfer permits, queued work, stale completions, inactive cache
eviction, visible-entry retention and first-install custom options. Fixture
sockets explicitly use blocking reads after nonblocking listener acceptance,
which matters on macOS. No public provider is called by the automated suite.

Three HTTP UI tests cover a real URL -> decoded image -> GPU texture workflow,
public retry recovery, a deliberately slow response's named loading indicator,
and gallery/lightbox keyboard navigation/focus return. A known PNG color is
sampled from the actual rendered image. Independent theme/scale outputs are:
`target/visual-proof/url-images-{light-1x,dark-1_5x,purple-2x}.png` and corresponding
`url-lightbox-*` files. Multiple uses of one URL perform one GET.

Three loading tests compare animated versus static shapes and repaint delays,
reduced motion, clipped scheduling, busy ProgressIndicator metadata,
completion/failure removal, unchanged reserved dimensions and tiny boxes.
Pending media now show a square Spinner and “Loading image” label, omitting the
label/spinner as space becomes insufficient. Progress is indeterminate. Existing
media goldens were reviewed and updated for this intentional pending-state change;
loaded-image fitting and lightbox baselines remain covered.

## Native and provider evidence

Live HTTP requests to Picsum IDs 10, 28, 237, 870, 1025 and 1043 returned status
200 and JPEG content after redirects. Provider metadata supplied the verified
photographer names and original Unsplash links. Downloaded inspection samples
stay under ignored `target/visual-proof/picsum`; none are packaged.

The native Glow showcase on macOS displayed a live lake photo in an expanded
viewer, navigated to the forest photo, loaded the collection, explicitly reloaded
it, and expanded the puppy photograph in dark mode with reduced motion enabled.
A subsequent native pass showed the pending host example's visibly rotated
square, switched to its static reduced-motion treatment, then removed the busy
indicator when “Show loaded image” supplied the texture. Native inspection and
WGPU checks use distinct renderers.

## Acceptance boundaries

Native GUI evidence is macOS-only without an active screen reader. Browser
WASM runtime/CORS, Windows/Linux GUI, strict end-to-end deadlines/cancellation,
authenticated HTTP, additional codecs and a Rust 1.95 toolchain run remain
separate work. ehttp's configured timeout covers native response-body reads;
connection/header waits and browser fetches are not bounded by it. Forget stops
at the next callback rather than immediately cancelling the socket. A stalled
transport may retain its concurrency slot. These limitations are documented
rather than inferred away from successful compilation or provider responses.

Issue records: [DS-022](issues/DS-022.md) and [DS-023](issues/DS-023.md).

## Final checks and package qualification

Passed on macOS aarch64, Rust/Cargo 1.99.1:

- `cargo fmt --all --check`
- `cargo test --all-features`: 241 integration tests, ten unit tests, one doctest.
- `cargo test --no-default-features --features url-images`: 175 integration
  tests, ten unit tests, one doctest.
- `cargo test --no-default-features`: 167 integration tests, ten unit tests,
  one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo check --lib --all-features --target wasm32-unknown-unknown`
- `cargo build --example showcase --features url-images`
- `git diff --check`; local documentation links resolve. All 212 Rust files
  remain below 400 lines; the largest is 382.

Log: `target/visual-proof/checks-url-images-final.log`. The package inventory has
378 files, including new source/docs and all bundled font licenses/notices.
Downloaded provider samples and generated snapshot backups are excluded.
Package and extracted-example verification commands and results are recorded in
`target/visual-proof/package-url-images.log`.
