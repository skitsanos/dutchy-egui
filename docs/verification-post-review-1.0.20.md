# Verification follow-up — 1.0.19 and working 1.0.20

Date: 2026-11-07. Reviewed local main at `facb01c` (Scribble smoothing), plus
the staged and unstaged 1.0.20 form primitives and DS-047/DS-048 changes.
This is local qualification; no commit, push or publication was performed.

## Findings and remediation

The original working tree passed its required checks and 392 all-feature tests.
Additional lifecycle, arithmetic and rendering regressions exposed five gaps:

| Ledger | Reproduced problem | Local correction |
| --- | --- | --- |
| [DS-049](issues/DS-049.md) | Disabled typed inputs still stepped during focus loss; an open calendar survived disabling | Guard stepping with enabled editor focus; close disabled pickers |
| [DS-050](issues/DS-050.md) | Inserting a sibling lost slider focus | Give the native slider a persistent child scope |
| [DS-051](issues/DS-051.md) | Calendar entry/navigation reached disabled days or lost focus during sizing | Clamp dates and month navigation; defer initial focus until after sizing |
| [DS-052](issues/DS-052.md) | Extreme date/time offsets overflowed; rounding a finite large number produced infinity | Check day bounds/addition, widen time intermediates, guard scaled rounding |
| [DS-053](issues/DS-053.md) | Month arrows were invisible in the native calendar | Use the shared indicator size before applying the navigation inset |

Each defect was reproduced before its fix. New regressions live in
`tests/primitive_lifecycle.rs`, `tests/calendar_limits.rs` and
`tests/numeric_limits.rs`; they also cover segmented focus stability and
controls under a covering modal, which already worked.

Scribble documentation now distinguishes frame timing from changes to the
sample stream, documents collapsed tails shorter than 0.5 drawing units, and
states that the 16-segment flattening cap can exceed the target tolerance on
large jumps. The intended smoothing algorithm is unchanged.

## Automated qualification

Final results are recorded in `target/review-new-work/qualification.log`:

- `cargo fmt --all --check`.
- `cargo test --all-features`: 406 passed; no failures or ignored tests.
- `cargo test --no-default-features`: 302 passed; no failures or ignored tests.
- All three required Clippy configurations with `-D warnings`.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`.
- WASM library checks with all features and without default features.
- Native showcase build with all features.

Rendering covers light/orange 1x, dark/orange 1.5x, light/green 2x, the open
calendar and desktop/narrow showcase layouts. These images were inspected.
Scribble's existing curved-sample, capacity, history, export and live/committed
render comparisons also pass. The final suite runs without snapshot-update mode.

Only `primitives-calendar-1x.png` was refreshed during this review: 48 pixels
restore the two chevrons. The prior image is retained under
`target/review-new-work/primitives-calendar-before.png`. All tracked snapshots,
including the already modified DS-047 baselines, match their task-start hashes.
The staged patch is byte-for-byte unchanged; no starting file was removed.

## Native macOS verification

Ran a fresh all-feature showcase at Retina scale. Checked light/orange,
dark/blue and dark/green presentation, centered number/date/time rows and the
calendar anchored below the whole field. Switch click/Space, Tab past disabled
controls, segmented Left/End behavior, slider Right from 40 to 45,
number-button stepping and typed decimal entry updated the host correctly.
Calendar entry focused the selected day; Right/Enter selected the next date,
Escape returned to the trigger, month buttons changed months, and Previous
disabled at the bounded month. Time seconds wrapped 23:59:59 to 23:59:00.

With Scribble smoothing enabled, a native pointer drag committed ink; Undo,
Redo, local preview and scrolling to the shared read-only copy worked. This
drag verifies desktop operation, not handwriting quality. Curved sample
geometry and release continuity are covered by automated rendering tests.

## Limits

No Windows/Linux native, browser runtime, screen-reader, stylus or touch-device
acceptance was performed. Existing feature limits in the form and Scribble
guides remain. Reversed host date bounds are invalid configuration. This
review adds no dependencies and does not claim full cross-platform parity.
