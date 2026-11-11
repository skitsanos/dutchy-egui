# Shared commands and shortcuts

CommandRegistry keeps application metadata together; it never executes an
action or owns application records. The existing Menu, MenuBar, Toolbar and
CommandPalette APIs remain available.

```rust
use dutchy_egui::{Command, CommandRegistry};
use egui::{Key, KeyboardShortcut, Modifiers};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Action { Save, Grid }
let commands = CommandRegistry::new("editor-commands", [
    Command::new(Action::Save, "Save document")
        .shortcut(KeyboardShortcut::new(Modifiers::COMMAND, Key::S))
        .allow_in_text(true),
    Command::new(Action::Grid, "Show grid").checked(true),
]);
// During the UI frame, before controls:
// for request in commands.dispatch(ctx) { /* host handles request */ }
// let save = commands.menu_item(ctx, &Action::Save);
// let tool = commands.toolbar_item(ctx, &Action::Save);
```

## Registry contract

- Keys identify actions independently of labels and order. Inserting the same
  key updates its definition. The host updates enabled/checked state and can
  remove or rebind actions. Registry metadata is session-local; persistence is
  a host concern.
- `menu_item`, `menu_items`, `toolbar_item` and `shortcut_label` share the same
  labels, availability, checks and native platform formatting. Call UI adapters
  during a frame, after fonts have been installed. Toolbar shortcuts appear in
  tooltips; Menu and CommandPalette show them as trailing hints.
- `request(&key)` validates current availability and returns a CommandRequest.
  Its `checked` field proposes the inverse host value; rejecting a request does
  not change that value. Route one surface result through this method, rather
  than handling both its compatibility action and typed event channels.
- `dispatch(ctx)` consumes non-repeated key presses before drawing controls.
  It uses each event's modifier snapshot and requires exact Command, Control,
  Alt and Shift state; on macOS Command and Control are separate keys.
  COMMAND maps to Command on macOS and Control elsewhere. Each action returns
  at most once per frame; a repeated dispatch or discarded layout pass does
  not execute it again or return its chord to another control.
- Application bindings yield to text editors, IME composition, modal overlays,
  Dutchy popups and egui memory-managed popups. For a custom host-managed
  overlay outside these APIs, skip dispatch while it owns input. A host can
  explicitly opt a command into editing with
  `allow_in_text(true)` or overlays with `allow_in_overlays(true)`. Keep ordinary
  edit commands with the focused editor. Focused(Id) scope applies only to the
  named widget and takes precedence over application scope. Unmodified Enter,
  Space, Tab, Escape, arrow, Home, End and page keys stay with a focused
  control: an application binding on them applies only while nothing has focus.
- `conflicts(ctx)` reports equally scoped enabled bindings, including physical
  platform aliases. Ambiguous active bindings do not execute. Different focus
  targets can reuse a chord; a focused binding intentionally shadows a global
  one. Separate registries are separate command domains: callers must avoid
  competing ownership of the same input.

No multi-chord sequences, system-wide hotkeys, physical scan-code bindings or
operating-system menu integration are supplied.

## SplitButton

`SplitButton::new(id, label, &mut open, &alternatives).show(ui)` composes an
ordinary primary button and an adjacent, separately named arrow. The host reads
`primary_clicked` or `action`; opening alternatives never invokes the primary.
Five variants, three control sizes, disabled and busy states use shared tokens.
The arrow has its own Tab stop; Down or Alt+Down from the primary opens the
menu. Dropdown supplies arrows, Enter/Space, Escape, Tab exit and focus return.
Empty alternatives disable the arrow, leaving the primary usable. Disabled and
busy states fade the joined control once, divider included. Use
`alternatives_name` for a clear name such as "Export alternatives".

## KeyRecorder

`KeyRecorder::new(id, &mut shortcut).show(ui)` edits an optional
KeyboardShortcut. Clicking or activating the named button starts recording;
the next non-repeated key chord commits. Modifier-only input does not commit.
Escape cancels; Tab cancels and follows native navigation; focus loss,
disabling and IME end capture without changing the value. The explicit Clear
button removes a binding. Read-only controls remain focusable.

Supply `reserved(&other_bindings)` to reject conflicts with an accessible
inline alert. The uncommitted chord never replaces the host value. Portable
Command/Control formatting is retained when recording. While capture is active,
CommandRegistry yields input. Clear/cancel policies reserve Tab and Escape for
navigation; those keys cannot be assigned by this control. Enter, Space, arrow,
Home, End and page keys need Command, Control or Alt, with an explanatory
alert, unless the host calls `allow_navigation_keys(true)`. On macOS, Control
stays part of a Command chord. OS-reserved chords
may never reach the application. When wrapped in Field, its visible label is
also the recorder's accessible name.

## Nested discovery

CommandPalette recursively searches submenu leaves. Breadcrumb group
headings retain ancestor context, and every query word must match the label
or the path. Disabled ancestors disable descendants; empty branches
produce no commands and branch identity values never execute. Full keyed
paths distinguish equal leaf keys in separate branches. Renaming a branch
changes its display path without changing leaf identity. Flat items retain
existing identity. Matching is [fuzzy](fuzzy-search.md): results are ranked
best first and matched glyphs are bold.

The palette's default opening shortcut remains available. When a registry owns
opening/closing, use `.shortcut(None)` to give it sole ownership. MenuItem
shortcut strings remain display-only unless a host routes the matching binding.

## Examples

DutchyTodo shares New task, Search tasks, Import/Export and palette bindings;
its Shortcuts drawer edits the live catalog. New task directs an empty
application to project creation. The split action offers file operations.
Canal Studio shares Save, export formats, Grid/Rulers and palette commands across
its menu bar, toolbar, split action and nested palette. Rebinding Save updates
all hints. The "Apply view changes" switch demonstrates declining requests.
DutchyMail shares command names and shortcuts across its menu bar, message
context menus, toolbar and status hints. The host validates availability
against the selection or the context menu's captured message targets.
These examples keep command execution and Undo/import state with the host.
