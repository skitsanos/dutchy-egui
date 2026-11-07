# File selection, uploads and drag-and-drop

`FileUpload` returns browse requests and lazy `egui::DroppedFileHandle`s.
`FileRules` validates metadata. `FileList` presents a keyed collection and returns
remove, cancel and retry requests. The host owns files, selection, byte reads,
transfer jobs, progress and business actions. No runtime dependency is added.

## Pick or drop

```rust
use dutchy_egui::{FileUpload, FileUploadVariant};

fn select(ui: &mut egui::Ui) {
    let result = FileUpload::new("attachments")
        .label("Drop project files")
        .hint("PDF or images · up to 10 MB each")
        .show(ui);
    if result.browse_requested {
        // Open the host's native/browser picker.
        // Retain result.response.id and return focus when the picker closes.
    }
    for file in result.dropped {
        // Queue this lazy handle in the host; do not read contents in the UI pass.
        // native: file.path(), then worker I/O; web: file.bytes_async().await.
    }
    FileUpload::new("compact")
        .variant(FileUploadVariant::Compact)
        .label("Choose files")
        .show(ui);
}
```

The whole surface is a native clickable/focusable Button, with the original
label as its accessible name and instructions/limits as its description.
Enter/Space and click set the same `browse_requested` result. Sharp dashed
borders, centered wrapped text, a decorative upload glyph, a muted hover/press/
drag fill (live `bg-muted/30`) and an inset keyboard focus cue use shared
tokens. Padding (32 points), the 24-point glyph and the bold 14-point uppercase
body instruction follow the live component. Compact uses a solid border and a
wrapped label on a full-width 56-point surface where the web uses an inline
control; that remains a documented difference. Parent/explicit disabled state
blocks events.

Drop routing requires a visible, enabled target under the current pointer and
native egui layer/modal checks. It never routes a viewport-wide file event to
an arbitrary control when the pointer is absent/outside. Only the first eligible
claim consumes `input.raw.dropped_files`; a per-viewport frame guard prevents
repeated handling across discarded layout passes. Use distinct stable IDs.
`drag_hovered` highlights external files over the target but adds no selection.
Other host drop consumers should coordinate with this consumption rule.

The actual backend must deliver hovered/dropped files and current pointer
position. Enable drag-and-drop in eframe's viewport builder for platforms that
require it; the native showcase does so explicitly. OS drop-coordinate delivery
and browser permission behavior are separate platform acceptance concerns.
There is no internal file queue or hidden reader/picker in the widget.

## Metadata policy

```rust
use dutchy_egui::{FileInfo, FileRules};
let candidates = [FileInfo::new("brief.PDF").size(2_400_000)];
let rules = FileRules::new()
    .extensions(["pdf", "png", "jpg"])
    .mime_types(["image/*"])
    .max_bytes(10_000_000)
    .max_files(5);
let result = rules.validate(&candidates, 2); // two files already retained
assert_eq!(result.accepted, [0]);
```

Default rules are unrestricted. Extensions are case-insensitive, optionally
start with a dot, and may be compound (`tar.gz`). Extension and MIME rules form
an OR list. MIME parameters/case are normalized; exact types and `type/*` match.
Empty names are rejected. With a byte limit, an unknown size is rejected rather
than treated as zero; exact boundaries and empty files are accepted. Count limits
include the supplied retained count; invalid candidates consume no capacity.
Results preserve input order and refer to candidate indices. Zero limits mean
zero capacity/bytes, not unlimited. Duplicate identity stays a host decision.

`FileInfo::from_path` extracts a basename without touching the filesystem.
Neither the widget nor the rules stat a file or sniff its contents. The host
supplies metadata off the UI thread where inspection may block. Names/MIME/picker
filters are presentation metadata, not proof of contents or a server policy.
`file_size` uses decimal units and keeps unknown size explicit.

## Controlled list and transfer states

`FileItem::new(key, name)` adds optional bytes, a `FileStatus`, enabled actions
and remove/cancel availability. Keys must stay unique and immutable through
reordering, renaming and status changes. Keep actual handles/paths in a host map.
Names wrap, including unbroken filenames; decorative file icons are omitted and
actions stack when columns become very narrow.

`FileStatus::Ready` shows readiness; `Uploading(Some(percent))` shows clamped
0..100 progress, `Uploading(None)` an indeterminate shared Progress component,
`Complete` a successful track, and `Failed(message)` an error with Retry.
Progress retains native range/busy metadata and reduced-motion/clipped behavior.
The collection has List/ListItem roles; actions include the full file name.

`FileList::new(id, &items).show(ui)` returns independent `remove_requested`,
`cancel_requested` and `retry_requested` keys. The X requests Cancel while
uploading and Remove in other states; it uses shared close-control hover/pressed/
focus feedback. The host decides when to remove data or cancel/retry work.
Names use the medium weight, sizes are plain monospaced text, transfer tracks
are 8 points and dismiss glyphs 16 points, following the live rows.
Disabled controls preserve readable metadata; hiding remove does not hide retry.
No action infers completion or mutates files automatically.

After actual host removal, focus moves to the next available remove/cancel
control (or the previous endpoint). `.empty_focus(picker_id)` supplies a fallback
when no remove/cancel control remains. Continue calling the list with an empty slice to
finish this return. Explicit host focus changes elsewhere are preserved. A host
that removes the whole list subtree owns its replacement focus decision. The
return keys on the focused control no longer being enabled, so a row that is
disabled after Cancel is treated like a removed one; a host that keeps such a
row and wants focus to stay on it moves focus itself.
A focused action is revealed once when focus arrives or layout moves it;
scrolling away from it is never undone.

## Showcase and scope

The File Uploads page opens the current stable [rfd 0.17.2](https://docs.rs/rfd/0.17.2/rfd/) picker on the native
UI thread. This is a development-only integration, not a runtime library helper.
File metadata inspection runs on a worker and wakes egui on completion. Picker
and dropped native paths share rules for PDF/PNG/JPEG/TXT, ten MB and five files.
Directories and inspection failures stay explicit; duplicate paths are reported.
Selection cancellation retains the collection and returns focus to its opener.

Add sample metadata or select local files, then start the clearly labelled
**demo upload**. Progress, one-time interruption, Retry, Cancel and removal are
host-controlled simulation. No file contents are read or sent to a network.
Real applications connect their own I/O/transport and cancellation lifecycle.
Browser hosts can retain egui handles and use async byte reads or their own
browser picker; this native demonstration is not a browser upload adapter.

Reviewed the web `components/file-upload.md` and `src/components/FileUpload` at
revision `104e4dab59fd94130b44fb276736c6be731e1ebf`. Repository-owned implementations
adapt the source's flat dashed zone, compact input and removable progress rows.
No third-party widget/helper code was copied. Transport, folder traversal,
chunk/resume protocols, previews and content verification remain host concerns.
See [DS-024](issues/DS-024.md) and [verification](verification-file-uploads.md).
