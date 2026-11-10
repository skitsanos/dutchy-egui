# Richer menus and application command surfaces

The implementation extends Menu, Dropdown and Toolbar without a new
dependency. Flat command lists and their `.action` handlers remain supported.

```rust
use dutchy_egui::{MenuBar, MenuBarItem, MenuBarState, MenuEvent, MenuItem};

fn menus(ui: &mut egui::Ui, visibility: &mut MenuBarState, grid: &mut bool) {
    let formats = [MenuItem::new("png", "PNG image"), MenuItem::new("svg", "SVG vector")];
    let file = [MenuItem::new("export", "Export").submenu(&formats)];
    let view = [MenuItem::new("grid", "Show grid").checked(*grid)];
    let entries = [MenuBarItem::new("file", "File", &file), MenuBarItem::new("view", "View", &view)];
    let output = MenuBar::new("application", visibility, &entries)
        .accessible_name("Editor menus").show(ui);
    match output.event {
        Some(MenuEvent::Checked { value: "grid", checked }) => *grid = checked,
        Some(MenuEvent::Activated(format)) => { /* Host export command. */ }
        _ => {}
    }
}
```

## Public contract

- `MenuItem::checked(bool)` supplies the host's current checked value. A
  `MenuEvent` distinguishes command activation from a requested checked value;
  the existing `.action` result remains available for value-based handlers.
- `MenuItem::submenu(&items)` creates a non-executing branch with a stable key.
  Branches may nest; disabled ancestors disable their descendants. Empty
  branches are inert. Keys need only be unique within their sibling list.
- `MenuBarItem` names a borrowed menu; `MenuBar` binds a host `MenuBarState`
  controlling which top-level entry is open. This is an in-window application
  menu, not an operating-system menu-bar integration.
- `ToolbarItem::menu(&items)` opens the same command tree. Menu checked
  requests map to `ToolbarEvent::Toggled`, keeping one host event channel.
  Popup/focus paths are transient; application values and commands stay with
  the host. Shortcut labels do not register bindings.

## Interaction

Menus use Up/Down/Home/End, skip disabled entries, and activate with
pointer/Enter/Space. Right enters a submenu; Left or Escape returns one level.
Escape at the root dismisses to its trigger; on a closed MenuBar entry or
Toolbar menu button it releases focus like other egui widgets. Tab/Shift+Tab
exits the whole popup tree without trapping focus. Pointer hover supports
moving between a branch and its child; clicks outside all popup levels
preserve outside focus.
Tab direction comes from the individual key event's modifier snapshot, even
when Shift is released in the same native frame. Ctrl/Cmd/Alt+Tab chords remain
available to the host. Immediate Escape after entering a child returns one
level; a menu never recovers focus from an external editor.

The latest input method owns the active highlight: a moving pointer and
keyboard focus cannot light two sibling commands at once. While the pointer
owns it and rests outside the rows, the focused row keeps a muted fill so the
command Enter would activate stays visible. Open ancestors keep
a muted fill and accent arrow, with regular-weight labels so expansion does
not change wrapping or row height. Inline menus close descendants after a
leaf activation and return focus to the root branch.

The menu bar has one remembered Tab stop, horizontal navigation and Down/Up
opening at the first/last entry. When open, moving across top-level entries
switches menus. Toolbar retains its own one-stop navigation and opens menu
buttons with Down/Up, pointer/Enter/Space. Identity survives labels/reordering;
removing or disabling the focused command repairs focus.

Vertical toolbars keep Up/Down for tool navigation and open menus with Right.
Toolbar menu buttons with visible labels include a trailing chevron; icon-only
buttons retain their square target and popup metadata. Checked leaf choices
close the popup after returning a request, even when the host declines it.
Using `.event` and `.action` simultaneously would handle the same activation
twice; choose one channel. Branch values never execute, and checked values on
branches are ignored. Keep command trees acyclic and visible labels meaningful.

`MenuBarState::open(key)`, `close()` and `is_open(key)` control top-level
visibility. Missing, empty or disabled menus close safely. MenuBar and Toolbar
support `focus_when_empty(id)` for an explicit host destination after removal.
`Dropdown::show_for(ui, &response)` attaches to an already drawn native/Dutchy
button without allocating host layout space; the host supplies its enabled
state and accessible name. Context attachments also allocate no extra space.
Menu opening, switching and closing leave following controls in place. Context menus
continue to support secondary click and Shift+F10.

The [WAI menu/menubar](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/)
and [toolbar](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/) patterns inform
the navigation. This native egui implementation skips disabled entries rather
than making them arrow-focusable. Alt/F10 system-menu activation, mnemonic
letters, type-ahead, radio/mixed checks, arbitrary embedded widgets and
operating-system menu integration are out of scope. Shortcut hints remain
descriptive unless supplied through a host CommandRegistry; see
[shared commands](commands-shortcuts.md) for bindings and recursive
CommandPalette search with ancestor breadcrumbs. Branches never execute.

## Layout and showcase

Application menus share the menu bar's bottom border. Cascades share their
side border and align the first child row with the parent row when viewport
space permits. Short children use a measured width (minimum 200 points,
capped by the configured menu width and viewport) instead of copying a wide
root. Native popup placement can flip or shift the child at screen edges;
very narrow viewports may overlap the parent to keep commands reachable.
See the [menu interaction decisions](menu-interaction-decisions.md) for the
design rationale and a possible future drill-down presentation.

The Menus and Toolbars showcase pages both include Canal Studio: File/Edit/View
menus, a three-level export tree, connected grid/ruler settings and a toolbar
export button. “Apply view changes” demonstrates rejecting requests without
changing checked values. Export/copy choices report requests in the demo; they
do not write files or the system clipboard.
