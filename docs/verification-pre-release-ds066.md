# Pre-release review after DS-066

Date: 2026-11-08. Base commit: `cd2688fb8f5d8b42ad9d23ea73970491a4f6f79e`
(1.0.23), plus the current local work through DS-066. This is a review, not a
release. This historical review preceded the DS-067–069 fixes and the approved
Markdown integration; see [current qualification](verification-fixes-markdown.md).
Existing production source, README, snapshots, staged settings and IDE
files are preserved. New ledger entries record the remaining work.

## Findings

- **P2 / [DS-067](issues/DS-067.md):** MediaCard title text still enters the
  arrow column inside a right-aligned host. A 200-point card at 1x ends its title
  at 274.40625 against an arrow starting at 258: 16.40625 points of overlap.
  The DS-066 default-left case passes. Center alignment loses almost all of the
  intended gap. The allocation, rather than only the wrap measure, must reserve
  the arrow slot.
- **P2 / [DS-068](issues/DS-068.md):** Secondary toggles have identical on/off
  painting. A same-ID/same-name host-value update changes AccessKit to
  `Toggled::True`, but every painted shape remains identical. The public
  Secondary variant and shared toolbar renderer need a distinct state cue.
- **P3 / [DS-069](issues/DS-069.md):** README's blanket Widget/Response and
  dependency claims need qualification; earlier review status and verification
  wording need to distinguish historical evidence from current state.

The existing suite does not cover the two P2 cases. Two additional diagnostic
tests in an ignored, isolated probe crate reproduce them and fail as expected;
they do not alter or disable repository tests. The fixes and permanent
regressions remain outstanding. Resolve the P2 findings before release.

## Automated qualification

Current repository checks pass:

- `cargo fmt --all --check`.
- `cargo test --all-features`: **456 passed**, 0 failed, 0 ignored.
- `cargo test --no-default-features`: **337 passed**, 0 failed, 0 ignored.
- `cargo clippy --all-targets -- -D warnings`, repeated with `--all-features`
  and `--no-default-features`: all pass.
- Strict all-feature rustdoc, all-feature and no-default `wasm32-unknown-unknown`
  library checks, and the all-feature native showcase build: all pass.
- The all-feature library also passes `cargo +1.95.0 check --lib --all-features
  --locked` on the declared minimum compiler. The main matrix used Rust 1.99.1.

An isolated `cargo package --allow-dirty --locked --all-features` verification
and byte-for-byte archive comparison with the checkout pass. The new sources,
guides, README screenshots and font notices are included; media, target outputs,
IDE/editor settings and generated snapshot differences are excluded. These
package gates do not waive the two functional findings.

Private `.git/codex-maintenance/review-ds066-*` receipts contain command logs,
the original-file hash manifest and diagnostic output. No registry publication,
version bump, commit or push is performed. A future main push must advance
Cargo.toml/Cargo.lock and release the Unreleased changelog entries.

## Runtime and rendering

Rebuilt and ran the native macOS showcase. Confirmed ragged Color picker and
MediaCard copy, consistent Native additions kickers, and a new Icon buttons page
opening at its heading after scrolling Color picker. MediaCard save and preview
remain independent; Escape dismisses the lightbox. Default toggle state changes
and toolbar arrow-without-activation, Enter/Space actions, Tab exit and remembered
Shift+Tab reentry work. Reviewed Orange/light and Blue/dark; closed only the
task-owned preview. Main rendering tests verify the tracked themes/scales.

The title/alignment and Secondary-variant findings above are isolated harness
evidence, not claims that the current showcase happens to demonstrate those
configurations. No Windows/Linux native or screen-reader acceptance run, and no
browser runtime parity claim.

## Release housekeeping

README is currently a 118-line landing page with two checked-in screenshots.
The staged `.claude/settings.json` and untracked `.idea/` files need an intentional
staging decision before a release commit; this review leaves both untouched.
Generated probes/previews live under ignored target paths. Media/storyboard work
is outside release/package scope and is preserved.

Remote comparison also found unmerged Markdown/CodeBlock work at `684a840` on
`gedankrayze/vibrant-cerf-rat7ho`, diverging from main at 1.0.22. No open PR
exists at review time. It duplicates version 1.0.23 and ticket DS-058 and carries
the old package patterns. [DS-070](issues/DS-070.md) records the integration
decision and safeguards needed if it belongs in the next release. The branch
is preserved; it was not merged or qualified as part of the current tree.
