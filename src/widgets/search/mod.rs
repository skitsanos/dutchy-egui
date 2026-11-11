mod keys;
mod options;
mod rank;
mod results;
pub(crate) mod tree;

use egui::{Context, Id, IdSalt};
pub(crate) use keys::{SearchKeys, input_metadata};
pub(crate) use options::{option_items, options_popup};
pub(crate) use results::{Results, ResultsResponse, Selection};

/// Host-owned open state and editable query for searchable selection/commands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SearchState {
    /// Whether results/dialog are open.
    pub open: bool,
    /// Editable search text. Selection/action execution is separate.
    pub query: String,
}

#[derive(Clone, Default)]
pub(crate) struct Cursor {
    pub active: Option<IdSalt>,
    query: String,
    pub was_open: bool,
}
impl Cursor {
    pub fn load(ctx: &Context, root: Id) -> Self {
        ctx.data(|data| {
            data.get_temp(root.with("search-cursor"))
                .unwrap_or_default()
        })
    }
    pub fn store(self, ctx: &Context, root: Id) {
        ctx.data_mut(|data| data.insert_temp(root.with("search-cursor"), self));
    }
    pub fn update(&mut self, query: &str, enabled: &[IdSalt], keys: &SearchKeys) {
        let fresh = !self.was_open || self.query != query;
        if fresh || self.active.is_none_or(|id| !enabled.contains(&id)) {
            self.active = enabled.first().copied();
        }
        if !enabled.is_empty()
            && let Some(movement) = keys.movement
        {
            let current = self
                .active
                .and_then(|id| enabled.iter().position(|entry| *entry == id))
                .unwrap_or(0);
            let index = match movement {
                keys::Movement::First => 0,
                keys::Movement::Last => enabled.len() - 1,
                keys::Movement::Next if fresh => 0,
                keys::Movement::Previous if fresh => enabled.len() - 1,
                keys::Movement::Next => (current + 1) % enabled.len(),
                keys::Movement::Previous => (current + enabled.len() - 1) % enabled.len(),
            };
            self.active = Some(enabled[index]);
        }
        self.query = query.into();
        self.was_open = true;
    }
}
