use super::{InlineEditEvent, InlineEditState, geometry, gesture, input, view};
use egui::{
    Id, Rect, Response, TextEdit, Ui,
    text::{CCursor, CCursorRange},
};

pub(super) fn edit(
    ui: &mut Ui,
    rect: Rect,
    state: &mut InlineEditState,
    options: view::Options<'_>,
) -> (Response, Option<InlineEditEvent>) {
    let slots = geometry::Slots::new(rect, options.size);
    // The editor's scope is explicit so identity survives compact layout.
    let editor_id = view::editor_id(ui);
    let actions = view::action_ids(ui);
    let intent = input::edit(ui, editor_id, actions);
    if state.focus_pending && ui.is_enabled() {
        select_all(ui, editor_id, state.draft().unwrap_or_default(), true);
        ui.memory_mut(|m| m.request_focus(editor_id));
        state.focus_pending = false;
    }
    gesture::swallow_space(ui, [editor_id, actions[0], actions[1]]);
    let response = view::editor(
        ui,
        slots.editor,
        state.draft.as_mut().expect("open draft"),
        options,
    );
    let [save, cancel] = view::actions(ui, &slots, options);
    let opening_click = gesture::pointer_target(ui, [&response, &save, &cancel]);
    if opening_click.is_some() {
        select_all(ui, editor_id, state.draft().unwrap_or_default(), false);
        response.request_focus();
        ui.ctx().request_repaint();
    }
    input::lock_escape(ui, response.id, true);
    for action in [&save, &cancel] {
        input::lock_escape(ui, action.id, false);
    }
    let clicked = |action: &Response| action.clicked() && opening_click != Some(action.id);
    let event = if clicked(&cancel) || intent == Some(input::Intent::Cancel) {
        state.cancel();
        Some(InlineEditEvent::Cancelled)
    } else if !options.read_only && (clicked(&save) || intent == Some(input::Intent::Commit)) {
        let value = state.draft.take().expect("open draft");
        state.cancel();
        Some(InlineEditEvent::Commit(value))
    } else {
        None
    };
    if event.is_some() {
        input::consume_activation(ui);
        gesture::disarm(ui);
        state.focus_pending = true;
        ui.ctx().request_repaint();
    }
    (response, event)
}

/// Select the whole draft; a fresh session also starts with empty undo history.
fn select_all(ui: &Ui, editor: Id, draft: &str, fresh: bool) {
    let mut memory = TextEdit::load_state(ui.ctx(), editor).unwrap_or_default();
    let end = draft.chars().count();
    memory
        .cursor
        .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(end))));
    if fresh {
        memory.clear_undoer();
    }
    memory.store(ui.ctx(), editor);
}

/// Return focus to the label only if nothing outside the component took it
/// while the label was hidden or disabled.
pub(super) fn may_return_focus(ui: &Ui, trigger: Id) -> bool {
    let own = [trigger, view::editor_id(ui)];
    let actions = view::action_ids(ui);
    ui.memory(|m| m.focused())
        .is_none_or(|id| own.contains(&id) || actions.contains(&id))
}
