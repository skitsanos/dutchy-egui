# Verification — independent review of the richer-menu work (DS-073 to DS-076)

Shipped in release 1.0.26; see the [release record](verification-release-1.0.26.md).

Date: 2026-11-09. Subject: the uncommitted working tree of
`ticket/ds-073-richer-menus` (based on main `5d68a93`, version 1.0.25) that
implements the richer-menus guide (`docs/richer-menus.md`) and the follow-up tickets
DS-073, DS-074,
DS-075 and DS-076. The review also
checks how that work combines with the two other local branches waiting for
the same release: the DutchyTodo example (DS-077) and the
scroll-fade fix (DS-078). The review changed nothing in the
menus checkout and performed no commit, push or release there.

## Method

The 58 modified and untracked files of the checkout were copied into a scratch
worktree at `5d68a93` and committed there as a disposable snapshot (3696
insertions, 214 deletions). A byte comparison against the checkout showed one
difference outside that set, the Display font licence (DS-080). Every command
below ran fresh in the copy. The tickets, the contract, the
interaction decisions (`docs/menu-interaction-decisions.md`) and the three
verification records were read and their claims compared with the code and the
results. The nine new baselines were inspected. An independent read of the diff
covered focus and keyboard routing, identity, placement, accessibility, host
ownership and token use. Finally the three branches were merged in a
throwaway worktree to surface conflicts and run the gates on the combined tree.

## Automated results on the menus copy

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features --no-fail-fast` | 517 passed, 0 failed, 0 ignored |
| `cargo test --no-default-features --no-fail-fast` | 366 passed, 0 failed |
| `cargo clippy --all-targets -- -D warnings` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | Pass |
| `RUSTDOCFLAGS='-D warnings' cargo doc --all-features --no-deps` | Pass, no warnings |
| `cargo check --lib --target wasm32-unknown-unknown` (all features; minimal) | Both pass |
| `cargo +1.95.0 check --lib --all-features --locked` | Pass |
| `cargo build --example showcase --all-features` | Pass |
| `cargo package --list` | 648 files; no media, storyboard, IDE or snapshot-difference paths |
| Relative Markdown links | 145 files, 0 broken |
| `cargo check --all-targets --no-default-features --features bundled-fonts`; `cargo test --no-default-features --features markdown` | Pass; 389 passed |

The counts match the figures in DS-076 (517 all-feature and 366 minimal tests,
648-file archive). Changed Rust files stay under the 400-line soft cap (largest
`src/widgets/dropdown.rs` at 389 lines; largest new file
`src/widgets/menu_bar/mod.rs` at 310). The diff adds no `allow(` attribute and
leaves `Cargo.toml` and `Cargo.lock` untouched, so no dependency was added.

## Documentation review

The four tickets carry Problem, Remediation and Resolution under 500 words,
with evidence and explicit limits: radio and mixed checks, mnemonics and
type-ahead, operating-system menu bars, screen-reader acceptance and other
native platforms stay outside scope. The contract, the coverage rows, the
roadmap paragraph and every status line say "Unreleased" or "Resolved locally";
the release that ships this work must replace them with its version, the drift
DS-072 corrected after 1.0.24. The CHANGELOG has an Unreleased section with one
bullet per ticket. README gains only the MenuBar name. The coverage matrix
updates the Menu and Toolbar rows and adds MenuBar; `docs/index.md` links the
contract and the decision record; the menus and toolbar guides cross-link. The
earlier verification record is marked historical and points forward. One
pre-existing documentation defect sits next to the new MenuBar row: the blank
line that splits the matrix before the Markdown rows (DS-079).

## Rendering review

The nine baselines cover Orange/light at 1x, Orange/dark at 1.5x and Blue/light
at 2x, each with the root File menu, the three-level export cascade and the
checked View menu. They show the menu bar and root menu sharing a border, each
cascade level sharing its side edge with the parent and starting at the parent
row, content-sized children, one accent target with muted open ancestors,
regular-weight branch labels, visible check boxes on both highlighted and idle
rows, shortcut hints in on-primary ink on the active row, zero corner radius
and flat fills with no gradient or shadow. A small dark fragment to the right
of the root menu in the toolbar row is the trailing edge of the labelled
"Export artwork" toolbar menu button beneath the overlay, not a paint defect.
These baselines were rendered before the scroll-fade fix; the merged tree,
which includes that fix, reproduces all nine byte for byte, so no menu baseline
carried the gradient.

## Trial merge of the three branches

Merging the fade branch onto the snapshot conflicts only in the CHANGELOG
Unreleased section; both bullet lists were kept. Merging the Todo branch then
conflicts in `docs/component-roadmap.md` and `docs/index.md`: the roadmap
rewrite was kept with its DS-073 bullet replaced by an Unreleased line covering
DS-073 to DS-076 and linking the contract, and the index paragraph combines
both link lists. The merged documents have no broken links (149 files). The
Todo example still zeroes the scroll fade in its own theme setup, which the
library now does; the release merge should drop that override.
On the merged tree `cargo fmt --all --check` passes,
`cargo test --all-features --no-fail-fast` reports 544 passed (517 from the
menus work, 24 from the example, 3 from the fade fix) with no snapshot
change, `cargo test --no-default-features --no-fail-fast` reports 369 passed,
and all three Clippy configurations pass.

## Ledger numbering

The menus work owns DS-073 to DS-076. The DutchyTodo ticket, written as DS-074
on its branch, was renumbered to DS-077 with its links, and the scroll-fade
ticket moved from DS-076 to DS-078 with its branch renamed accordingly.

## Findings

An independent read of the diff produced seven items; the behavioral ones were
then reproduced with throwaway probe tests in the review copy before being
recorded. Three defects are confirmed and registered as open tickets:

- [DS-081](issues/DS-081.md): Tab from a menu attached to a host-drawn button
  with `Dropdown::show_for` closes the menu but leaves focus on that button,
  although the guide promises the next host control.
- [DS-082](issues/DS-082.md): once the pointer moves, the keyboard-focused row
  of an open menu paints neither fill nor focus ring while Enter still
  activates it (the row pixel changes from the primary fill to the surface).
- [DS-083](issues/DS-083.md): in a menu bar whose only other entry is
  disabled, Left or Right refocuses the trigger while the popup stays open with
  no focused row.

Later on 2026-11-09 all three were fixed in the richer-menus work with failing
regressions first; the resolved tickets live in that work and supersede the
open copies on this branch when the two merge.

Observations without a ticket: inside one Toolbar, Escape keeps focus on a
menu button but drops it from a plain tool (probe-confirmed, undocumented);
MenuBar triggers register widget information twice, first as a Button and then
as a MenuItem, so assistive technology receives two events per click or focus
change; `src/widgets/menu/render.rs` re-anchors a popup that its response
already anchors, and `src/widgets/dropdown.rs` assigns the item memory twice.
Coverage gaps: Up opening at the last entry works (probe-confirmed) but no bar
or toolbar test presses Up; the pointer path between a branch and its child is
never exercised; icon-only menu buttons have no geometry assertion; no test
presses Tab through a `show_for` trigger. The read found identity, placement,
accessibility roles, host ownership and the token rules respected: ids derive
from keys rather than labels, popup edges are computed in points with the
border inside the frame, no literal colors or radii appear, only host open
flags are written and values return as events. The Clippy gate at threshold 15
is authoritative for cognitive complexity and passes.

Two pre-existing defects outside the menus work are fixed on this branch:
[DS-079](issues/DS-079.md) (coverage matrix split) and
[DS-080](issues/DS-080.md) (Display font licence line endings).

## Limits

This review did not run the native showcase; the macOS pointer, keyboard and
accessibility observations rest on the menus session's own records. Windows,
Linux, browser and screen-reader behavior remain unverified. The review copy
differs from the checkout only by the licence file described in DS-080.
