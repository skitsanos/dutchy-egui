use egui::{Context, Id, LayerId, Response};
#[derive(Clone, Default)]
pub(super) struct Memory {
    outside: Option<Id>,
    controls: Vec<Id>,
}
impl Memory {
    pub fn load(ctx: &Context, root: Id) -> Self {
        ctx.data(|data| data.get_temp(root.with("focus")).unwrap_or_default())
    }
    pub fn capture(&mut self, ctx: &Context, layer: LayerId) {
        if let Some(response) = ctx
            .memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            && response.layer_id != layer
        {
            self.outside = Some(response.id);
        }
    }
    pub fn finish(
        &mut self,
        ctx: &Context,
        root: Id,
        remaining: Vec<Id>,
        return_focus: Option<Id>,
    ) {
        let removed_focus = ctx
            .memory(|m| m.focused())
            .is_some_and(|id| self.controls.contains(&id) && !remaining.contains(&id));
        if removed_focus {
            let target = return_focus
                .into_iter()
                .chain(remaining.iter().copied())
                .chain(self.outside)
                .find(|id| {
                    ctx.read_response(*id).is_some_and(|r| {
                        r.enabled() && ctx.memory(|m| m.allows_interaction(r.layer_id))
                    })
                });
            if let Some(target) = target {
                ctx.memory_mut(|m| m.request_focus(target));
                ctx.request_repaint();
            }
        }
        self.controls = remaining;
        ctx.data_mut(|data| data.insert_temp(root.with("focus"), self.clone()));
    }
    pub fn register(&mut self, responses: impl Iterator<Item = Response>) {
        self.controls.extend(responses.map(|response| response.id));
    }
}
