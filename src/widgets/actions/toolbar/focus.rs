use crate::widgets::navigation::{self, Axis};
use egui::{Id, Ui};

#[derive(Clone, Default)]
pub(crate) struct Memory {
    pub tab: Option<Id>,
    pub previous: Vec<Id>,
    pub open: Option<Id>,
    pub popup_ids: Vec<Id>,
}

impl Memory {
    pub fn prepare(&mut self, ui: &Ui, ids: &[Id], axis: Axis, fallback: Option<Id>) {
        if !ui.is_enabled() || !ui.memory(|m| m.allows_interaction(ui.layer_id())) {
            return;
        }
        let focused = ui.memory(|m| m.focused());
        if self.open.is_some_and(|id| !ids.contains(&id)) {
            if focused.is_some_and(|id| self.popup_ids.contains(&id)) {
                let next = self
                    .tab
                    .filter(|id| ids.contains(id))
                    .or_else(|| ids.first().copied())
                    .or(fallback);
                ui.memory_mut(|m| {
                    if let Some(next) = next {
                        m.request_focus(next);
                    } else if let Some(old) = focused {
                        m.surrender_focus(old);
                    }
                });
            }
            self.open = None;
            self.popup_ids.clear();
        }
        if let Some(old) = focused.filter(|id| self.previous.contains(id) && !ids.contains(id)) {
            let index = self.previous.iter().position(|id| *id == old).unwrap_or(0);
            let next = ids
                .get(index.min(ids.len().saturating_sub(1)))
                .copied()
                .or(fallback);
            ui.memory_mut(|m| {
                if let Some(next) = next {
                    m.request_focus(next);
                } else {
                    m.surrender_focus(old);
                }
            });
        }
        navigation::navigate(ui, ids, axis);
        self.remember(ui, ids);
    }
    pub fn remember(&mut self, ui: &Ui, ids: &[Id]) {
        if let Some(id) = ui.memory(|m| m.focused()).filter(|id| ids.contains(id)) {
            self.tab = Some(id);
        }
        self.tab = self
            .tab
            .filter(|id| ids.contains(id))
            .or_else(|| ids.first().copied());
        self.previous = ids.to_vec();
    }
}
