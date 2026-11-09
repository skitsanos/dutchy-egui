# Verification — richer menus

Historical DS-073 qualification before the follow-up review. See
[DS-074/075 verification](verification-menu-polish.md) for the isolated native
Tab cause, corrected cascades and current check results. The totals and native
limitation below describe the earlier source state.

Date: 2026-11-08. Local work after main `5d68a93` (1.0.25), tracked in
[DS-073](issues/DS-073.md), on `ticket/ds-073-richer-menus`. No version bump,
commit, push or registry publication. Cargo.toml/Cargo.lock are unchanged;
no runtime/development dependency or borrowed implementation was added.

## Behavior and review

Checked actions return a typed proposed value without changing host data.
Submenu branch keys never execute. MenuBar binds host-controlled top-level
visibility; Toolbar menu buttons return leaf commands through ToolbarEvent.
The Menus/Toolbars showcase shares grid settings and export commands and can
decline checked requests. Flat menus and their `.action` results remain
compatible. CommandPalette remains a flat-command API and omits branch entries.

Sixteen added tests cover checked acceptance/rejection, three-level keyboard
navigation, nested Escape/Left, Tab/Shift+Tab exit, outside clicks, disabled and
empty branches, stable IDs under label/reorder changes, dynamic focus repair,
menu switching/reopening/hover, shared toolbar events, vertical toolbar keys,
native metadata/child-menu containment and narrow viewport placement. Tests
include Tab from a nested menu into an adjacent Toolbar inside a ScrollArea.

Review found and fixed:

- Escape discarded focus before a nested menu could dispatch it. Menu rows now
  reserve Escape along with both arrow axes.
- Hidden/reopened menus retained a previous top-level switching request during
  their sizing pass. Requests reset per render; reopening also resets the
  dropdown's transient opening state after an absence.
- Inactive roving controls could not accept explicit assistive focus. Shared
  action controls honor native Focus requests and register the requested item
  while preserving the composite's single Tab stop.
- Submenu rows appeared beside an empty Menu node in native accessibility.
  A real child Menu scope now contains its rows and is controlled by its branch.
- Root menus overlapped menu-bar padding; anchors now extend to the bar's edge.
  Labels clip within the column reserved beside checks, hints and arrows.

No lint suppression was added. Changed Rust files remain below 400 lines and
Clippy's cognitive-complexity threshold remains 15.

## Automated qualification

macOS 27.2/arm64, Rust 1.99.1. All commands below pass with no failed or ignored
tests. Private receipts retain exact commands, times, totals and source hashes.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 506 passed |
| `cargo test --no-default-features` | 355 passed |
| `cargo clippy --all-targets -- -D warnings` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | Pass |
| Strict all-feature rustdoc (`RUSTDOCFLAGS='-D warnings'`) | Pass |
| All-feature/minimal WASM library checks | Both pass |
| Fonts-only all-target check and rendering/CodeBlock tests | Pass; 12 tests |
| Parser-only library/Markdown/CodeBlock tests | 48 passed |
| `cargo +1.95.0 check --lib --all-features --locked` | Declared MSRV passes |
| `cargo build --example showcase --all-features` | Pass |
| `cargo package --allow-dirty --locked --all-features` | Isolated archive build passes |

The required checks were refreshed after adding the final nested Tab regressions.
An earlier run was stopped when native review found the submenu containment
defect; only results against the corrected source are counted here.

## Rendering and native observations

Nine new baselines show root, nested-export and checked menus in Orange/light
1x, Orange/dark 1.5x and Blue/light 2x. All 119 previous snapshots/screenshots
remain byte-identical. New views were reviewed for sharp borders, column/arrow
clearance, popup placement, checked-state visibility and contained preview paint.

Ran the rebuilt native Glow showcase in Orange/light and Blue/dark. Verified
View-menu changes update the toolbar, the toolbar updates the same host value,
declined changes retain it, keyboard opening enters the third export level,
Escape returns one level and Toolbar SVG selection reports a request then
returns focus to its trigger. The corrected native tree contains nested Menu
nodes with their actual command descendants.

One initial native Tab action returned to the preceding inline action. Repeating
the same nested sequence reached the following toolbar's Save button as expected.
The two nested/composed Tab regressions also pass; no repeatable cause was
isolated. This observation is retained as a native qualification limit rather
than evidence of complete focus-routing parity. The task-owned preview was
closed after verification; no unrelated app was closed.

## Package and limits

The actual archive is checked against root-approved paths and every ordinary
file's current bytes. Generated manifests, the lock subset and VCS metadata are
verified separately. Fonts/licenses, new guides, ledger, sources, tests and nine
goldens are included; media/storyboard, IDE/tool settings and generated snapshot
differences are absent. All 779 protected file hashes are unchanged. The package
receipt is retained with the local check logs.

This establishes macOS native observations, headless renderer/interaction
evidence and WASM compilation, not browser execution, other native platforms
or screen-reader acceptance. The [component contract](richer-menus.md) records
scope: two-state checks close after selection; disabled entries are skipped;
shortcut hints do not bind keys; radio/mixed checks, mnemonic/type-ahead input,
OS menu bars and arbitrary embedded menu widgets are outside this milestone.
