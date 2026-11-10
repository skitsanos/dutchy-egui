# MediaCard and SearchInput

Both components use native egui interaction, repository-owned modules and
shared tokens; the showcase combines them in an artwork browser. No dependencies
are added. The host owns queries, filtering, sorting, saved state,
navigation, retries and any keyboard shortcuts.

```rust
use dutchy_egui::{MediaCard, SearchInput};

# fn example(ui: &mut egui::Ui, query: &mut String, saved: &mut bool,
#            source: egui::ImageSource<'_>) {
let edited = ui.add(SearchInput::new("artwork-search", query)
    .accessible_name("Search artworks")
    .placeholder("Styles, moods or techniques…")
    .key_hint("Ctrl+K"));
if edited.changed() {
    // Recompute application-owned filtered results.
}

let card = MediaCard::new("canal", source, "Canal houses illustration", "Canal houses")
    .category("Architecture")
    .description("Warm color and a quiet architectural rhythm.")
    .saved(*saved)
    .show(ui);
if let Some(next) = card.save_requested {
    *saved = next;
}
if card.open_requested {
    // Open the application's editor, details or lightbox.
}
# }
```

## SearchInput contract

`SearchInput::new(id, &mut query)` is a Widget returning the native editor's
Response. Add a decorative magnifier, conditional Clear button and optional
Kbd hint. Preserve text selection, IME, clipboard, Field validation, char limits
and native single-line Enter behavior. Clearing is an edit, returns focus to
the editor and sets changed. Disabled/read-only parents cannot clear or type.
Names and IDs remain stable when siblings change. Hints advertise a host-owned
shortcut; the widget does not bind one or consume Escape. Omit decorative slots
as width shrinks while retaining usable text and clear controls.
Standalone clearing is named `Clear {accessible_name}`. The key hint and
magnifier are decorative; they add no extra keyboard stops. Clicking decorative
margins focuses the editor. A Field can supply the editor's visible label.

## MediaCard contract

`MediaCard::new(id, source, alt, title)` composes a bordered Card, Image,
category/title/description and a directional affordance. Preview and metadata
return one open request; the optional saved toggle returns its desired next
state without mutating host data. Save and retry must never open the card.
Use stable IDs, native pointer/keyboard/assistive actions, explicit names and
focus-visible cues. Show the image action on hover and keyboard focus. Keep
normal image loading, failure, optional retry, lazy URI loading and reduced
motion; preserve aspect through contain/cover. Text wraps within its column and
stays left-aligned even inside justified host layouts such as `ui.columns`.
Titles are capped at two visible rows and wrap at the width reserved beside the
direction arrow; descriptions are capped at three rows; accessible names retain
the full title. Omitting `.saved(...)` removes the bookmark control.
The response includes the existing ImageResponse: inspect its status or
`image.retry_requested` and perform the host's retry policy. Sources work under
the same loader contract as Image; URI support is not installed by MediaCard.
The primary image action is keyboard-visible. Other card parts do not capture
bookmark or retry clicks, and retained focus does not pull manual scrolling back.
MediaCard has no custom metadata slots or masonry layout.

## Showcase

The showcase's **Media cards** page demonstrates host filtering, category and
saved-only filters, sorting, explicit counts and Lightbox focus return. Its
Cmd/Ctrl+K binding is application code. **Search input** demonstrates all three
control sizes and read-only/disabled states. Gallery illustrations are original,
generated in-repository and independent of the ignored video media directory.
