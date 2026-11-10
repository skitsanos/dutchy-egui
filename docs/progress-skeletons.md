# Progress indicators and skeletons

## Progress

`Progress` is a non-interactive egui Widget. The application supplies values and
executes work; the widget does not advance a task or infer completion. Width
defaults to its available column, with sharp flat track/fill rectangles.

```rust
use dutchy_egui::{Progress, ProgressSize, ProgressVariant};

fn upload(ui: &mut egui::Ui, sent: f64, total: f64) {
    ui.add(Progress::new(sent)
        .max(total)
        .label("Uploading")
        .id_salt("upload")
        .show_value(true)
        .size(ProgressSize::Medium)
        .variant(ProgressVariant::Primary));
}
```

Small/Medium/Large tracks are 4/8/16 logical points. Fills are Primary, Success,
Destructive, and Foreground. Use meaningful labels as well as color to communicate
status. Labels wrap above the track; the optional right-aligned value uses mono
text and rounds to a whole percent. Primary/inverted card foregrounds apply to
both label and value. `accessible_name` overrides the name without adding text.
`id_salt` preserves identity when surrounding siblings or labels change.

The default maximum is 100. Painting and accessibility use the same sanitized
range: non-finite/non-positive maxima fall back to 100, NaN values become zero,
and values clamp to 0..=max (including negative/positive infinity). Sanitization
does not modify host state. Negative/non-finite explicit widths become zero;
finite widths are constrained to available space. Determinate fills update
immediately, with no stored transition or automatic repaint loop.

```rust
// When the application cannot estimate a total:
// ui.add(Progress::indeterminate().label("Discovering files").show_value(true));
```

Indeterminate progress exposes busy metadata and "In progress", with no numeric
completion/range. A quarter-width segment travels back and forth. Reduced motion
or `animate(false)` holds it at the center; this does not imply 25% completion.
Neither form creates a Tab stop or takes keyboard focus. Progress has no live
announcement loop; the host can use an Alert for a completion/failure message.

## Skeletons

`Skeleton` is decorative and hidden from assistive technology. Use a named
`SkeletonGroup` for one busy region rather than many per-block announcements.
The host switches from placeholders to real content when its own state changes.

```rust
use dutchy_egui::{Card, SkeletonGroup};

fn project(ui: &mut egui::Ui, loading: bool) {
    if loading {
        SkeletonGroup::new("project", "Loading project").show_card(ui);
    } else {
        Card::new().show(ui, |ui| { Card::title(ui, "Project ready"); });
    }
}
```

Presets in `SkeletonVariant` match the web component: Text (full width × 16),
Title (75% width × 32), Avatar (48 square), Card (full width × 192), and TableRow
(four cells with flexible leading columns). `count` repeats shapes vertically
with 12-point gaps; zero renders none. `width`/`height` override per-shape geometry.
Widths stay inside the column; negative/non-finite explicit dimensions become
zero. Table cells and gaps shrink for narrow columns.

```rust
use dutchy_egui::{Skeleton, SkeletonGroup, SkeletonVariant};

fn custom_placeholder(ui: &mut egui::Ui) {
    SkeletonGroup::new("article", "Loading article").show(ui, |ui| {
        ui.add(Skeleton::new().variant(SkeletonVariant::Title));
        ui.add(Skeleton::new().count(3).animate(false));
    });
}
```

Groups expose busy status and a polite live announcement; `announce(false)`
retains a named busy group with live announcements off. Groups and placeholders
add no keyboard stops and do not move focus. Stable group IDs preserve identity
when siblings are inserted.

Compositions: `show_card`, `show_list(ui, rows)`, `show_content`, and
`show_table(ui, rows)`. These are placeholders, without real data-table/list
behavior or virtualization. Use `show` to match an application's actual content.
Reserve a common minimum height or otherwise match both layouts when avoiding
layout shift matters; arbitrary loaded content is not guaranteed to match a
preset. The showcase demonstrates and tests a reserved-height transition.

## Motion and reference decisions

The pulse uses the live web stylesheet's two-second period (the prose docs say
1.5 seconds), with a smooth cosine approximation of its easing and opacity
between 1 and 0.4. Shared theme muted colors replace hardcoded light-only CSS.
Indeterminate travel uses a 1.5-second cycle. There are no gradients or blur.
Reduced motion freezes skeletons at full opacity; `animate(false)` pauses an
individual block. Pattern helpers follow the theme's reduced-motion preference.

Animations request a frame after 16ms only while their painted geometry is
visible. Clipped, empty, zero-size, paused, and reduced-motion indicators do not
run a continuous repaint loop. The application must request repaint when its
background work updates values or replaces content.

Source-palette contrast limitations remain in [DS-003](issues/DS-003.md).
