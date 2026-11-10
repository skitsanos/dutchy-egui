# List

`List` renders a current view of keyed `ListItem`s. The application owns the
rows and `ListState`: `selected` is a set of keys and `focused` is the roving
key. Order, filtering, data fetching and actions stay with the host. No helper
crate is required; there is no separate VirtualList component.

```rust
use dutchy_egui::{List, ListItem, ListSelectionMode, ListState};

fn folders(ui: &mut egui::Ui, state: &mut ListState<u64>) {
    let items = [ListItem::new(10, "Inbox"), ListItem::new(20, "Archive")];
    let result = List::new("folders", state, &items)
        .accessible_name("Folders")
        .selection_mode(ListSelectionMode::Multiple)
        .empty_text("No folders")
        .max_height(288.0)
        .show(ui);
    if let Some(key) = result.activated {
        // The host opens the item identified by key.
    }
}
```

## Selection and keyboard

Single selection is the default and can be empty. Multiple selection uses the
desktop modifier model from the [W3C listbox pattern](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/).
Focus and selection are distinct. Command means ⌘ on macOS and Control on
Windows and Linux, as egui reports it; a bare macOS Control-click is the
secondary click and does not toggle.

| Input | Result |
| --- | --- |
| Click | Focus and replace selection |
| Command + click | Toggle one key |
| Shift + click / navigation | Replace selection with the enabled anchor-to-target range |
| Command + Shift + click / navigation | Add that range to existing selection |
| Up/Down, Home/End, Page Up/Down | Move without wrapping; skip disabled rows; selection follows |
| Command + navigation | Move focus without changing selection |
| Space | Select the single row, or toggle it in multiple mode |
| Command + A | Select all enabled rows in multiple mode |
| Enter / double-click | Return `activated`; the host executes it |
| Tab / Shift+Tab | Enter or leave through one roving row |

Type-ahead compares Unicode lowercase label prefixes, searches the whole view,
and reveals a match. Spaces continue a pending prefix, so typing "new y"
reaches "New York"; Space toggles selection only when no prefix is pending.
Repeating a single character cycles through matches; a one-second idle period
resets the prefix. It is bounded to 64 characters and
is not fuzzy, accent-insensitive or locale-aware search. Clipboard and IME
preedit events are not interpreted as commands.

Setting `focused` from the host reveals that key without stealing focus from
another control. Reordering keeps keyed identity. Removing or disabling keys
prunes selection and repairs focus to the first selected/enabled row. Switching
to Single keeps the first selected key in current-view order; None clears
selection. Missing/filtered keys are pruned, so retain cross-filter selections
separately if the application needs them. Only the first duplicate key is
interactive; unique keys remain the host contract.

## Virtualization and composition

All lists use egui's `ScrollArea::show_rows`. Height is fixed per row (default
48, configurable 32–256 points); viewport caps range from 96–4096 points.
Only the visible range and egui overscan receive row responses. Row content is
rendered only where visible. An offscreen focus proxy preserves the active
row's identity without painting it or snapping manual scrolling back.
Keyboard navigation reveals its destination immediately. Metadata validation
scans the current view; content layout and painting scale with the viewport.

`show` paints each label on one line and ends a label that does not fit in
"…". An empty view shows muted `empty_text` (default "No items"); keep it to a
short line. `show_with` composes passive text, icons and badges inside the
fixed clipped row. It does not support embedded buttons, checkboxes or editors: those need
a different composite such as DataTable. Use a unique list ID per surface and
keep callback side effects safe for egui's sizing/discard passes. Variable
height, drag reorder, trees and asynchronous data sources are outside this
component's scope.

Selectable modes expose ListBox/ListBoxOption roles, selection, total size and
zero-based AccessKit positions. None uses List/ListItem roles. Virtualized
accessibility exposes visible rows and the focused proxy; keyboard navigation
reaches every enabled row. Disabled rows cannot be selected, focused or
activated. Read-only keeps navigation and scrolling but suppresses selection
edits and activation; stale keys are still normalized.

The showcase's **Lists** page demonstrates 10,000 generated projects, mode
switching, passive badges, host removal/reorder and activation requests.
