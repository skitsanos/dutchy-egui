# StatusBar

`StatusBar` is a one-line strip of keyed segments, usually at the bottom of a
window: counts, sync state, progress and keyboard hints. The application owns
every value and formats it; the bar lays out, hides what does not fit and
reports activation. No helper crate or hidden state is involved.

```rust
use dutchy_egui::{egui, StatusAlign, StatusBar, StatusItem, StatusTone};

fn status(ui: &mut egui::Ui, unread: usize, syncing: bool) {
    let sync = if syncing {
        StatusItem::labeled("sync", "Sync", "Checking mail").busy()
    } else {
        StatusItem::labeled("sync", "Sync", "Up to date").tone(StatusTone::Success)
    };
    let items = [
        StatusItem::labeled("unread", "Unread", unread.to_string()).priority(2),
        sync.clickable(!syncing).live(true).priority(1),
        StatusItem::hint("compose", "N", "Compose").align(StatusAlign::End),
    ];
    let result = StatusBar::new("mail-status", &items).show(ui);
    if result.clicked == Some("sync") {
        // The host starts a sync.
    }
}
```

Place it in an `egui::Panel::bottom` with no inner margin, or at the end of
any column. It fills the available width (640 points when unbounded) at the
32-point compact control height and paints a thin top divider; pass
`.divider(false)` when the host frame already draws one.

## Segments

- `text` shows a value; `labeled` a muted label and its value; `hint` a Kbd
  chip and a muted action. Text uses the mono label style on one line, with
  line breaks displayed as spaces.
- `icon` adds a leading icon. `tone` colors it, or adds a square marker when
  there is no icon. Text stays in the foreground color, so the words must
  state the status; color only repeats it.
- `progress(percent)` adds a 64-point determinate track (0–100 units);
  `busy()` an indeterminate one that respects reduced motion.
- `align(StatusAlign::End)` packs a segment against the right edge. Both
  groups keep item order, with 16 points between them at minimum.

Keys identify segments across frames. Keep them unique; reordering or
inserting items keeps focus on the same key.

## Overflow

When the segments do not fit, the lowest `priority` is hidden first and, among
equal priorities, the later item. Hidden keys are returned in item order in
`hidden`, so the host can show them elsewhere. If even the highest-priority
segment cannot fit alone, its value is elided with "…". If its label and
adornments leave too little space, the segment uses a compact line containing
its wording. A tooltip retains the full wording; accessibility keeps the
segment's role and value. If the compact line cannot fit, the key is reported
as hidden. There is no overflow menu.

## Interaction and accessibility

- `clickable(true)` makes a segment a button in the normal Tab order.
  Pointer clicks, Enter, Space and assistive Click actions set `clicked` to
  its key. Other segments are passive: they never take focus or clicks.
- `enabled(false)` disables one segment; `StatusBar::enabled(false)` all of
  them, as does a disabled parent.
- `tooltip` adds a hover and focus hint that is also the description.
- `segments` returns each visible segment's response, for anchoring a
  dropdown or context menu.

The bar is a named group (`Status` by default, see `accessible_name`).
Passive segments are labels whose text is the full content, such as
"Unread: 42", because AccessKit reads a label's text from its value.
Clickable segments are buttons named by their label with the value as their
value. Progress segments are progress indicators with a value range or a busy
state. Hints expose their chord as the keyboard shortcut. `live(true)` makes a
segment a polite live status; use it for state changes such as a finished
sync, not for counters that change every frame.

The **Status bars** showcase simulates a mailbox sync, a failure state and a
width control for overflow. Not included: overflow menus, resizable or
draggable segments, multi-line content, per-segment custom painting or a
window-level status service.
