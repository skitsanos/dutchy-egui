# Verification — form primitives (1.0.20)

Date: 2026-11-07. Covers Switch, Kbd, Segmented, Slider, NumberInput,
DatePicker, TimePicker and the calendar values. Every control was written
test-first: each test failed against the tree without the control (or, for the
calendar arithmetic, against reference values computed independently) before
the implementation made it pass.

## Automated evidence

- `cargo fmt --all --check`; the three required `cargo clippy --all-targets`
  configurations with `-D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc`;
  WASM library checks with all and no default features.
- `cargo test --all-features`: 376 tests; `cargo test --no-default-features`:
  275 tests. No failures, no ignored tests, no snapshot artifacts.
- `tests/toggles.rs`: switch click, Space and Enter toggling, Switch role with
  toggled state, on/off paint (36 × 20 track, 12-point thumb colors and side),
  disabled switch; Kbd label role, height, muted fill, border and uppercase mono
  text; Segmented radio-group semantics, click, Left/Right wrap past a disabled
  segment, Home, equal segment widths inside the group border, foreground
  selection and background text.
- `tests/numeric_inputs.rs`: Slider role with value and bounds, arrow stepping
  that stays inside the control, rail click snapping to the step, 4-point rail
  and fill, 20-point handle at the half-way point, host-formatted label after
  the rail, disabled slider; NumberInput buttons and Up/Down stepping within
  bounds with disabled buttons at the minimum, typed parsing, invalid text
  keeping the value and reverting on blur, SpinButton value/min/max, square
  buttons inside the control.
- `tests/pickers.rs`: typed ISO dates commit, invalid text reverts on blur,
  emptied text clears; caret-segment stepping (End reaches the day, Home the
  year); calendar pointer selection closes the grid; Right, Down and PageDown
  move focus across days and months with the header following; Enter selects;
  Escape closes and returns focus to the button; bounds disable days and clamp
  typed dates; TimePicker typing, minute and hour wrap, seconds formatting.
- `src/calendar/tests.rs`: epoch day counts and weekdays for eight reference
  dates, leap years and month lengths, month/day arithmetic with clamping and
  roll-over, strict date and time parsing, segment wrapping.
- `tests/primitives_rendering.rs`: baselines `primitives-light-1x`,
  `primitives-dark-1_5x`, `primitives-green-2x` and `primitives-calendar-1x`
  (open month grid with today outlined and the selection filled); showcase
  Form controls page at 1200 × 1500 and 480 × 1900 with a valid accessibility
  tree, saved under `target/visual-proof/controls-{desktop,narrow}.png`.

## Reviewed images

All four baselines and both showcase renders were inspected. The review found
and fixed two defects before the baselines were kept: the number and date
editors inherited egui's 48-point minimum editor height, which pushed small
controls' text below the frame, and the showcase key-chip sentence wrapped
inside a wrapping row. The editors now match their slot height and the chips
sit in non-wrapping rows.

## Limits

Native macOS, Windows/Linux, screen-reader and browser runs were not performed;
the kittest harness exercises real egui focus, keyboard, pointer and
accessibility events. No range or vertical slider, read-only pickers, date
ranges, week numbers, locale formats, 12-hour clocks or time zones. The
showcase reads the system clock for `today` on native targets only; the
library reads no clock.

## Centering and calendar anchoring (DS-047, DS-048)

`tests/editor_alignment.rs` holds the text row of `Input` (Small, Medium,
Large) and `TimePicker` within 0.75 points of the frame center and keeps the
textarea row at the top; `tests/pickers.rs` checks at a 900-point width that
the calendar hangs below the field, starts under it rather than under the
button, and stays within the field's right edge. Centering moved text in 29
existing baselines, all refreshed and inspected for the text shift only:
`accordion-keyboard-focus`, `tab-roving-focus`, `disclosure-{light-1x,
dark-1_5x, purple-2x}`, `forms-{light-1x, dark-1_5x}`, `layout-{light-1x,
dark-1_5x, purple-2x}`, `menus-{light-1x, dark-1_5x, purple-2x}-{inline,
search, commands}`, `overlay-{light-1x, dark-1_5x, purple-2x}-{modal, left,
right}` and `overlay-close-hover`; the four `primitives-*` baselines were
regenerated with the centered time picker and the re-anchored calendar.

## Independent follow-up

The later [1.0.20 verification follow-up](verification-post-review-1.0.20.md)
records additional lifecycle and arithmetic regressions, fixes, current test
counts and native macOS checks. It supersedes this report's original totals
and native-testing limitation; other platform limits still apply.
