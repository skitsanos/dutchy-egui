# Verification — follow-up fixes and Markdown integration

This is the pre-release integration record. The subsequent authorized release
is qualified in [the 1.0.24 record](verification-release-1.0.24.md).

Date: 2026-11-08. Local qualification on macOS 27.2, Rust/Cargo 1.99.1,
after main `cd2688f` (1.0.23). Existing DS-062–066 work is included. DS-067–069
are resolved locally; Markdown/CodeBlock content from remote commit `684a840`
is integrated as Unreleased work (DS-070/071). There is no new commit, version
bump, push or registry publication. The unmerged remote branch is preserved.

## Fix evidence

- **DS-067:** MediaCard titles own their alignment and measured width beside
  the arrow. The regression checks short and long headings at 160/200/360
  points, Left/Center/Right hosts, justification on/off, and three theme/scale
  combinations. All 54 configurations retain the token gap and heading name;
  the right-aligned case failed before the fix.
- **DS-068:** Secondary toggles show a shared-token accent edge when pressed.
  Same-ID comparisons cover all five variants, standalone/Toolbar, hover/rest
  and enabled/disabled states across light 1x, dark 1.5x and Blue 2x. The
  Secondary comparison failed before the fix. Three new image baselines were
  reviewed, including standalone and toolbar states.
- **DS-069:** README distinguishes native Widgets from `.show(...)` composites
  and describes actual Cargo features. The landing-page structure/screenshots
  remain. Setup, notices and historical review records agree with the current
  public API and link this qualification separately from their earlier results.

## Markdown integration

Retained main's root-anchored package allowlist, DS-058 package ticket,
application controls and per-page showcase scroll state. The imported Markdown
ticket is DS-071. `markdown` is enabled by default and adds optional
pulldown-cmark 0.13.6 with its default features disabled; the published version
was checked against [docs.rs](https://docs.rs/crate/pulldown-cmark/0.13.6).
Only pulldown-cmark and unicase are new lockfile packages. CodeBlock remains
available without the parser. The showcase has 39 pages with Markdown and 38
in a fonts-only build.

Additional integration regressions caught and fixed nested emphasis losing its
outer style, native opening of unsupported URI schemes, horizontal host layouts
changing document flow, CodeBlock ignoring requested wrapping, and language/
table-header ink inheriting an inverse Card's foreground. Nested styles now
restore their parent; HTTP(S)/mailto are the only native-open schemes, with
case-insensitive recognition. Other destinations are still reported to the
host. Markdown owns vertical flow; code wrapping and owned-surface ink are
explicit. Tests reproduce these defects before their fixes.

## Automated matrix

All commands passed on the combined tree; no tests failed or were ignored.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 490 passed, 98 test/doc-test groups |
| `cargo test --no-default-features` | 342 passed |
| `cargo clippy --all-targets -- -D warnings` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | Pass |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` | Pass |
| `cargo check --lib --target wasm32-unknown-unknown`, all/minimal features | Both pass |
| `cargo check --all-targets --no-default-features --features bundled-fonts` | Pass |
| Fonts-only `rendering` and `code_block` tests | 12 passed |
| Parser-only library, Markdown, links, composition and CodeBlock tests | 48 passed |
| `cargo +1.95.0 check --lib --all-features --locked` | Declared MSRV passes |
| `cargo build --example showcase --all-features` | Pass |
| `cargo package --allow-dirty --locked --all-features` | Isolated archive build passes |

The fonts-only/parser-only checks use `--no-default-features`. The latter
selects `--features markdown --lib --test markdown --test markdown_links
--test markdown_composition --test code_block`. Package and MSRV verification
use separate target directories to keep compiler/package artifacts isolated.

## Rendering and native acceptance

Existing local baselines, including the user's collection updates, are
unchanged. All six imported Markdown baselines pass. The imported Linux
`markdown-narrow` baseline differed on macOS by 3,369 pixels, maximum channel
delta 3. The unchanged original branch produced the same failure and a
byte-identical new image, isolating the difference from integration changes.
Only that imported baseline was refreshed for this renderer, after review;
the other five imported images were retained.

Ran the rebuilt native showcase in Orange/light and Blue/dark. Click and Space
toggle the Secondary preview's visible edge. Opening Markdown after scrolling
another page starts at its heading. Confirmed report-only links, heading
demotion, native fragment scrolling, table/header styling, and live source
replacement. A nested-emphasis example preserved its outer bold text; CodeBlock
showed the edited source and native Copy feedback. The headless test also
asserts the complete `CopyText` command. The preview was closed afterward.

## Package, preservation and limits

The actual crate archive contains 612 files. All 608 ordinary files match the
checkout byte for byte; Cargo's generated manifests/lock metadata and VCS
receipt were checked separately. Approved paths include Markdown sources/tests,
issue records, guides, README screenshots, font files and licenses. Generated
snapshot differences, media/storyboard outputs, target directories, IDE settings
and private tool files are absent. Relative documentation links pass. Changed
Rust files stay below 400 lines; the existing 448-line data-table test is
unchanged. The package receipt and check logs are retained locally; this archive
qualifies Unreleased content, not a release.

Existing staged `.claude/settings.json`, untracked `.idea/`, screenshots and
media/storyboard work are preserved; all 772 protected media/storyboard files
match their saved hashes. No branch/worktree cleanup was performed
because there was no push; the Markdown branch still contains unmerged history.

This run establishes macOS native behavior, headless renderer evidence and
WASM compilation. It does not establish browser execution, Windows/Linux native
acceptance or screen-reader acceptance. Markdown syntax and selection limits
remain explicit in [the component guide](markdown.md). The branch's historical
Linux qualification is retained [separately](verification-markdown.md).
