# Tooltips and layout

Version 1.0.5. The reference is the same web revision recorded in [plan](plan.md),
reviewed against `components/tooltip.md`, `components/flex.md`,
`components/layout.md`, and the Tooltip/Flex implementations on 2026-11-06.

## Tooltip

```rust
use dutchy_egui::{Button, Tooltip, TooltipPosition};

fn actions(ui: &mut egui::Ui) {
    let response = ui.add(Button::new("Save"));
    Tooltip::new("Save changes to this workspace.")
        .position(TooltipPosition::Top)
        .show(&response);
    if response.clicked() {
        // The application saves its state.
    }
}
```

Top is the default; Bottom/Left/Right are preferred placements. egui flips and
constrains the popup at viewport edges. Text wraps within a maximum 256-point
text width, reduced to the viewport width minus 32 points. The inverted
foreground fill, background text, 12-point medium face with 0.02 em letter
spacing, 1.4 leading, 12/6 padding, 8-point gap, and 5-point arrow follow the
live stylesheet (`src/styles/dutchy.css`); the Markdown spec's 10-point padding
and uncapped single line are not used. There are no interactive tooltip children.

Keyboard focus presents help immediately. Hover uses egui's configured delay,
grace period, scrolling/dragging and popup suppression. Disabled controls support
pointer help without adding a keyboard stop. Fully clipped controls do not show
popups. Escape dismisses hover help until both focus and hover leave; the key is
not consumed. Escape pressed during egui's hover delay is not remembered, so a
hint can still appear once that delay elapses. Native focus policy applies: egui clears button focus on Escape.
Tooltip rendering itself neither requests nor steals focus.

The control's accessible description includes the hint even while the popup is
closed; existing field guidance is retained. Its name stays unchanged. Attach the
tooltip after `Field::show` so field metadata is already present. Keep essential
instructions and disabled-state explanations visible. A tooltip does not name
an otherwise unnamed control or make static content focusable.

## Layout composition

```rust
use dutchy_egui::{Button, Container, ContainerWidth, Flex, Spacing};

fn toolbar(ui: &mut egui::Ui) {
    Container::new("toolbar-container")
        .max_width(ContainerWidth::Xl4)
        .show(ui, |ui| {
            Flex::row("toolbar").wrap(true).gap(Spacing::Lg).show(ui, |ui| {
                let _ = ui.add(Button::new("Projects"));
                let _ = ui.add(Button::new("Settings"));
            });
        });
}
```

`Flex::column` stacks vertically. `FlexDirection` supports row/column/reverse;
reverse columns need a bounded host height. `CrossAlign` maps Start/Center/End/
Stretch onto egui's cross-axis alignment/justification. Stretch is native sizing,
not CSS equal-height layout. Wrapping is supported for rows; column wrapping,
baseline alignment, and CSS space-between/around/evenly are not implemented.
Compose native egui controls/spacers when those patterns are needed.

`Spacing` covers 0/2/4/6/8/12/16/20/24/32/40/48/64 points. `ContainerWidth`
covers 384/448/512/576/672/896/1152/1280 points or Available. The maximum includes
horizontal gutters: 16 points by default, 24 at viewport width 768+, or an
explicit `Spacing` override. Containers add no vertical padding. Children retain
egui's sizing contract and must respect the width offered to them.

```rust
use dutchy_egui::{Card, ResponsiveGrid};

fn projects(ui: &mut egui::Ui, names: &[String]) {
    ResponsiveGrid::new("projects").columns(3).show(
        ui, names, |name| name.as_str(), |ui, name| {
            Card::new().show(ui, |ui| { Card::title(ui, name); });
        },
    );
}
```

Use a durable project ID as the key in real applications; keys must be unique
and stable. The grid renders once in row-major order and preserves child ID scopes
through reordering, sibling insertion, and column changes. Default columns: at
most three, minimum 280 points, gaps 24. Typed minima also include 160/240/320/336.
One column may shrink below the minimum; zero maximum columns means one. An
empty slice calls no renderers. A partial final row keeps the preceding column
widths. Row height follows its tallest child; painted tiles need not be equal
height. Cells lay out top-down and left-aligned: egui's column justification is
not applied, so wrapped copy stays ragged and children fill a cell through their
own width options (DS-063). There is no virtualization or two-pass child
measurement.

Layout helpers preserve normal control roles, keyboard behavior, and host state.
They do not add unnecessary focusable wrappers. The showcase's Layout page
composes cards, fields, checkbox, stats, alerts, and tooltip help into workspace
settings/dashboard content. Its Forms page also uses the keyed grid. The host
owns values, validation, saving, navigation, and egui Panels. The web `Layout`
component's HTML/head/assets/SSR shell has no native equivalent in this library.

All implementations are maintained in this repository; no helper crates or
third-party widget source were added. See [verification](verification-layout.md).
