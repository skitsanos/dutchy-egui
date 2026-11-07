# Accordions and tabs

Version 1.0.6 begins the interaction-components milestone. References:
`docs/01-design-system/components/{accordion,tabs}.md` and
`src/components/{Accordion,Tabs}/index.tsx` in the web revision recorded in
[plan](plan.md), rechecked 2026-11-06. Geometry follows the component source
where older prose examples differ.

## Controlled accordions

```rust
use dutchy_egui::{Accordion, AccordionItem, AccordionState, Text, TextSize};

fn details(ui: &mut egui::Ui, expansion: &mut AccordionState<&'static str>) {
    let items = [
        AccordionItem::new("overview", "Overview"),
        AccordionItem::new("settings", "Settings"),
    ];
    Accordion::new("workspace-details", expansion, &items)
        .accessible_name("Workspace details")
        .show(ui, |ui, key| {
            ui.add(Text::new(format!("Content for {key}")).size(TextSize::Sm));
        });
}

let single = AccordionState::Single(Some("overview"));
let multiple = AccordionState::Multiple(vec!["overview", "settings"]);
```

The host owns expansion; `Single(None)` closes everything. `Multiple` permits
independent open items and retains temporarily absent keys. `toggle` is also
available for host actions. Keys must be unique. Item constructors derive widget
identities from values (`Hash + Debug`), independently of labels and order.

The default surface has separate background-filled items, eight-point gaps,
four-point left borders, primary hover edges, display headers, and 24-point
padding. `AccordionVariant::Bordered` uses one two-point outer border and shared
dividers, body headers, 24/16-point header padding, muted hover fills, and thin
panel dividers. Header text is 14-point bold uppercase with wider (0.05 em)
tracking on the default variant and wide (0.025 em) on the bordered one, as
live. The default accent edge lights while the header or the open body is under
the pointer. Long
headers wrap beside a reserved chevron; body width and typography remain under
the callback's control. Host spacing is preserved inside panels.

Every enabled header is a Tab stop. Enter/Space toggles; Up/Down wrap among
enabled headers, Home/End choose the first/last. Arrow navigation does not expand
items and does not intercept keys in nested controls. Clicking a header focuses
it. Disabled items/parents prevent both header and body interaction without
changing expansion. Visible bodies have Region roles, names, and labelled-by
relationships to their expanded buttons; closed regions are absent.

Only open body callbacks run. `AccordionResponse` contains the group response,
headers, and optional body results in source order. User toggles mark the response
changed. A single-open toggle of a later item may settle the earlier item's
layout on the requested next repaint, as expected in an immediate-mode layout.
Draw callbacks should not execute business actions merely because they render.

Body visibility changes immediately, matching native HTML disclosure. Only the
chevron rotates, using the shared 200ms motion token; reduced motion disables it.
Clipped headers and settled transitions do not schedule animation frames. The
host should choose a focus destination if it externally removes/collapses a
panel while a child owns focus. Normal header toggles already focus that header.

## Controlled tabs

```rust
use dutchy_egui::{Tab, Tabs, Text};

fn views(ui: &mut egui::Ui, selected: &mut Option<&'static str>) {
    let items = [
        Tab::new("overview", "Overview"),
        Tab::new("settings", "Settings"),
        Tab::new("history", "History").enabled(false),
    ];
    Tabs::new("workspace-views", selected, &items)
        .accessible_name("Workspace views")
        .show(ui, |ui, key| { ui.add(Text::new(format!("View: {key}"))); });
}
```

The host owns `Option<T>`. A missing/disabled selection normalizes to the first
enabled tab, or `None` if none remain; normalization marks the group response
changed. Values and widget keys must be unique and durable. Only the active
panel callback runs; hidden values must live in host state. Keyed panel IDs
preserve editor focus across reordering, label changes, and resizing.

`TabsVariant::Underline` uses a four-point active accent edge over a two-point
baseline and 56-point tabs. `Segmented` uses a muted strip, four-point inset,
background active fill, and 44-point segments. Labels are single-line, 14-point
bold uppercase. Newlines display as spaces. Panel padding is 24 points.
`full_width(true)` requests equal widths; segmented controls default to this,
underline tabs default to natural widths. Long labels still scroll rather than
being clipped into unreadable equal-width cells. Tabs do not wrap.

The tab list is named and horizontal. Tabs expose selected/disabled state. Only
the active panel exists in the accessibility tree; its labelled-by/controls
relationships link to the selected tab. The panel is a focusable TabPanel so
static content also has a keyboard destination. Its children retain native roles
and normal keyboard behavior; panel focus has a visible outline.

This is **manual activation**: Left/Right wrap across enabled tabs without
selecting; Home/End move to the first/last. Enter/Space activates. One tab is in
the Tab order: the selected one on entry, or the current roving tab while the
strip owns focus. A closable roving tab also exposes its close button as the next
Tab stop, then the active panel follows. Disabled tabs are skipped. Left/Right
are locked on a focused tab and on its close button; Up/Down keep egui's native
spatial navigation. Focused tabs scroll into view, with shared timing or
immediate reduced-motion scrolling. The close glyph is an 8-point, 2-point
stroke and underline labels center in the 56-point strip (about 2 points lower
than the web); the overflow strip keeps egui's floating scrollbar.
Native horizontal wheel/trackpad/scrollbar behavior remains available.

Selected tabs use their active underline or segmented surface without an extra
outline. Keyboard navigation to an unselected tab still paints the focus outline;
close controls and the panel keep their own focus indicators.
Shared focus-visible behavior hides extra rings after pointer presses without
removing native focus. Keyboard and assistive actions restore cues; hover does
not change the mode. Accordion rings sit inside headers so adjacent fills cannot
cover their edges.

## Closing and focus

Mark individual items with `.closable(true)`. Clicking the named close button,
pressing Enter/Space on it, or Delete on a focused closable tab/close button
returns `TabsResponse::close_requested`. Closing does not select the tab or
remove data. The host can reject the request, prompt for unsaved changes, remove
the item, and choose its next selection. The showcase retains notes and chooses
an adjacent surviving document.

If the host leaves a removed selection unchanged, the next render normalizes it
to the first enabled tab. If a focused tab/close button disappears or becomes
disabled, focus moves to the valid selection. `.focus_when_empty(add_button.id)`
supplies an explicit destination when none remain. This only reacts to removal
of a previously focused strip control; appearing or empty tabs do not steal
unrelated focus. The host must move focus itself when externally hiding a focused
panel child. Disabled parents prevent tab selection, closing, and panel editing.

`TabsResponse` exposes the group and tab responses, visible strip bounds, active
panel response/result, and close request. The group is changed for selection,
not for a close request that leaves host state unchanged. Display-scale rounding
can leave up to a physical pixel at a scroll boundary.

## Native scope

These are original, repository-maintained implementations using egui interaction,
focus memory, accessible metadata, and ScrollArea. No dependencies or copied
third-party widget code were added. Durable content/selection/expansion and save/
close decisions remain with the host; egui memory holds transient focus,
scroll and animation information plus two small flags: the shared input-mode
flag behind the focus-visible policy and each strip's previous-focus record.

The web overflow dropdown/arrows are represented by native horizontal scrolling
and keyboard reveal. An overflow menu, icon slots, docking, reorder dragging,
vertical tabs, and panel-height animation are outside
this step. This does not complete the entire interaction-components milestone.
See [coverage](coverage.md) and [verification](verification-disclosures.md).

Tab close controls use the shared flat hover/pressed surface and pointing cursor,
with their own keyboard focus cue. Hovering close does not select its tab;
activation still returns a host-owned close request. See [DS-013](issues/DS-013.md).
