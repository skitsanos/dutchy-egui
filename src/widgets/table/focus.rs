use egui::{Id, IdSalt, LayerId, Rect, Ui};
#[derive(Clone)]
struct OwnedFocus {
    widget: Id,
    row: IdSalt,
    column: IdSalt,
}
#[derive(Clone)]
struct Cell {
    row: IdSalt,
    column: IdSalt,
    rect: Rect,
    layer: LayerId,
}
#[derive(Clone, Default)]
pub(super) struct Focus {
    focused: Option<OwnedFocus>,
    cells: Vec<Cell>,
}
impl Focus {
    pub fn load(ui: &Ui, root: Id) -> Self {
        let mut memory: Self = ui
            .ctx()
            .data(|data| data.get_temp(root.with("cell-focus")).unwrap_or_default());
        // egui resolves input against preceding-pass rectangles. Resolve ownership
        // before a reorder/removal moves another cell into that old rectangle.
        if let Some(response) = ui
            .memory(|m| m.focused())
            .and_then(|id| ui.ctx().read_response(id))
            && let Some(cell) = memory.cells.iter().find(|cell| {
                cell.layer == response.layer_id && cell.rect.contains_rect(response.rect)
            })
        {
            memory.focused = Some(OwnedFocus {
                widget: response.id,
                row: cell.row,
                column: cell.column,
            });
        }
        memory.cells.clear();
        memory
    }
    pub fn capture(&mut self, ui: &Ui, row: IdSalt, column: IdSalt) {
        let rect = ui.min_rect();
        if let Some(response) = ui
            .memory(|m| m.focused())
            .and_then(|id| ui.ctx().read_response(id))
            && self
                .focused
                .as_ref()
                .is_none_or(|owned| owned.widget != response.id)
            && response.layer_id == ui.layer_id()
            && rect.contains_rect(response.rect)
        {
            self.focused = Some(OwnedFocus {
                widget: response.id,
                row,
                column,
            });
        }
        self.cells.push(Cell {
            row,
            column,
            rect,
            layer: ui.layer_id(),
        });
    }
    pub fn finish(
        self,
        ui: &Ui,
        root: Id,
        rows: &[IdSalt],
        columns: &[IdSalt],
        target: Option<Id>,
    ) {
        if let Some(focus) = &self.focused {
            let removed = !rows.contains(&focus.row) || !columns.contains(&focus.column);
            if removed
                && ui.memory(|m| m.focused()) == Some(focus.widget)
                && let Some(target) = target.and_then(|id| ui.ctx().read_response(id))
                && target.enabled()
                && ui.memory(|m| m.allows_interaction(target.layer_id))
            {
                target.request_focus();
                ui.ctx().request_repaint();
            }
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(root.with("cell-focus"), self));
    }
}
