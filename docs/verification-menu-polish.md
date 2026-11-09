# Verification — menu focus and cascade review

Subsequent layout correction: [DS-076](issues/DS-076.md) and its
[verification report](verification-menu-layout.md) cover opening menus adding
an extra layout gap. The results below describe the preceding source state.

Automated qualification: 2026-11-08; native acceptance: 2026-11-09.
Follow-up to [DS-073](issues/DS-073.md), covering
[DS-074](issues/DS-074.md) and [DS-075](issues/DS-075.md). Local branch
`ticket/ds-073-richer-menus` still starts at main `5d68a93` (1.0.25).
No version bump, commit, push or publication; no dependency changes.

## Native reproduction and fix

Temporary tracing captured a real Shift+Tab exit from the third-level PNG row
to the following Toolbar Save control instead of the preceding inline Delete
action. The input batch contained Shift-down, Tab press/release with Shift,
then Shift-up. Frame modifiers were therefore NONE. The previous routing read
that final state and requested forward traversal. The opposite disagreement
also reproduces the earlier plain-Tab/backward observation deterministically.

Popup routing now reads the key event's modifier snapshot and consumes eligible
plain/Shift Tab presses once. Ctrl/Cmd/Alt Tab chords remain host-owned. Raw-event
tests cover both disagreements across Dropdown, MenuBar and Toolbar, including
a neighboring Toolbar as the next host control.

The review also found that immediate Escape after child focus could clear focus
inside egui before a new row's focus-lock filter took effect. A per-viewport
end-of-pass focus snapshot recovers only the descendant that actually owned
focus. Tests verify one-level return and that an external editor taking focus
cannot be displaced by stale menu focus. Temporary tracing is absent from the
library/showcase sources.

## Cascade review

Shared frame borders replace the menu-bar/cascade gutters. Child first rows
align with parent rows when screen space allows. Measured children use a
200-point minimum, bounded by configured width and viewport, while wrapped
parent labels keep stable regular-weight geometry. One input owner controls
the accent target; open ancestors use muted context and an accent arrow.
Hover does not steal an external editor's focus. Inline leaf activation closes
its descendants and returns to the root branch.

Seven polish tests cover pointer/keyboard ownership across three themes/scales,
child sizing, right and left shared seams, stable wrapped labels, inline leaf
dismissal, immediate Escape and external focus. Narrow viewport tests retain
native flipping/clamping. Nine rich-menu baselines were updated intentionally;
all 119 earlier snapshots/screenshots and 779 protected local files are unchanged.
See the [design rationale](menu-interaction-decisions.md) for the optional future
drill-down alternative and current very-narrow-window overlap limitation.

## Automated qualification

All 515 all-feature tests pass, including three rich-menu rendering tests and
nine new focus/polish regressions. The actual 645-file crate archive builds in
isolation and passes path, source-byte, manifest, lock and VCS inspection. It
includes fonts/licenses, docs and intended baselines, excluding media/storyboard,
IDE settings, tracing files and generated snapshot differences. Changed Rust
files remain below 400 lines with no added lint suppression. Relative docs links
and `git diff --check` pass.

All 14 automated gates pass on macOS/arm64, Rust 1.99.1. No failed or ignored
tests, no added dependencies or lint suppressions.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 515 passed |
| `cargo test --no-default-features` | 364 passed |
| `cargo clippy --all-targets -- -D warnings` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | Pass |
| Strict all-feature rustdoc (`RUSTDOCFLAGS='-D warnings'`) | Pass |
| All-feature/minimal WASM library checks | Both pass |
| Fonts-only all-target check and rendering/CodeBlock tests | Pass; 12 tests |
| Parser-only library/Markdown/CodeBlock tests | 48 passed |
| `cargo +1.95.0 check --lib --all-features --locked` | Pass |
| `cargo build --example showcase --all-features` | Pass |
| `cargo package --allow-dirty --locked --all-features` | Isolated archive build and audit pass |

## Native acceptance

After the Mac was unlocked, rebuilt and ran the native Glow showcase on
2026-11-09. All qualified source hashes still matched the automated run, so
no unchanged Rust checks were repeated. Sixteen recorded observations cover:

- Plain and Shift+Tab from third-level PNG, including repeated default presses
  and 120 ms presses: forward reaches Toolbar Save, backward reaches the preceding
  inline Delete action. After Toolbar Export became the remembered roving tool,
  forward correctly reached Export instead of Save. The dark-theme repeat
  preserved the same direction and remembered-tool behavior.
- Escape from PNG returns to Image format, then Export artwork, then File at
  the root. Immediate Right/Escape also returns one level in dark mode.
- Physical pointer clicks through both branches in Orange/light and Blue/dark:
  Save loses its active fill, the current branch alone highlights, ancestors
  stay muted, and content-sized child frames share aligned borders. Keyboard
  input takes over from the pointer without competing sibling highlights.
- Nested Toolbar SVG activation reports the host request and restores the
  Export trigger in both themes. Tab from a Toolbar submenu reaches Apply view
  changes; Shift+Tab returns to the preceding menu bar's File entry.

The corrected native behavior and original event capture now support resolving
DS-074/075 locally. Screenshots and native accessibility observations are saved
with the private maintenance receipts. The task-owned preview was quit,
confirmed no longer running, then its temporary bundle was removed.

Nine renderer baselines were visually reviewed across Orange/light 1x,
Orange/dark 1.5x and Blue/light 2x. This establishes headless rendering/interaction,
WASM compilation and the observed macOS native behavior, not browser execution,
other-platform or screen-reader acceptance. No release is performed. Private
maintenance receipts retain exact commands, logs, source hashes and the original
native trace. Task-owned snapshot differences, unused tracing source and the
closed preview bundle were removed. Protected media and unrelated apps are
preserved.
