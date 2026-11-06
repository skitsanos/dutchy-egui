use super::OverlayOptions;
use egui::{Context, Id, LayerId};
#[derive(Clone, Default)]
pub(in crate::widgets) struct Memory {
    was_open: bool,
    pending_entry: bool,
    return_focus: Option<Id>,
    restore_pending: bool,
}
impl Memory {
    pub fn load(ctx: &Context, root: Id) -> Self {
        ctx.data(|data| {
            data.get_temp(root.with("focus-lifecycle"))
                .unwrap_or_default()
        })
    }
    pub fn store(self, ctx: &Context, root: Id) {
        ctx.data_mut(|data| data.insert_temp(root.with("focus-lifecycle"), self));
    }
    pub fn enter(&mut self, ctx: &Context, options: &OverlayOptions) -> bool {
        let opening = !self.was_open;
        if opening {
            self.return_focus = options.return_focus.or_else(|| ctx.memory(|m| m.focused()));
            self.pending_entry = true;
            self.restore_pending = false;
        }
        if let Some(target) = options.return_focus {
            self.return_focus = Some(target);
        }
        self.was_open = true;
        opening
    }
    pub fn focus(&mut self, ctx: &Context, layer: LayerId, fallback: Id, options: &OverlayOptions) {
        if !self.pending_entry {
            return;
        }
        if ctx.memory(|m| m.top_modal_layer().is_none_or(|top| top == layer)) {
            let inside = ctx
                .memory(|m| m.focused())
                .and_then(|id| ctx.read_response(id))
                .is_some_and(|r| r.layer_id == layer);
            if let Some(id) = options.initial_focus {
                ctx.memory_mut(|m| m.request_focus(id));
            } else if !inside {
                ctx.memory_mut(|m| m.request_focus(fallback));
            }
            self.pending_entry = false;
        }
        ctx.request_repaint();
    }
    pub fn closed(&mut self, ctx: &Context, layer: LayerId) {
        if self.was_open {
            self.restore_pending = true;
        }
        self.was_open = false;
        self.pending_entry = false;
        if !self.restore_pending {
            return;
        }
        // egui's modal layer is from the preceding pass. Wait until it is gone.
        if ctx.memory(|m| m.top_modal_layer()) == Some(layer) {
            ctx.request_repaint();
            return;
        }
        let focused = ctx.memory(|m| m.focused());
        let host_moved_focus = focused
            .and_then(|id| ctx.read_response(id))
            .is_some_and(|r| r.layer_id != layer && Some(r.id) != self.return_focus);
        if !host_moved_focus
            && let Some(id) = self.return_focus
            && let Some(target) = ctx.read_response(id)
            && target.enabled()
            && ctx.memory(|m| m.allows_interaction(target.layer_id))
        {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        self.restore_pending = false;
        ctx.request_repaint();
    }
}
