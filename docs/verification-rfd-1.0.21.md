# Verification — rfd 0.18.0 / release 1.0.21

Date: 2026-11-07. Covers Renovate
[PR #2](https://github.com/skitsanos/dutchy-egui/pull/2), original head
`fa2f655f9ba82f7df422eeae9b178219f08863e7`.

## Dependency and API review

The crates.io API confirms rfd 0.18.0 is the latest stable, non-yanked release.
Its Rust 1.92 minimum fits the project's Rust 1.95 requirement. Checked the
published crate's MIT license and `FileDialog` methods used by the showcase:
construction, title, filters, filename, `pick_files` and `save_file`.
Both call sites compile unchanged. rfd remains development-only; the core
runtime dependency graph is unchanged.

The upstream changelog includes macOS filename/directory fixes, GTK fixes and
an optional GTK4 backend. This release retains the existing feature choices.
The lockfile adds uniform-type-identifiers 0.3.2 and removes the extra pollster
0.4.0 entry. The package itself advances from 1.0.20 to 1.0.21.

## Automated qualification

The local release receipt under `.git/codex-maintenance/` records commands and
results. Passed formatting, 406 all-feature tests, 302 no-default-feature tests,
all three required Clippy configurations with warnings denied, rustdoc with
warnings denied, both WASM library configurations and the all-feature native
showcase build. Existing rendering baselines passed across multiple themes at
1x, 1.5x and 2x without changes. Package verification uses a clean detached
checkout of the release commit to exclude unrelated local video files.

The unrelated, untracked storyboard test is an explicitly ignored video-frame
generator; it accounts for the one ignored local all-feature test and is
excluded from the release. `/media/` is gitignored at the user's request.
Media source and storyboard outputs are preserved during cleanup.

## Native macOS acceptance

Ran a fresh all-feature showcase reporting 1.0.21 with rfd 0.18.0:

- Opened the multi-file picker from File Uploads. Cancellation retained an
  empty collection and restored focus to its opener.
- Selected a task-owned 46-byte TXT fixture. The host displayed its name,
  size and ready state. Reopening and cancelling retained that selection.
- Loaded the deterministic sample drawing: three strokes, 23 points.
  SVG Save As supplied `dutchy-scribble.svg`; cancelling retained the drawing
  and restored focus to the export button.
- Saved SVG and PNG to the task's verification directory. Both completed and
  restored focus to their export buttons. The PNG dialog supplied the correct
  `.png` filename. No existing file was overwritten.
- Read back the SVG XML and verified three polylines and 23 points. Validated
  PNG signature, chunk checksums, 1200 × 400 dimensions and decompressed
  scanline length. SVG/PNG outputs were 939 and 27,281 bytes, respectively.
- Closed the task-owned preview after verification; unrelated apps were left
  running. Generated fixtures/exports are eligible for post-push cleanup.

## Limits and provenance

Windows/Linux native dialogs and browser file pickers were not run. WASM
checks validate the library; the synchronous rfd showcase adapter is native.
No new picker options or widget behavior were introduced.

Sources: [crate registry](https://crates.io/crates/rfd),
[0.18.0 changelog](https://github.com/PolyMeilex/rfd/blob/0.18.0/CHANGELOG.md),
[published FileDialog API](https://docs.rs/rfd/0.18.0/rfd/struct.FileDialog.html).
