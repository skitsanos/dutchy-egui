# Icon actions, toggle buttons and toolbars

The implementation adds repository-owned icons, IconButton, ToggleButton and
Toolbar without new dependencies. They reuse button variants, control sizes,
focus, tooltips and square loading paint. The README remains a short entry point;
this guide owns the API details and supported scope.

## Standalone actions

```rust
use dutchy_egui::{Icon, IconButton, ToggleButton, egui};

fn actions(ui: &mut egui::Ui, bold: &mut bool) {
    if ui.add(IconButton::new("save", "Save document", Icon::Save)).clicked() {
        // Execute the host's save command.
    }
    if ui.add(ToggleButton::new("bold", bold, "Bold").icon(Icon::Bold)).changed() {
        // Persist/apply the new host value.
    }
}
```

IconButton requires a stable ID, descriptive action name and decorative icon.
Its target is square: 32, 48 or 56 points via ControlSize. Ghost is the default;
all five ButtonVariant treatments are available. Pointer, Enter and Space
produce the usual egui Response. Names keep their original casing.

ToggleButton binds a host bool. It defaults to Outline, optionally adds an icon,
and supports `.icon_only(true)`. Keep the name stable between states: use
“Bold”, not “Turn bold on/off”. Pressed metadata and `changed()` track user edits;
external host changes are rendered without generating an edit. Toggles are
two-state; exclusive choices can use Segmented or host-controlled toolbar items.

Both builders support `.enabled`, `.loading`, `.size`, `.variant` and `.tooltip`.
Busy controls expose busy/disabled metadata, show a square spinner and cannot
activate. Disabled toggles retain their pressed surface. Icon-only controls
show their name on hover/focus; override with `.tooltip("Help")` or opt out with
`.tooltip("")`. Visible-label toggles show help only when requested. Transparent
resting actions inherit inverse card ink. Keyboard focus uses the shared cue;
pointer focus does not add a separate outline.

A toggle's two surfaces stay distinguishable while hovered (DS-064): an unpressed
Outline toggle hovers with the muted fill and keeps its stroke, an unpressed Ghost
toggle with the muted fill alone, and a pressed toggle keeps the inverse Secondary
fill, tinted, instead of switching to the accent. Held pointers darken the current
fill. Plain IconButtons keep the ordinary button hover, including the inverted
Outline hover. Secondary toggles add a token-sized accent edge while pressed
(DS-068), so their on/off states remain visible even though both use the inverse
surface. This applies to standalone and toolbar toggles.

## Typed toolbars

```rust
use dutchy_egui::{Icon, Toolbar, ToolbarEvent, ToolbarItem, egui};

fn tools(ui: &mut egui::Ui, bold: &mut bool) {
    let items = [
        ToolbarItem::action("save", "Save document").icon(Icon::Save),
        ToolbarItem::toggle("bold", "Bold", *bold)
            .icon(Icon::Bold).separator_before(true),
    ];
    let output = Toolbar::new("document-tools", &items)
        .accessible_name("Document tools")
        .show(ui);
    match output.event {
        Some(ToolbarEvent::Activated("save")) => { /* Host save command. */ }
        Some(ToolbarEvent::Toggled { value: "bold", pressed }) => *bold = pressed,
        _ => {}
    }
}
```

Toolbar returns a command or desired toggle value; it never applies durable
state. The host may accept or reject a request. Values seed stable IDs, so use
unique values or `.id_salt(...)` for repeated commands. Labels/reordering do not
change identity. Item builders support icons, visible labels, variants, help,
separators, disabled and busy states. ToolbarResponse includes the group
response, visible viewport, keyed item responses and at most one event.

Tab enters through one remembered enabled item and exits normally. Horizontal
Left/Right or vertical Up/Down, plus Home/End, move focus without activating.
Arrows wrap and skip disabled/busy items. If the focused item disappears, the
bar repairs focus to a surviving item. `.focus_when_empty(id)` supplies an
explicit host target when none remain; keep calling `.show` while empty to
apply this policy. Moving focus reveals overflow once; retained focus leaves
manual scrolling alone.

Horizontal scrolling and Small controls are the defaults. Use `.wrap(true)`
for wrapping with linear arrow order, `.orientation(ToolbarOrientation::Vertical)`
for Up/Down navigation, `.width(...)` for a preferred bounded width, or
`.bordered(false)` for a plain group. Vertical bars fit an icon target by default;
set a wider width for text items. Vertical bars reserve their natural height;
`.max_height(...)` bounds the content viewport and enables vertical overflow.
Toolbar owns its background and token padding.
Standalone icon/toggle buttons each remain ordinary Tab stops outside a toolbar.

These conventions follow the [WAI toolbar pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/)
and [button pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/). The native
implementation is verified independently; web guidance is not a native parity claim.

## Menu buttons

`ToolbarItem::action(key, name).menu(&commands)` opens a shared MenuItem tree.
The trigger key never executes; leaf commands return `ToolbarEvent::Activated`
and checked leaves return `ToolbarEvent::Toggled { value, pressed }`.
Horizontal menu buttons open with Up/Down; vertical ones use Right and retain
Up/Down navigation. Escape returns to the trigger and Tab exits the whole
toolbar. Stable keys, disabled/busy inheritance and focus repair still apply.
See the [menu guide](richer-menus.md) for a connected showcase and full limits.

## Icons and scope

`Icon::ALL` contains 22 original geometric glyphs. `Icon::Custom(fn)` accepts a
host painter receiving `&Painter`, `Rect` and semantic `Color32`. Painting is
clipped to finite positive bounds. Icons create no interaction/accessibility
nodes themselves; name the containing control for its action. No icon font,
helper crate or borrowed source is bundled.

The three showcase pages include an icon gallery/custom diamond, variants/sizes,
pressed/disabled/busy toggles and a host-owned formatting preview with overflow,
wrapping and vertical toolbars. The connected command-surface section also
demonstrates menu buttons. Commands, persistence, shortcuts, arbitrary
child inputs and a rich-text editor remain application concerns. No mixed-state
toggle, drag reorder or customizable overflow menu is included.

See [verification evidence](verification-actions-toolbars.md) and
[DS-062](issues/DS-062.md). This work shipped in release 1.0.24.
