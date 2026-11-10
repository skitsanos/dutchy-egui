# InlineEdit

`InlineEdit` replaces a name with a native single-line editor. The host owns
both the committed value and an `InlineEditState` for each stable row key.

```rust
use dutchy_egui::{egui, InlineEdit, InlineEditEvent, InlineEditState};

fn name(ui: &mut egui::Ui, value: &mut String, edit: &mut InlineEditState) {
    let result = InlineEdit::new("project-name", value, edit)
        .accessible_name("Project name")
        .char_limit(80)
        .show(ui);
    if let Some(InlineEditEvent::Commit(proposed)) = result.event {
        *value = proposed;
        // Persist the accepted name in the host.
    }
}
```

Click the label, or focus it and press Enter, Space or F2, to start. The current
name is selected for replacement. A matching double-click keeps the selection
without activating the newly shown actions. A held Space leaves the selection
intact until release or until focus leaves InlineEdit. Enter or the check action
submits the draft; Escape or the close action discards it. Focus then returns to
the label, unless another control took focus while the label was hidden or
disabled, for example while the host saves. Tab visits the editor, Save and
Cancel, then leaves the component. Focus loss keeps the draft; there is no
implicit save or cancellation. Modifier chords stay with the host, and
composition keys stay with native IME handling.

`InlineEditEvent::Commit` is a request, not a persistence result. Empty,
unchanged and whitespace strings are submitted verbatim. The host validates,
accepts or declines them. To retain a rejected proposal, call `edit.begin` with
that proposal and present feedback beside the control. The showcase rejects
blank names this way. `response.changed()` reports native draft edits; use the
event to decide whether to rename.

The draft survives label updates, row reordering, hiding and disabling. Keep
one state and a stable ID per record. While editing, an external label update
does not replace the draft; cancelling reveals the latest host value. Resolving
concurrent rename conflicts, asynchronous validation and persistence belong to
the host. Call `edit.cancel()` when a record is removed. Programmatic `begin`
and `cancel` do not emit user events.

Small, Medium and Large share control tokens; Small is the default. Width fills
available space unless `.width` bounds it. Narrow editors move their actions to
a second row. Long idle labels are elided; painting and pointer input are
clipped. Unbounded/invalid widths use a finite fallback. A Unicode scalar
`char_limit` affects typing and paste without truncating host-provided seeds.

`enabled(false)` disables all controls and retains the session. Disabled
parents are respected. `read_only(true)` prevents starting, changing and
submitting a name; an existing editor still supports selection/copy and Cancel.
Name each property with `accessible_name`, independent of the current value.
Idle controls expose a named Button whose value is the current name, or the
placeholder it shows when the name is empty; editors expose native text-input
semantics and named Save/Cancel buttons.

This scope does not include multi-line editing, implicit blur commits, a
validation service, rename conflict resolution or automatic List/TreeView row
integration. Compose the control into a host layout with enough row height for
its editing mode.
