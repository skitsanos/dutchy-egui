use egui::{Context, Id, Ui};

/// Transient measurements, keyed by cell identity rather than display position.
/// Each cell closure still runs only once per egui pass.
#[derive(Clone, Default)]
pub(super) struct Heights(Vec<(Id, f32)>);

const KEY: &str = "cell-heights";

/// Drop measurements of rows that left the view, so paging never accumulates them.
pub(super) fn evict_stale(ctx: &Context, root: Id, current: &[Id]) {
    let key = root.with("measured-rows");
    let previous: Vec<Id> = ctx.data(|data| data.get_temp(key)).unwrap_or_default();
    ctx.data_mut(|data| {
        for stale in previous.iter().filter(|id| !current.contains(id)) {
            data.remove_temp::<Heights>(stale.with(KEY));
        }
        data.insert_temp(key, current.to_vec());
    });
}

impl Heights {
    pub fn load(ui: &Ui, ids: &[Id]) -> Self {
        let mut measured: Self = ui
            .ctx()
            .data(|data| data.get_temp(ui.scope_id().with(KEY)).unwrap_or_default());
        measured.0.retain(|(id, _)| ids.contains(id));
        measured
    }

    pub fn offset(&self, id: Id) -> f32 {
        let height = self.0.iter().map(|(_, height)| *height).fold(0.0, f32::max);
        self.0
            .iter()
            .find(|(cell, _)| *cell == id)
            .map_or(0.0, |(_, cell)| (height - cell) / 2.0)
    }

    pub fn save(self, ui: &Ui, measured: Vec<(Id, f32)>) {
        // Ignore subpixel rounding noise; offsets must not feed back into the
        // natural height measurement or rows will grow on successive passes.
        let tolerance = 0.5 / ui.pixels_per_point();
        let changed = self.0.len() != measured.len()
            || self
                .0
                .iter()
                .zip(&measured)
                .any(|(old, new)| old.0 != new.0 || (old.1 - new.1).abs() > tolerance);
        ui.ctx()
            .data_mut(|data| data.insert_temp(ui.scope_id().with(KEY), Self(measured)));
        if changed {
            ui.ctx()
                .request_discard("Dutchy table cell heights changed");
            if !ui.ctx().will_discard() {
                ui.ctx().request_repaint();
            }
        }
    }
}
