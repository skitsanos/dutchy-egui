//! Shared typed-value editing: while an editor has focus its text lives in
//! temporary memory and each change parses it; losing focus reformats the text
//! from the committed value.

use egui::{Context, Id, TextEdit, Ui};

/// Check before handling keys: egui drops disabled focus during widget layout,
/// which happens after the typed controls' stepping handlers.
pub(super) fn editable_focus(ui: &Ui, id: Id) -> bool {
    ui.is_enabled() && ui.memory(|memory| memory.has_focus(id))
}

fn key(id: Id) -> Id {
    id.with("dutchy-typed-text")
}

/// The text to edit this frame: the retained draft while focused, else `formatted`.
pub(super) fn draft(ctx: &Context, id: Id, formatted: &str) -> String {
    if ctx.memory(|memory| memory.has_focus(id)) {
        ctx.data(|data| data.get_temp::<String>(key(id)))
            .unwrap_or_else(|| formatted.to_owned())
    } else {
        formatted.to_owned()
    }
}

/// Retain the draft while the editor keeps focus; forget it once focus leaves.
pub(super) fn retain(ctx: &Context, id: Id, focused: bool, text: &str) {
    if focused {
        ctx.data_mut(|data| data.insert_temp(key(id), text.to_owned()));
    } else {
        ctx.data_mut(|data| data.remove_temp::<String>(key(id)));
    }
}

/// Zero-based index of the `separator`-delimited segment under the caret; the
/// last segment when the editor has no stored cursor yet.
pub(super) fn caret_segment(ctx: &Context, id: Id, text: &str, separator: char) -> usize {
    let caret = TextEdit::load_state(ctx, id)
        .and_then(|state| state.cursor.char_range())
        .map_or(text.chars().count(), |range| range.primary.index.into());
    text.chars()
        .take(caret)
        .filter(|character| *character == separator)
        .count()
}
