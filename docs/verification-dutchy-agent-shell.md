# DutchyAgent header and sidebar verification

Date: 2026-11-09. [DS-101](issues/DS-101.md), on
`ticket/ds-099-dutchy-agent`, targeting the unreleased 1.0.30 scope. Package
version remains 1.0.29; no commit or push is part of this change.

## Changes and scope

The header now separates the title/muted provider badge from right-aligned
New chat and settings actions. Narrow windows use a named plus icon for
New chat. The settings glyph is an original, application-owned sliders painter
inside the library's IconButton; tooltips, focus and keyboard semantics come
from the existing control. No dependencies were added.

Conversation rows keep full-width hit targets and a consistent left inset.
`Button::text_align(egui::Align)` adds padded left/center/right label alignment;
center remains the default. Loading spinners follow the aligned label group.
Long labels retain native truncation and complete accessible names.

## Focused evidence

The shell regression checks 480, 560 and 1160-point widths in both themes at
1x, 1.5x and 2x. It asserts right-edge placement and equal action centers,
left-aligned short/long/selected conversation labels, full-width row targets,
settings Escape/focus return, keyboard New chat and accessibility.

A separate Button geometry regression checks all alignments with/without
loading, spinner position and long-label truncation. Nine Agent rendering
baselines are updated intentionally for the header/sidebar; component
baselines retain their previous centered button appearance.

Native macOS Retina 2x inspection passed in light/dark themes. A mock-only
conversation and a fresh selected conversation both showed left-aligned names;
the long title truncated within its row. The settings icon opened its drawer,
displayed its tooltip and regained focus on Escape. The task preview used a
process-only empty key override, preserving the real `.env`; no live API turn
was made for this presentation change. The preview was closed afterwards.
Native resizing and screen-reader output were not exercised; narrow layouts
and accessible roles/actions have kittest coverage.

## Qualification

All 21 local gates pass on macOS arm64, Rust/Cargo 1.99.1:

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 630 passed, including 33 Agent and 36 Todo tests |
| `cargo test --no-default-features` | 406 passed |
| Clippy: default, all features, minimal | All targets, `-D warnings`; all pass |
| Strict rustdoc | All features, `RUSTDOCFLAGS='-D warnings'`; pass |
| WASM library | All-feature and minimal profiles pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| MSRV 1.95.0 | Locked all-feature library check passes |
| Native examples | All-feature build passes |
| Workflow and shell | actionlint, ShellCheck and shell syntax pass |
| Relative documentation links and whitespace | Pass |
| Actual crate build and inspector | 778 files; approved paths, matching bytes, notices and packaged links pass |

The required Clippy commands run as
`cargo clippy --all-targets [feature flags] -- -D warnings`. Final qualification
has no failed or ignored tests. Source/configuration/asset hashes remained
unchanged throughout the final matrix; logs and native receipts are retained
privately under `.git/codex-maintenance/dutchy-agent-shell/`. The archive is
rebuilt and inspected after final documentation updates. The ignored `.env`,
user media and unrelated processes are preserved. Hosted CI awaits an authorized push.
