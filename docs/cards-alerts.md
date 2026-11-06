# Cards and alerts

## Cards

Cards fill their available column and grow with content. Use the host's egui
layout/columns to choose width. Six `CardVariant` values mirror the web component:
Default, Accent, Bordered, Muted, Dark, Primary. `CardPadding` supplies None,
Small (24), Medium (32, default), and Large (40) logical-point insets. Accent
cards reserve a four-point left border; bordered cards use a two-point border.

```rust
use dutchy_egui::{Card, CardVariant};

fn project(ui: &mut egui::Ui) {
    Card::new().variant(CardVariant::Bordered).show(ui, |ui| {
        Card::header(ui, |ui| { Card::title(ui, "Project details"); });
        Card::description(ui, "Related information belongs together.");
        Card::footer(ui, |ui| { ui.label("Updated today"); });
    });
}
```

`show` returns `InnerResponse<R>` and creates a non-focusable container. Compose
normal controls, images supplied by the host, or arbitrary content in its UI.
The helpers are optional: title supplies an uppercase display heading with
heading level 3 metadata; description supplies relaxed small text; header/footer
add the documented spacing and footer divider.

`show_interactive(ui, id, accessible_name, contents)` adds a native button action
to the whole surface. Use a stable ID unique within the parent namespace and
inspect `result.response.clicked()`. Enter/Space activate it. Display text does
not intercept pointer clicks; explicit child controls retain their own actions.
Keep interactive-card content simple, especially when targeting screen readers.
`enabled(false)` disables the card and descendants and preserves disabled parents.

Hover uses the muted surface and normal foreground, without animation. Focus
has the shared sharp outline. Appearance changes never invoke application work.

Primary/inverted surfaces set a scoped foreground. Dutchy `Text` Foreground and
Muted colors and native egui text inherit it. Explicit semantic/custom Text
colors remain explicit. A nested card restores its own surface color. Manually
created `LayoutJob`s resolve the Theme passed to `layout_job`, so their caller
must supply the desired surface color. Buttons retain their own variant colors.
Dark means an inverted foreground surface, so it becomes light in dark mode.

## Alerts and banners

`Alert::new(stable_id, message)` defaults to Information. `AlertVariant` adds
Success, Warning, and Error. A custom title replaces the visible severity title;
the accessible name retains severity. Symbols also distinguish intent.

```rust
use dutchy_egui::{Alert, AlertVariant, Button};

fn connection(ui: &mut egui::Ui, visible: &mut bool, retries: &mut u32) {
    let restore = ui.add(Button::new("Show connection alert"));
    if restore.clicked() { *visible = true; }
    if *visible {
        let result = Alert::new("connection", "Reconnect before continuing.")
            .variant(AlertVariant::Error)
            .title("Connection interrupted")
            .action("Retry")
            .dismissible(true)
            .return_focus_to(restore.id)
            .show(ui);
        if result.action_clicked { *retries += 1; }
        if result.dismissed { *visible = false; }
    }
}
```

The result separates container `response`, `dismissed`, and `action_clicked`.
Message clicks do not activate either control. No timers, network operations,
removal, or application state are hidden inside the widget. The host decides
whether to remove the alert and what its action does. Changing/adding an alert
does not steal focus. If supplied, the dismissal focus target must remain present
and focusable; without a target, the host is responsible for focus after removal.

Information/Success expose polite status announcements; Warning/Error expose
assertive alerts. `announce(false)` presents a group with live announcements off
for static/history content. Dismiss controls have specific accessible names and
native keyboard activation. Disabled alerts and disabled parents cannot activate
their controls.

Standard alerts use an opaque 10% accent tint blended over the current theme
background, a four-point left border, and 16-point padding. Information follows
the active brand accent. Success/Warning/Error reuse the shared semantic palette,
including its existing warning token. This resolves conflicting hardcoded colors
in the web implementation in favor of theme tokens.

`banner(true)` uses a solid intent fill with on-fill text. Actions use a secondary
button on solid banners and an outline button on tinted alerts. Actions sit below
the message, allowing wrapping in narrow columns. Banners fill their parent UI;
the application decides whether that means a card, page, or window width. They
are inline notifications, with no blocking overlay behavior.

Source-palette contrast limitations remain in [DS-003](issues/DS-003.md), including
white text on some brand/status fills. Native testing currently covers macOS
accessibility and interaction, not a running screen reader. See the
[verification record](verification-surfaces.md).
