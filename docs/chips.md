# Chips

`Chip` renders a compact square label. `selected` makes the body a toggle
button with the current selection exposed to assistive technology. `removable`
adds a separately focusable button named “Remove” followed by the chip label.
Both options can be used on the same chip.

```rust
use dutchy_egui::Chip;

fn show_filter(ui: &mut egui::Ui, selected: &mut bool, visible: &mut bool) {
    if *visible {
        let chip = Chip::new("draft", "Draft")
            .selected(*selected)
            .removable(true)
            .show(ui);
        if chip.remove_requested {
            *visible = false;
        } else if let Some(next) = chip.toggle_requested {
            *selected = next;
        }
    }
}
```

Use a stable key within the parent UI so focus follows the chip when nearby
items change. Tab visits the toggle body and then Remove when both are present.
Enter, Space, pointer clicks and assistive activation raise the corresponding
request. A removable chip with a passive label has one Tab stop. A disabled
chip, including one inside a disabled parent, rejects both actions.

Keep selection, removal, filter policy and focus after removal in the host.
The `remove` response identifies the remove button, and `response` identifies
the body. Chips fit in wrapping rows and shorten long visible labels to fit
while preserving their accessible names. Showcase's **Chips** page filters
documents by status and removes or restores project tags.

## Filter chip groups

`FilterChipGroup` lays out keyed toggle chips in a named accessibility group.
Rows wrap as space runs out, in item order, with the small spacing token
between chips and rows whatever spacing the parent sets. Give each choice a
unique key within its group.
Reordering or renaming a choice preserves its identity. Tab visits enabled
choices in item order, and Enter, Space, pointer and assistive activation
produce the same keyed request.

```rust
use dutchy_egui::{FilterChipGroup, FilterChipItem};
use std::collections::HashSet;

fn show_formats(ui: &mut egui::Ui, selected: &mut HashSet<&'static str>) {
    let formats = [
        FilterChipItem::new("drawing", "Drawings"),
        FilterChipItem::new("photo", "Photography"),
    ];
    let group = FilterChipGroup::new("formats", &formats)
        .accessible_name("Media types")
        .show(ui, |key| selected.contains(key));
    for request in group.requests {
        if request.selected {
            selected.insert(request.key);
        } else {
            selected.remove(&request.key);
        }
    }
}
```

The host supplies selection each frame and chooses which requests to apply.
A disabled choice displays its host-supplied selection and skips activation
and Tab. Showcase combines multiple media types with a single collection and
displays the assets that match both filters.
