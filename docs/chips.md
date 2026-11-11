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

## Tag input

`TagInput` pairs wrapping Chips with a draft editor. The host stores the
draft and the keyed tags, validates additions, and applies requests. Give
each tag a stable, unique key. Use `.removable(false)` for a fixed tag.

```rust
use dutchy_egui::{TagInput, TagInputRequest, TagItem};

fn show_tags(ui: &mut egui::Ui, draft: &mut String, tags: &mut Vec<(u64, String)>, next_key: &mut u64) {
    let items: Vec<_> = tags.iter()
        .map(|(key, label)| TagItem::new(*key, label.as_str()))
        .collect();
    let result = TagInput::new("project-tags", draft, &items)
        .accessible_name("Project tags")
        .show(ui);
    for request in result.requests {
        match request {
            TagInputRequest::Add(label) => {
                tags.push((*next_key, label));
                *next_key += 1;
                draft.clear();
            }
            TagInputRequest::Remove(key) => tags.retain(|(tag_key, _)| *tag_key != key),
            _ => {}
        }
    }
}
```

Enter and Add tag request one trimmed, nonblank label and focus the editor.
Backspace in an already empty draft requests removal of the last removable
tag. Tab visits each remove button before the draft, and Enter or Space on
a remove button requests removal and returns focus to the draft. Remove
buttons also support pointer and assistive activation.

The draft keeps its text until the host clears it. Showcase's **Tag input**
page accepts new project tags, keeps one tag fixed, and displays a validation
message when the host declines a duplicate.

## Multi-select

`MultiSelect` filters a catalog of `SelectOption` values using [fuzzy
search](fuzzy-search.md). The host stores the chosen values and `SearchState`.
The widget shows chosen values as removable Chips and requests changes by
value. Give options unique values and use `id_salt` to keep their identity
stable when labels change or repeat.

```rust
use dutchy_egui::{MultiSelect, SearchState, SelectOption};

fn show_media_types(ui: &mut egui::Ui, selected: &mut Vec<u8>, search: &mut SearchState) {
    let options = [SelectOption::new(1, "Photography"), SelectOption::new(2, "Prints")];
    let result = MultiSelect::new("media-types", selected, search, &options)
        .accessible_name("Media types")
        .show(ui);
    for request in result.requests {
        selected.retain(|value| *value != request.value);
        if request.selected {
            selected.push(request.value);
        }
    }
}
```

Click the query, type, or press Enter or an arrow key to open results. The
arrow keys move the active result and Enter requests a toggle. Toggling
preserves the query and keeps results open. Escape, Tab, or clicking outside closes
the popup. Tab reaches each Chip's remove button, which requests removal and
returns focus to the query. A selected disabled option stays visible as a fixed
Chip. Values absent from the catalog remain with the host.

Showcase's **Multi-select** page filters assets by any of the selected media
types and shows all assets when the selection is empty.
