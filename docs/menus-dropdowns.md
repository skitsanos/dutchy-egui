# Menus, dropdowns, and command search

The web reference is `docs/01-design-system/components/menu.md` and
`dropdown.md`, plus `src/components/Menu`, `MenuItem`, and `Dropdown` at
`e91ca1d506e1c7a11b58a5d6857d82208229812a`. The implementation uses native egui
responses, Popup, ScrollArea, TextEdit, and Modal. No helper crates or copied
third-party widget code were added.

## Actions and anchored menus

```rust
use dutchy_egui::{Button, Dropdown, MenuItem, MenuItemVariant};

fn project_actions(ui: &mut egui::Ui, open: &mut bool) -> Option<u8> {
    let items = [
        MenuItem::new(1, "Duplicate").group("Project").shortcut("Ctrl+D"),
        MenuItem::new(2, "Archive").group("Project").enabled(false),
        MenuItem::new(3, "Delete").variant(MenuItemVariant::Destructive),
    ];
    Dropdown::new("project-actions", open, &items)
        .accessible_name("Project actions")
        .show(ui, |ui| ui.add(Button::new("Actions")))
        .action
}
```

`Menu::show` renders the same list inline. `MenuItem` values supply stable keys:
use unique, immutable action identifiers implementing Hash and Debug. Labels may
change or items may reorder without changing their identity. Consecutive equal
group names form named groups. Shortcut strings are descriptive hints; register
actual key bindings in the host. An optional `MenuIconPainter` function draws a
decorative icon using the supplied token-sized rectangle and foreground color.

`MenuWidth` supplies 200/320/384/448-point preferences, constrained by the parent
or viewport. Rows wrap labels and reserve space for hints/icons. All menus use
flat fills and a two-point outer border. Disabled items cannot activate.

- Click, Enter, or Space toggles a dropdown. Up/Down opens at its last/first
  enabled item respectively. Within a menu, Up/Down/Home/End navigate enabled
  actions; Enter/Space or pointer/assistive click returns an action.
- Escape closes and restores trigger focus. For closure-built dropdown triggers,
  Tab/Shift+Tab closes and moves to the neighboring host control. A context menu
  attached to an already-drawn response dismisses to that target on Tab; the next
  Tab follows host order. Outside clicks keep their normal focus target.
- `show_context(ui, &response)` opens at a fixed pointer position on secondary
  click, or at the trigger edge with Shift+F10. The target needs native focus
  support for keyboard activation. Host-provided triggers need meaningful names.
- `DropdownAlign::{Start,End}` sets preferred alignment; native placement may
  flip to keep the popup visible. Long popup contents scroll vertically.

The host owns open flags and action execution. The library returns typed actions;
it never deletes data or registers the shortcut hints.

Checked actions, nested submenus, application menu bars and Toolbar menu buttons
are covered by the [richer-menu contract](richer-menus.md). `MenuResponse.event`
adds typed checked-state requests; `.action` remains a leaf-value result.

## Editable selection

`SearchableSelect::new(id, &mut Option<T>, &mut SearchState, &options)` uses the
existing `SelectOption` type. Give options stable `id_salt` values when labels can
change. The host owns the query, open flag, and committed selection separately.
Supply the initial query label explicitly if the host wants it displayed.

Typing filters and ranks options with [fuzzy matching](fuzzy-search.md) and
marks matched glyphs in bold. Up/Down opens and moves the active
result; Home/End navigate open results. Enter or a pointer click commits, copies
the label to the query, and closes. Escape/outside click dismisses without
committing. Tab exits normally. Disabled results remain visible but are skipped.
An unknown/missing selected value is retained; clearing the query does not clear
selection. Responses distinguish `committed` from `selection_changed` and native
query `response.changed()`.

The editor retains actual keyboard focus. AccessKit exposes an editable combobox,
controlled listbox, selected options, and its active descendant. Accessibility
consumers may report the active option as focused while typing still goes to the
editor. No matches provides a polite status message.

## Command palette

Call `CommandPalette::new(global_id, &mut SearchState, &items).show(ctx)` once per
frame, including while closed. The default logical Command+K shortcut is Cmd+K
on macOS and Ctrl+K elsewhere. `.shortcut(None)` leaves opening entirely to the
host; `.shortcut(Some(...))` changes the binding. Opening is suppressed while a
different native modal is on top.

The palette header uses the shared bordered 48-point `Input` rather than the
web's borderless 14-point field with a leading search glyph; menu icon slots are
20 points (web 16); shortcut hints on a highlighted row switch to the on-primary
color for contrast where the live row keeps muted-foreground. Dropdown lists are
capped at 320 points (`DROPDOWN_MAX_HEIGHT`); the web dropdown has no cap.

The palette uses a centered native modal, four-point border, token backdrop,
512-point maximum width, and scrollable grouped results. Query and active-result
behavior match searchable selection. Enter returns an action. Escape, backdrop,
or the shortcut closes. Tab/Shift+Tab stays inside the modal. Closing restores
preceding host focus after the native modal layer clears; keep calling `show`
while closed to complete this. If another modal takes over first, the palette
abandons the restoration and requests no further frames. If the host removes that target, it must choose a
new focus destination. Host-set queries are retained between openings.

## Boundaries

Command search uses flat leaf lists; [fuzzy search](fuzzy-search.md) lists
the matching limits. Radio/mixed menu checks, rich arbitrary result bodies, asynchronous
search and virtualization are outside the supported scope.
Very long shortcut hints are clipped; avoid using hints for essential guidance.
Native popup placement may settle after a sizing pass. Display scales use logical
points, and source palette contrast limitations remain in DS-003.
