# Verification — Markdown and CodeBlock

Date: 2026-11-08. Local development on the `gedankrayze/vibrant-cerf-rat7ho`
branch, which used a parallel 1.0.23 label; no registry publication. This is
the branch's original qualification, not the main release history. Integration
onto current main is Unreleased; see
[the combined record](verification-fixes-markdown.md). Dependency change: the
default `markdown` feature adds pulldown-cmark 0.13.6 (MIT, default features
off). Its parser dependencies resolve to the current releases bitflags 2.13.2
and memchr 2.8.3, already in the lockfile, plus unicase 2.10.0 (MIT OR
Apache-2.0); nothing older or differently licensed enters the tree.

## Automated coverage

Final checks passed on Linux x86_64 (Ubuntu 24.04, Rust/Cargo 1.97.0, Mesa
25.2 lavapipe as the WGPU adapter; existing baselines rendered identically):

- `cargo fmt --all --check`.
- `cargo test --all-features`: 443 passed across 85 binaries;
  `cargo test --no-default-features`: 314 passed. No ignored tests. Two
  pre-existing media baselines (`images-light-1x`, `image-expand-hover`)
  differ by one pixel column at an image edge (234 pixels) on this software
  adapter; the unchanged `main` tree reproduces the same two differences, so
  they are an adapter difference, not an effect of this change (see Limits).
- `cargo clippy --all-targets -- -D warnings`, plus the `--all-features` and
  `--no-default-features` configurations, each with warnings denied.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`.
- `cargo check --lib --target wasm32-unknown-unknown` with all features and
  with no default features, and `cargo build --example showcase`.

The component adds ten model unit tests, nine interaction tests, two showcase
tests and three rendering tests with six reviewed baselines. One existing
showcase test (`layout_rendering`) now scrolls the sidebar before clicking a
page button, because the page list grew by one entry and overflowed at its
harness height; the assertion itself is unchanged.

Focused evidence:

- Event mapping: GitHub anchors with numbered duplicates and `{#id}` overrides,
  merged styled runs, soft breaks as spaces and hard breaks or `<br>` as
  newlines, links with titles and nested runs, tight-list implicit paragraphs,
  task markers in tight and loose items, nested lists, image-only and linked
  image paragraphs as figures, fenced language and trailing newline handling,
  table alignments and padded short rows, quotes, callouts, rules, HTML text
  extraction and opt-in smart punctuation.
- Semantics: Document, Heading levels (including offset and the level-six
  clamp), Link URL and description, read-only CheckBox state, Table counts with
  ColumnHeader and Cell children, ListItem and Paragraph counts, Code value.
- Behavior: external links emit one `OpenUrl` command and one report under the
  default policy and no command under `Report`; Tab reaches a link once even
  when it spans rows; Enter and Space activate; a `#fragment` click scrolls its
  heading to the top of an enclosing scroll area; Copy emits `CopyText` with
  the code and reads "Copied"; source edits re-render while link focus and
  identity survive; images use the host loader or fall back to alt text; a
  disabled parent disables links and the copy control.
- Rendering: Orange/light 1x, Blue/dark 1.5x and Purple/light 2x baselines,
  a 300-point narrow baseline, keyboard-focused link and hovered copy control.
  All rectangles are sharp; link rows stay inside the width; table columns
  stop shrinking at their widest word.

## Review findings

Reviewing the first baselines found two defects, both fixed before the
baselines were accepted: the ghost copy control's hover fill was the muted
color of the code surface and therefore invisible (the control is now an
outline button that inverts on hover), and proportional column shrinking broke
"VALUE" mid-word in the narrow table (columns now distribute only the slack
above their widest word and scroll when the words alone overflow).

## Showcase

The Markdown page renders the sample document with host-logged link
activations, a heading-demotion switch, smart punctuation, line numbers and a
live source editor. The showcase test navigates to the page, passes the
accessibility check, reports a link without an `OpenUrl` command, demotes the
title to level two and follows a source edit. The desktop preview is
`target/visual-proof/markdown-showcase.png`.

## Limits

No native GUI session was run in this environment; the Glow showcase was built
but not displayed, and no screen reader was exercised. Metadata tests do not
establish screen-reader acceptance. Inline images, footnotes, math and raw
HTML are outside the rendered scope. Text selection spans one block at a time.
The headless adapter is a software Vulkan driver installed for this session,
not a GPU. Its rasterization of one image edge differs from the macOS GPU that
produced the media baselines; those two baselines were left untouched rather
than regenerated here, and a GPU run should confirm them unchanged.
