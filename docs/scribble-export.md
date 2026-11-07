# Scribble and export

`Scribble` binds a stable widget ID to a host-owned `ScribbleDrawing`. Finished
strokes, brushes, undo/redo and clear history live in that model. egui stores only
the temporary gesture/cursor. There is no widget picker, filesystem write,
network request, automatic download or backend upload.

```rust
use dutchy_egui::{Scribble, ScribbleAction, ScribbleDrawing};

fn drawing_ui(ui: &mut egui::Ui, drawing: &mut ScribbleDrawing) {
    let result = Scribble::new("signature", drawing).label("Signature").show(ui);
    for action in result.actions {
        match action {
            ScribbleAction::Undo => { drawing.undo(); }
            ScribbleAction::Redo => { drawing.redo(); }
            ScribbleAction::Clear => { drawing.clear(); }
            _ => { /* Capture drawing.snapshot() and start a host export job. */ }
        }
    }
    if let Some(error) = result.error {
        // Present the error in the host; completed data remains intact.
        eprintln!("{error}");
    }
}
```

## Canvas and input

The default logical canvas is 600×200. Dimensions are immutable, from 1 to 4096
per side; the displayed rectangle fits the available column at the same aspect.
Stored coordinates and pen width do not depend on UI width or pixels per point.
The live canvas is pure white; the port's paper is the warm light background
(#FAF8F5) so exports and the canvas share one token. The surface uses shared
light-paper and dark-ink tokens in every theme so a
signature remains legible. Outer borders and controls follow the host theme.
New strokes capture their brush; later theme/brush changes do not recolor old ink.
Four pen tokens provide widths 1, 2, 4 and 8. Alpha colors use ordinary source-over.

The primary pointer (including backend-translated primary touch) starts a stroke;
ordered motion/release events preserve coalesced samples. Clicks make dots.
Leaving the visible canvas commits at the last valid point; reentry needs a fresh
press. Other pointer buttons do not draw. Window focus loss, external model
changes, disabling or switching read-only cancel incomplete gestures. Moving or
resizing the canvas cancels pointer gestures; keyboard drafts retain their logical
coordinates across layout changes.
A brush change during a gesture applies only to the next stroke.

The named, focusable Canvas exposes stroke/point counts and keyboard help.
Editable canvases retain arrow focus; Tab leaves the canvas normally:

- Arrows move the pen by four drawing units; Shift+arrows moves sixteen.
- Space starts/finishes a stroke; Enter adds a dot or finishes a stroke.
- Escape cancels the draft and native egui releases focus. Refocus to resume.
- Delete/Backspace requests Clear. Ctrl/Cmd+Z requests Undo; Shift+Ctrl/Cmd+Z or
  Ctrl/Cmd+Y requests Redo. During an active keyboard draft these shortcuts
  only cancel the draft; the next press reaches history. Delete/Backspace cancel
  the draft and request Clear.

Focus reveals a canvas once; handled keyboard drawing commands reveal it again
when needed. Keeping focus does not override manual scrolling. Toolbar controls
reveal on focus changes/activation, allowing wheel/scrollbar navigation to later
views. Empty read-only views say “No drawing”; `.placeholder(...)` overrides either
mode's default hint. See [DS-026](issues/DS-026.md).

The toolbar returns Undo, Redo, Clear, Preview, ExportSvg and optional ExportPng
requests; the host executes them in order. Completed pointer/keyboard strokes
edit the bound model directly and report `strokes_added`/changed responses.
`action_response` identifies an activated toolbar control for native-picker focus
return. Distinct stable IDs are required for different views of the same model.

Read-only preserves inspection/export but blocks drawing/history. Disabled and
parent-disabled states block all actions. Clip/layer/modal checks reject obscured
pointer presses. `.controls(false)` hides the toolbar, while editable keyboard
shortcuts remain. `.exports_enabled(false)` blocks new export requests during a
host job. Essential keyboard help is visible by default and stays accessible
when `.instructions(false)` hides the visible text.

## History, snapshots and exports

`ScribbleDrawing::push` validates finite in-bounds points before changing anything.
There are at most 512 strokes, 8192 points per stroke and 65,536 completed points.
A gesture that reaches a point limit is committed with the points captured so
far and reports `ScribbleError::Capacity`; only a stroke that cannot be stored
at all is dropped.
Rejected edits retain data and redo history. Clear is one undoable operation;
empty clear does nothing. History retains 32 operations using shared immutable
point buffers. New edits discard redo. `snapshot()` shares completed point buffers
without retaining history; background jobs remain unaffected by later edits.
Persistence/serialization, filename policy and storage stay with the host.

```rust
use dutchy_egui::{ScribbleDrawing, ScribbleExportOptions};
let drawing = ScribbleDrawing::default();
let snapshot = drawing.snapshot();
let options = ScribbleExportOptions::default(); // 1x, transparent
let svg: String = snapshot.to_svg(options).unwrap();
let preview: egui::ColorImage = snapshot.rasterize(options).unwrap();
// With `png-export`: let png: Vec<u8> = snapshot.to_png(options).unwrap();
```

SVG is static, self-contained vector output with round caps/joins and a generic
accessible title; dots become circles. CPU raster output uses analytic edge
coverage and composites each stroke once, including semitransparent joins. PNG
encodes that same RGBA raster. Export includes completed ink and optional
background, excluding borders, labels, buttons, placeholders and cursors. Set
`background: Some(tokens::scribble::PAPER)` for paper; None retains transparency.

Resolution multiplies logical size, rounded to whole pixels. Each side must be
1..8192 pixels. Rasters additionally cap at 16,777,216 pixels and 100 million
estimated sampling units, checked before allocating the output/masks. Reduce
scale or use SVG when raster work exceeds the budget. This is a bounded signature
pad, not an unbounded illustration editor. Run substantial raster/encoding jobs
off the UI thread. SVG/CPU export and native epaint rendering have independent
edge rasterization; pixel-identical screenshot/export parity is not promised.

`png-export` adds the existing optional image 0.25.10 codec with PNG only. It does
not enable HTTP/JPEG or bundled fonts. SVG and CPU previews need no extra runtime
crate; the default dependency remains egui. `url-images` still enables HTTP(S)
PNG/JPEG independently. egui 0.37.0, image 0.25.10 and development-only rfd 0.17.2
were confirmed as current stable releases through crates.io on 2026-11-06.

## Showcase and reference

Run `cargo run --example showcase --features png-export` and choose Scribble.
The host owns drawing/history, accent/width choices, transparent/paper exports,
sample data and a shared read-only view. Preview/raster encoding and file writes
run on a worker; native rfd Save As runs on the UI thread. The host activates the window and defers focus return until the
export finishes and its opener is enabled again; explicit intervening focus
changes are preserved. Save cancellation leaves the drawing untouched; failures
are explicit.
A retained preview identifies its revision and warns when the drawing changes.

Reviewed web `components/scribble.md`, `src/components/Scribble/index.tsx` and
`public/assets/js/scribble.js` at revision
`104e4dab59fd94130b44fb276736c6be731e1ebf`. The port preserves freehand/dot drawing,
clear, a sharp paper surface and PNG output, while adding vector export, bounded
host history and keyboard drawing. Controls sit below the canvas to avoid hiding
ink. No third-party widget/helper code was copied. Pressure/tilt, multitouch,
eraser, undo persistence, smoothing and arbitrary shape tools are outside scope.
See [DS-025](issues/DS-025.md) and [verification](verification-scribble.md).
