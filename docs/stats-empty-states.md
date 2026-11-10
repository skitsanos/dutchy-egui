# Stats and empty states

## Statistics

`Stat` is a non-interactive Widget accepting a label and an already formatted
value. The host owns aggregation, numeric policy, currency/locale formatting,
missing values, reporting periods, and updates. Strings such as "$48,290",
"0.80%", or "—" are preserved exactly.

```rust
use dutchy_egui::{Stat, Trend, TrendDirection, TrendSentiment};

fn error_rate(ui: &mut egui::Ui) {
    ui.add(Stat::new("Error rate", "0.8%")
        .accented(true)
        .trend(Trend::new("-18%", TrendDirection::Down)
            .sentiment(TrendSentiment::Positive)
            .comparison("fewer failed requests"))
        .description("Last 30 days"));
}
```

Standalone stats use a four-point border and 24-point padding (the web grows
to 32 above 640 px). `accented(true)`
replaces that border with a four-point left brand edge. Four `StatVariant`
surfaces match the web grid: Light, Muted, Dark (inverted), and Primary. Inverse
foregrounds apply to values, labels, descriptions, and neutral trends. Explicit
positive/negative trend colors remain semantic status colors.

`StatSize::Default` uses 36/48-point display values according to available content
width (the web steps 30/36/48 by viewport); Small uses 30. Dark/Primary labels
keep full inverse opacity rather than the web's 60/70 % (DS-003). Labels wrap rather than truncate. Default IDs derive from
the label; use `id_salt` for duplicate/changing labels within one parent scope.
Metrics expose a named group with formatted value and description/trend context.
They do not add Tab stops, click actions, live announcements, or animation.

Trends keep three independent inputs: formatted change text, Up/Down/Flat
direction, and Positive/Negative/Neutral sentiment. Neutral is the default;
direction or a numeric sign never determines sentiment. Visible direction words
and change text supplement decorative arrows, and accessibility includes the
explicit sentiment. `comparison` adds a wrapped contextual line. The arrow is
a single chevron 6 points before the text (the web draws a double chevron and
colors every non-negative change success by default).

```rust
use dutchy_egui::{Stat, StatsGrid, StatsLayout};

fn overview(ui: &mut egui::Ui) {
    let values = [Stat::new("Users", "2,847"), Stat::new("Views", "14,209")];
    ui.add(StatsGrid::new("overview", &values)
        .accessible_name("Workspace overview")
        .columns(2)
        .layout(StatsLayout::Compact));
}
```

StatsGrid supports up to four preferred columns and reduces them using shared
minimum column widths (200 regular, 160 compact). It uses two-point dividers and
equal tile height within each row. Regular layout retains the preferred column
slots on its final partial row. Compact adds an enclosing two-point border,
16-point padding, centered 24-point values, and smaller bold labels. Unlike the
web's compact branch, it retains explicitly supplied surfaces, trends, and
descriptions, and wraps at 160 points where the web keeps one row.
The grid's `size` supplies a shared regular scale instead of per-stat `size`.
Stable metric IDs survive responsive reflow and inserted siblings.

## Empty states

`EmptyState` is a presentation builder returning independent primary
and secondary action results. It does not navigate, create data, clear filters,
remove itself, or move focus automatically.

```rust
use dutchy_egui::{EmptyState, EmptyStateVariant};

fn results(ui: &mut egui::Ui, filtered: &mut bool) {
    let reset = ui.button("Apply sample filters");
    if reset.clicked() { *filtered = true; }
    if *filtered {
        let result = EmptyState::new("search-empty", "No results found")
            .variant(EmptyStateVariant::Search)
            .description("Try another search term or clear the current filters.")
            .action("Clear search")
            .announce(true)
            .show(ui);
        if result.action_clicked {
            *filtered = false;
            reset.request_focus();
        }
    }
}
```

Generic is centered guidance with no default icon. Search uses a search symbol
and more vertical space. Table is a compact empty body with inbox symbol and
secondary-colored primary button; the host supplies table structure and headers.
Dashboard adds a dashed border, larger icon/heading, and wider description.
`icon` overrides/removes the default through `EmptyStateIcon`.

Guidance wraps at a bounded text width. The description sits 24 points above
the actions, item spacing included; the table variant keeps the shared 24/8/32
rhythm rather than the spec's 16/4/16, and the title shares the paragraph's
text clamp. Actions are centered side by side when they fit, otherwise stacked. Both reuse native Dutchy buttons at the shared
Small size. `action_enabled` and `secondary_action_enabled` disable independently;
`enabled(false)` disables all controls and never enables a disabled parent.
`response` covers the container; title/body clicks do not activate its controls.

The required stable ID preserves action identity as content and width change.
The title exposes heading level 3. Icons are decorative and hidden from
accessibility. By default the container is a named group with live announcements
off; `announce(true)` opts into a polite status when replacing expected content.
Appearance and guidance updates never take focus. After replacing an empty state,
the host chooses a focus destination that remains present, as in the example.

## Reference decisions and limits

The current StatsGrid component is the primary surface/type reference; standalone
border/accent and detailed context follow the KPI documentation. Values precede
labels consistently. Generic/search/table/dashboard empty layouts combine the
component and documented patterns; action geometry reuses shared control tokens.
The square icons are small original painters, with no icon/helper dependency.

All data and actions in the showcase are deterministic local examples. Production
applications supply real work and content. Source-palette contrast limitations
remain in [DS-003](issues/DS-003.md), including small semantic trend text and some
on-primary text.
