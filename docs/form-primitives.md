# Form primitives

Switch, Kbd, Segmented, Slider, RangeSlider, NumberInput, DatePicker and
TimePicker are controls a native form needs but the web registry never listed. They follow
the existing form conventions: values stay in application state, IDs are
stable salts within the parent UI, `Field` supplies
the visible label and validation decoration, and every control returns the
Response that carries its change, focus and blur events. The web stylesheet
defines none of these controls, so their geometry is a decision of this port,
built from the shared tokens: 32/48/56-point control heights, the 20-point
indicator side, 2-point borders, the 2-point focus outline with its 2-point gap,
zero radius and flat fills.

```rust
use dutchy_egui::calendar::{Date, Time};
use dutchy_egui::{
    DatePicker, Field, NumberInput, RangeSlider, Segmented, SelectOption, Slider, Switch,
};

struct Settings {
    notify: bool,
    density: Option<u8>,
    volume: f64,
    price: (f64, f64),
    retries: f64,
    issued: Option<Date>,
    today: Date,
}

fn show(ui: &mut egui::Ui, s: &mut Settings) {
    ui.add(Switch::new("notify", &mut s.notify, "Email notifications"));
    let densities = [SelectOption::new(0, "Comfortable"), SelectOption::new(1, "Compact")];
    Field::new("Density").show(ui, |ui| {
        ui.add(Segmented::new("density", &mut s.density, &densities).full_width(true))
    });
    Field::new("Volume").show(ui, |ui| {
        ui.add(Slider::new("volume", &mut s.volume, 0.0..=100.0)
            .step(5.0)
            .value_label(|v| format!("{v:.0} %")))
    });
    let (low, high) = &mut s.price;
    Field::new("Price").show(ui, |ui| {
        ui.add(RangeSlider::new("price", low, high, 0.0..=500.0)
            .step(10.0)
            .accessible_name("Price")
            .value_label(|low, high| format!("{low:.0}–{high:.0}")))
    });
    Field::new("Retries").show(ui, |ui| {
        ui.add(NumberInput::new("retries", &mut s.retries).range(0.0..=10.0).accessible_name("Retries"))
    });
    Field::new("Issued").show(ui, |ui| {
        ui.add(DatePicker::new("issued", &mut s.issued).today(s.today).accessible_name("Issued"))
    });
    let _ = Time::new(9, 30, 0);
}
```

## Decisions

- **Switch** is a 36 × 20 track with a 12-point square thumb. Off: background
  fill, border edge, muted-foreground thumb; on: primary fill, on-primary
  thumb; hover lifts the edge to primary like the checkbox. The thumb slides
  over the fast motion token and jumps under reduced motion. Role Switch;
  Space/Enter toggle through egui's native click activation.
- **Kbd** is a static mono, uppercase, extra-small chip on the muted surface
  with a thin border, 20 points tall. One chip per key; hosts compose
  sequences in a row. It is a label whose accessible name keeps the host's
  casing.
- **Segmented** is an exclusive choice drawn as one bordered row of equal
  segments at a shared control height; the selected segment fills with the
  foreground and shows background text, hovered or focused segments use the
  muted surface. It is a radio group: Role RadioGroup with RadioButton
  segments, one tab stop, Left/Right/Home/End roving that also selects,
  wrapping across enabled segments. Options reuse `SelectOption`.
- **Slider** wraps egui's slider for drag, arrow-key stepping, smart aiming and
  the Slider accessibility role with its numeric value and bounds, under Dutchy
  paint: a 4-point rail in the border color with a primary trailing fill and a
  20-point square primary handle (foreground while hovered or dragged). The
  rail egui would paint is zero height; the Dutchy rail and fill are inserted
  beneath the handle. The value label is optional, host-formatted and sits in
  a 64-point column, 12 points after the rail. Rail, gap and column together
  take exactly the width the control was given.
- **RangeSlider** wraps egui's range slider and edits a low and a high value
  on the same rail, with the primary fill between two square handles. A press
  or drag on the rail moves the nearer handle and keeps it for the whole
  gesture; the handles may meet but never cross, and `min_separation` keeps
  them apart. Values out of range or out of order are pulled back in. Each
  handle is its own tab stop with Left/Right stepping and a Slider node whose
  bounds end at the other handle. The rail is a named Group. Handles are named
  after `accessible_name` with "minimum" and "maximum"; `handle_names`
  replaces both, for example in another language. A Field can only name the
  rail, so set `accessible_name` inside a Field too. The optional label
  formats both values in a 96-point column. The returned Response belongs to
  the rail: it reports changes and the drag, not the focus of a handle.
- **NumberInput** is a single-line editor in the input frame with square
  decrease/increase buttons the height of the control, Up/Down stepping while
  the editor has focus, optional bounds and decimals, and the SpinButton role
  with numeric value, minimum, maximum and step. Stepped values round to the
  decimals; typed and stepped values clamp to the range. The step buttons are
  named "Decrease"/"Increase" followed by the accessible name, so supply one
  when several number inputs share a view.
- **DatePicker** is an ISO `YYYY-MM-DD` editor with a calendar button that
  opens a month grid below the whole field, aligned with its start (egui's
  alternative placements apply near viewport edges): previous/next month,
  weekday header starting Monday,
  six rows of day buttons with adjacent-month days muted, today outlined,
  the selection filled with primary, and optional bounds that disable days
  and clamp typed dates. In the grid, arrows move by day and week, Home/End
  reach the week's ends, PageUp/PageDown move a month, Enter or Space selects
  and closes, Escape or a click outside closes, and focus returns to the
  calendar button. In the editor, Up/Down step the year, month or day under
  the caret. The library reads no clock: pass `today` from the host to outline
  it and to anchor an empty picker; without it the anchor is the value, a
  bound, or 2000-01-01.
- **TimePicker** is an `HH:MM` or `HH:MM:SS` editor; Up/Down step the segment
  under the caret and wrap within it, so 23:59 plus a minute is 23:00. Parsing
  is strict two-digit segments. Time zones stay with the host.

The number, date and time editors share one typed-value editor: while the
editor has focus the typed text lives in egui's temporary memory, each change
parses it and commits only a valid value (an emptied date or time commits
None), and losing focus reformats the text from the committed value. A host
that needs to reject out-of-range input instead of clamping validates through
`Field` and its own rules.

## Calendar values

`calendar::Date` covers years 1 to 9999 in the proleptic Gregorian calendar
with civil arithmetic (`add_days`, `add_months` clamping the day,
`days_since_epoch`, `weekday`) and strict ISO parsing and display.
`calendar::Time` is a 24-hour clock time with per-segment wrapping steps and
`HH:MM`/`HH:MM:SS` text. Both are plain values a host converts to and from its
own instants; no date crate is added.

## Boundaries

No vertical or logarithmic slider, week numbers, date ranges, locale-specific
formats, 12-hour clocks or time zones. Pickers have no read-only mode yet;
disable them instead. Slider label columns have fixed widths, so a Slider
and a RangeSlider stacked with labels end their rails 32 points apart. The
showcase derives `today` from the system clock on
native targets only.
