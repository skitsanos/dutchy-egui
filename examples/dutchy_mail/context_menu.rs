//! Bind a popup to a visible message row and capture its intended targets.

use crate::{
    commands::{Action, Request, context_items},
    mailbox::{FolderId, MessageId},
};
use dutchy_egui::{CommandRegistry, Dropdown, ListResponse, ListState, MenuWidth};
use egui::Ui;

#[derive(Default)]
pub struct ContextMenu {
    active: Option<Target>,
}

struct Target {
    row: MessageId,
    messages: Vec<MessageId>,
}

impl ContextMenu {
    pub fn close(&mut self) {
        self.active = None;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        folder: FolderId,
        registry: &CommandRegistry<Action>,
        state: &mut ListState<MessageId>,
        list: &ListResponse<MessageId>,
    ) -> Option<Request> {
        let visible = || {
            list.rows
                .iter()
                .filter(|row| list.viewport.intersect(row.response.rect).is_positive())
        };
        let secondary = visible().find(|row| row.response.secondary_clicked());
        let previous = self.active.take();
        let mut open = previous.is_some();
        let (row, target) = if let Some(row) = secondary {
            // Make this row the focus return point without opening it for reading.
            state.focused = Some(row.key);
            row.response.request_focus();
            open = false;
            (row, Target::capture(row.key, state))
        } else if let Some(target) = previous {
            let row = visible().find(|row| row.key == target.row)?;
            if row.response.clicked() {
                return None;
            }
            (row, target)
        } else {
            let row = visible().find(|row| row.response.has_focus())?;
            (row, Target::capture(row.key, state))
        };

        let items = context_items(registry, ui.ctx(), folder);
        let result = Dropdown::new(("mail-context", folder, row.key), &mut open, &items)
            .width(MenuWidth::Small)
            .accessible_name("Message context actions")
            .show_context(ui, &row.response);
        if let Some(action) = result.action {
            return Some(Request::Context {
                action,
                folder,
                messages: target.messages,
            });
        }
        self.active = open.then_some(target);
        None
    }
}

impl Target {
    fn capture(row: MessageId, state: &ListState<MessageId>) -> Self {
        let messages = if state.selected.contains(&row) {
            state.selected.iter().copied().collect()
        } else {
            vec![row]
        };
        Self { row, messages }
    }
}
