# Verification — MediaCard and SearchInput

Date: 2026-11-08. Local Unreleased work after repository release 1.0.22
(`0c60a874d257096438052013cd6de26b6d3b96e7`). The new components and package
allowlist hardening have not been committed or pushed. No dependencies added.

## Interaction and rendering evidence

- SearchInput: native query editing and Unicode paste limits; pointer/label/
  keyboard focus; conditional Clear by pointer/Space with one host change and
  focus return; read-only selection with no edit notification; disabled parents;
  stable IDs under inserted siblings and external host updates; responsive
  hints/clear targets at every control size. Escape stays available to the host.
- MediaCard: preview and metadata open requests; independent saved-state
  requests that do not implicitly mutate the host; Enter/Space; selected/disabled
  metadata; stable identity under renaming and inserted siblings; loading/failure
  transitions with stable image geometry; retry without opening or saving.
- Artwork-browser integration: query filtering, clear, saved-only removal,
  empty guidance and host Lightbox entry/Escape/focus return. Removing the last
  saved card returns focus to search. A separate scroll regression keeps a
  focused card from pulling the page back after manual scrolling.
- Reviewed three new baselines: Orange/light 1x, Orange/dark 1.5x and Blue/light
  2x, with ready/loading/failed cards and keyboard-visible image actions. Reviewed
  both showcase pages at desktop/narrow widths and a long-copy narrow card.
  Existing tracked snapshot baselines are unchanged.

The larger navigation list exposed a test that clicked Tooltips below the
sidebar clip. It now scrolls that item into view before clicking. Search
adornment hit areas are restricted to decorative margins to avoid overlapping
the native editor. The Unicode test opts into tofu for emoji outside the bundled
font's coverage; that test concerns editing/counting, not glyph rendering.

## Native macOS

Verified save without opening, Cmd+K focus, typed filtering/counts, clear/focus,
metadata preview, Escape/focus return, saved-only removal/empty/reset, alphabetical
sort, dark appearance and Blue accent. Search fields expose native SearchInput
roles; bookmarks expose named toggles. Read-only typing was rejected, clear was
disabled, and disabled state reached all sizes. The final rebuilt app verified
physical clicks in text and empty editor space plus typing and clear. The
task-owned preview was closed; unrelated applications were preserved.

## Qualification and limits

All final checks passed: `cargo fmt --all --check`; `cargo test --all-features`
(435 passed) and `cargo test --no-default-features` (324 passed); the three
required `cargo clippy --all-targets` configurations with `-D warnings`;
`RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`; both all-feature
and no-default WASM library checks; and the native showcase build. No tests
failed or were ignored.

Cargo's package file list and actual 538-file crate exclude media, target
outputs and generated snapshot differences. All 534 ordinary content entries
match the checkout; the original/normalized manifests and package version were
checked separately, allowing Cargo's generated VCS metadata and minimized lock.
New component sources, guides and font licenses are present. The isolated crate
verification build passed. The protected manifest's 772 media/storyboard files
remain unchanged. No registry publication was performed.

No Windows/Linux native, browser runtime or screen-reader
acceptance run; WASM compilation is not browser interaction proof. MediaCard
has no custom metadata slots or masonry. Filtering, saved persistence, image
retry policy, navigation and shortcuts stay with the host.

Post-push cleanup reclaimed 30.51 GiB before this development started; only
canonical main/local and remote main remain. Media, storyboard work, font
licenses, tracked assets and private drawing/maintenance receipts were preserved.
The 1.0.22 generated crate's failed content gate and the local fix are recorded
in [DS-058](issues/DS-058.md).

## Repository release 1.0.23

The components and root-anchored package allowlist are included in 1.0.23.
The release repeats the full check matrix and archive-content gate against
that package/lockfile version. README is reduced to essential setup and links;
detailed guidance is indexed under docs. Earlier local-development evidence
above describes the implementation before this release.
