# Scribble verification — 2026-11-06 / 2026-11-07

**Release:** 1.0.14 was committed and pushed to `main` as
`cc1018370c9afa09a2856fd95f1c45d2ff08d5e3`. A fresh remote-head query matched.
The prior successful check/package evidence was inspected before committing;
398 distributed source/assets/docs files matched the working tree, excluding
Cargo's normalized manifests/lock metadata. No registry publication or tag was
requested.

**1.0.15 release preparation:** The user authorized the qualified main release on
2026-11-07. The final packaged source matched the successful check results before
release; the main baseline was 1.0.14. Crates.io checks on 2026-11-06 confirmed
egui 0.37.0, image 0.25.10 and development-only rfd 0.17.2 as current stable
releases. `cargo tree --no-default-features --features png-export -e normal
--depth 1` shows egui and image, without ehttp; the default graph remains egui
only. No widget/helper implementation was vendored.

## Cleanup

The superseded task-owned upload preview was closed before cleanup. A cleanup
subagent used `cargo clean`, removing 48,025 files / 15.3 GiB, then deleted the
merged `ticket/file-uploads-drag-drop` branch and pruned worktree metadata.
Only the active worktree existed. `main`, active development, tracked assets/
snapshots and the unmerged Renovate remote branch were preserved. Build outputs
were subsequently recreated for this milestone. Three generated snapshot backup
images were removed after reviewing the new baselines.

## Model, input and export evidence

Model/export tests cover invalid dimensions/nonfinite/empty/out-of-bounds input,
nonmutating rejection, undoable clear, branching redo, immutable snapshots,
history/stroke/point limits, SVG/dots/colors/background, analytic raster geometry,
single compositing at translucent joins, PNG pixel/dimension round-trips and
preflight dimension/pixel/work limits. Fractional output rounding retains uniform
scale, centered geometry and full optional background coverage.

Interaction tests cover ordered coalesced primary-pointer events including the
release endpoint, other-button rejection, dots, leave/reentry without bridging,
keyboard stroke/dot/cancel/clear/undo, host-only history/export requests,
read-only exports, disabled/parent-disabled/clipped/modal guards, model replacement,
window focus loss, discarded passes, resizing, captured brushes, stable toolbar
identity/focus after sibling insertion and reflow, and explicit
capacity failure without data loss. Small model replacement resets an invalid
pen cursor. Escape follows native egui focus release; commands need focus again.

Showcase tests cover sample/keyboard edits, host undo/redo/clear, asynchronous
preview completion, real worker SVG file bytes and I/O failure feedback. Desktop
and narrow showcase previews exercise the public API. Workers receive immutable
snapshots; these tests never open a native picker or transmit drawings.

## Rendering and platform boundaries

Reviewed WGPU baselines:
`tests/snapshots/scribble-{light-1x,dark-1_5x,purple-2x}.png`, plus corresponding
`-png.png` baselines for the optional PNG toolbar control. They cover empty,
ink/focus/cursor, read-only and disabled surfaces. Paper/ink stay legible in dark
mode, and the focus cue follows the accent. Geometry checks at 280 and 160 points
verify canvas and toolbar containment; controls wrap into rows. Artifacts are
under `target/visual-proof/scribble-*.png`.

Native macOS Glow verification completed on 2026-11-07 after the Mac was
unlocked. A pointer drag committed a stroke; Space/arrows/Enter added another
without leaving the canvas. Cmd+Z, Cmd+Shift+Z, Clear and undoing Clear retained
consistent data in the editable and read-only views. Light/dark presentation and
a completed raster preview were observed.

Save As wrote a transparent 1200×400 SVG (515 bytes, two polylines/five points)
and a paper-backed 1200×400 RGBA PNG (17,405 bytes). XML geometry, PNG header and
chunk CRCs were checked; the PNG was visually reviewed. Sample exports also
exercised the native save path. Cancellation retained the drawing. After both
Save and Cancel, keyboard Enter/Space reopened the correct picker from its
restored opener; no pointer refocusing was needed. Fixtures are under
`target/visual-proof/native-scribble-20261107-mlc6plx9/`; the task preview was closed.

This check exposed two issues compilation/harness clicks had missed. Unmodified
arrows invoked egui's spatial focus navigation; an explicit editable-canvas event
filter now owns arrows while retaining Tab/Escape exit. A raw-context regression
splits native-style key down/up frames and verifies focus plus completed points.
Picker focus also needed OS-window activation and deferred restoration after the
export control becomes enabled. A controlled worker-completion test covers busy,
completion, cancellation and preservation of an intervening host focus change.

Physical touch/pen delivery, Windows/Linux GUI, browser runtime/download integration,
active-screen-reader and minimum Rust 1.95 toolchain acceptance also remain
separate. WASM compilation and deterministic input tests do not establish those
platform behaviors. Native epaint, SVG and CPU export use different edge
rasterizers; pixel-identical screenshot/export parity is not claimed.

## Scroll regression — DS-026

A later native check reproduced wheel scrolling snapping back to the focused
editable canvas. Toolbar controls and read-only focus had the same repeated
scroll request. Reveal now follows focus changes or explicit editing/activation;
manual scrolling retains control between those events.

Four focused scroll-area tests failed against the earlier behavior and pass with
the fix: scroll to read-only while the editor retains focus, scroll past a focused
toolbar, reveal read-only once then scroll away, and reveal/edit an offscreen
keyboard draft without cancelling it on the resulting layout change. Completed
host data is retained throughout. Pointer drafts still cancel on rectangle changes.

Native Glow checks confirmed all four flows, including Tab/Shift+Tab reveal and
an intact three-point keyboard stroke after scrolling away. The default empty
read-only hint was visually verified as “No drawing.” The corrected preview was
left at the read-only view for review. See [DS-026](issues/DS-026.md).

## Checks

Final checks passed on 2026-11-07, macOS aarch64 with Rust/Cargo 1.99.1:

- `cargo fmt --all --check`
- `cargo test`: 278 integration tests, ten unit tests, one doctest.
- `cargo test --all-features`: 290 integration tests, ten unit tests, one doctest.
- `cargo test --no-default-features`: 202 integration tests, ten unit tests,
  one doctest, including host-font fallback.
- `cargo test --no-default-features --features png-export`: 203 integration tests,
  ten unit tests, one doctest, without HTTP or bundled library fonts.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo check --lib --all-features --target wasm32-unknown-unknown`
- `cargo build --example showcase --all-features`

Log: `target/visual-proof/checks-1.0.15.log` (including default/PNG visual baselines
the stable-toolbar regression and DS-026 scroll cases). All 250 Rust files are below
400 lines (largest: 370); Clippy enforces cognitive-complexity threshold 15.
Snapshot comparisons passed without update mode after visual review. Local
Markdown links and `git diff --check` also passed. Package qualification is
recorded in `target/visual-proof/package-1.0.15.log`.

See [Scribble decisions](scribble-export.md) and [DS-025](issues/DS-025.md).

Package qualification passed: `cargo package --allow-dirty`, an independent
extracted all-features showcase build, and a standalone consumer of that extracted
package with PNG but no fonts/HTTP. The consumer generated 1200×400 SVG, transparent
PNG and paper PNG from sample strokes. Archive comparison matched 429 source/
assets/docs files; fonts, licenses, notices and all new guides are included, while
generated target outputs and snapshot backups are excluded.

## 1.0.19 pointer smoothing

Opt-in `ScribbleSmoothing` is covered by unit tests in `src/scribble/smooth.rs`
and by `tests/scribble_smoothing.rs`: pass-through, endpoint anchoring, tremor
and turn-angle reduction per level, in-bounds output at the drawing edges,
widget/`smooth()` equivalence, level capture at gesture start, dots and exact
keyboard strokes. A WGPU check at 1x and 2x shows that live and committed
smoothed ink match (0 pixels beyond a one-pixel tolerance); removing the
preview's catch-up tail made it fail. Existing Scribble baselines passed
unchanged on Mesa lavapipe. Evidence and limits are recorded in
[DS-046](issues/DS-046.md).

Checks on 2026-11-07 (Linux container, Mesa lavapipe for WGPU): formatting,
the three Clippy configurations and rustdoc with `-D warnings` pass. The
no-default-features suite passes (262 tests). The all-features suite passes
358 tests; the only failures are 2 `tests/media_rendering.rs` image snapshots
(`image-expand-hover`, `images-light-1x`, 234 pixels each). They fail
identically on an untouched 1.0.18 checkout in the same container, so they
come from the software renderer, not this change. They still need confirming
on the reference renderer. The wasm check was not run because the
`wasm32-unknown-unknown` target is not installed here.
