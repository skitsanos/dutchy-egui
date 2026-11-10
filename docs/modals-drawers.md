# Modals and drawers

`Modal` and `Drawer` bind an application-owned `bool`. They render only while
open and never execute business actions or discard content values. Call each
component **every frame, including while closed**, using a globally unique,
stable ID. Transient focus/entrance memory lives in egui; the host owns visibility,
forms, validation, work, and any persistence or draft-reset policy.

```rust
use dutchy_egui::{Button, Modal, OverlayOptions, Text};

fn editor(ctx: &egui::Context, open: &mut bool, saves: &mut usize) {
    let result = Modal::new("sample-editor", open, "Edit record")
        .options(OverlayOptions::default())
        .show_with_footer(
            ctx,
            |ui| { ui.add(Text::new("Review before saving.")); },
            |ui| ui.add(Button::new("Save record")).clicked(),
        );
    if result.is_some_and(|response| response.inner.1) {
        *saves += 1; // The host performs the action.
        *open = false;
    }
}
```

`show(ctx, body)` returns `Option<OverlayResponse<R>>` for the body result.
`show_with_footer(ctx, body, footer)` returns the body/footer results as a tuple.
Closed overlays return `None` and do not invoke either content closure. Like
other egui UIs, closures can run in sizing/layout passes: keep business effects
behind actual activation results, rather than executing effects while drawing.

## Presentation

| Component | Width tokens | Surface |
| --- | --- | --- |
| `Modal` | `ModalWidth::{Small, Medium, Large}` = 448/512/672 points | Centered, 4-point border, 16-point viewport clearance. Default Small. |
| `Drawer` | `DrawerWidth::{Small, Medium, Large}` = 320/384/448 points | Full-height, 4-point inward-facing edge, left/right attachment. Default Small and Right. |

The archive confirmation showcase uses Medium (512 points) so its two standard
buttons share a row at desktop widths and wrap on smaller viewports.

Widths clamp to the viewport. `Drawer::side(DrawerSide::Left)` changes attachment;
`width(...)` chooses a token. Both use a flat themed fill, bold uppercase display
title, 32-point close target, 24-point horizontal/16-point vertical header/footer
padding, 24-point body padding, and 2-point dividers. Header padding follows the
web source; it intentionally differs from the alert's equal 16-point insets.

The header stays outside the scroll area. Body and footer share a constrained
vertical scroll area in native reading/Tab order; a footer can scroll below the
viewport and is not sticky. Keyboard focus reveals off-screen content controls
without animation. Use wrapping action rows or full-width buttons for
narrow surfaces. User content must respect `ui.available_width()`; custom painting
or unbounded host widgets can still overflow their allocation.

Drawers slide in over the shared 200ms motion token. Reduced motion places them
at their final edge immediately. Centered dialogs open without the web scale/fade
animation. All overlays disappear immediately on closing; no exit animation delays
input release or focus restoration. They schedule no entrance frames once settled.
Entry focus is applied in the first pass that reports the surface as the
topmost modal; an overlay opened beneath another modal waits without
repainting until that modal closes.
A drawer is a blocking overlay; use native egui panels for docked layouts.

## Dismissal and focus

`OverlayOptions` exposes independent `dismiss_on_escape`, `dismiss_on_backdrop`,
and `show_close_button` flags, all true by default. Each field has a chainable
setter of the same name, such as `OverlayOptions::default().dismiss_on_backdrop(false)`;
the struct is non-exhaustive, so it can't be built as a literal. A content closure can call
`ui.close()` regardless of these flags. Close/Escape/backdrop/content dismissal
sets the bound open flag false and returns the corresponding
`OverlayDismissal` in `response.dismissed`. A host closing its own flag after an
action is separate from these dismissal reasons. Neither path executes an action.

Native egui modal layers block the underlying UI and confine keyboard focus.
The close control is named “Close {title}”; the title is an accessible level-two
heading. A dialog is named and marked modal. Enter/Space activate native buttons.
Escape dismisses an open child popup before dismissing the top overlay. Prefer
one overlay at a time, as the web guidance recommends. Native stack behavior is
covered by a topmost-dismissal/focus-return regression; this is not a wizard or
a nested-dialog orchestration API.

Entry defaults to the close button, or the heading when close is hidden. The
heading is not an extra Tab stop when the close control is available. A body
closure's explicit focus request is honored. `initial_focus: Some(actual_id)`
selects a particular enabled native control instead. It must be an actual egui
widget ID, not an unhashed Dutchy input salt. For example, a native
`TextEdit::id(egui::Id::unique("editor-name"))` can share that ID with the option.

Return focus defaults to the previously focused control. Prefer setting
`return_focus: Some(trigger_response.id)` when opening by pointer, or provide a
surviving replacement if the opener disappears. This target can change while the
overlay is open. Restoration waits until egui releases the modal layer, skips
missing/disabled targets, and preserves an explicit host move to another control.
Allow subsequent frames after closing; conditionally omitting the component
prevents this lifecycle from completing. No overlay state is serialized for you.

## Reference decisions and scope

Reviewed the web checkout at `104e4dab59fd94130b44fb276736c6be731e1ebf`:
`docs/01-design-system/components/modal.md`, `src/components/{Modal,Drawer}/index.tsx`,
`public/assets/js/{modal,drawer}.js`, and the component registry. Both source
surfaces use native dialogs; the port uses native `egui::Modal` rather than
emulating modality with a SidePanel. The 50% backdrop uses the theme's foreground
color, so the dark theme lightens the workspace beneath it, consistent with the
semantic source token. There are no copied third-party implementations or new
runtime dependencies. Existing font/third-party notices remain distributed.

Top/bottom or resizable/docked drawers, draggable dialogs, exit transitions, and
full web/browser parity are outside this step. Platform and screen-reader limits
are recorded in [verification](verification-overlays.md) and
[DS-016](issues/DS-016.md).
