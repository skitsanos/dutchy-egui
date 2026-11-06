use egui::{Id, Ui};
#[derive(Clone, Copy)]
struct Pending {
    origin: Id,
    index: usize,
}
pub(super) fn remember(ui: &Ui, root: Id, origin: Id, index: usize) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(root.with("pending-focus"), Pending { origin, index }));
}
pub(super) fn restore(ui: &Ui, root: Id, controls: &[Id], empty: Option<Id>) {
    let key = root.with("pending-focus");
    let Some(pending) = ui.ctx().data(|data| data.get_temp::<Pending>(key)) else {
        return;
    };
    if controls.contains(&pending.origin) {
        return;
    }
    ui.ctx().data_mut(|data| data.remove::<Pending>(key));
    let current = ui.memory(|memory| memory.focused());
    if current.is_some_and(|id| id != pending.origin) {
        return;
    }
    let target = controls
        .get(pending.index.min(controls.len().saturating_sub(1)))
        .copied()
        .or(empty);
    if let Some(target) = target {
        ui.memory_mut(|memory| memory.request_focus(target));
    }
}
