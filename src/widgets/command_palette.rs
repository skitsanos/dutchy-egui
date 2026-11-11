use crate::{
    Input, MenuItem, SearchState, Theme,
    tokens::{border, menu as geometry, space},
    widgets::{
        menu::paint,
        search::{Cursor, Results, SearchKeys, input_metadata},
    },
};
use egui::{Context, Id, IdSalt, Key, KeyboardShortcut, Modifiers, Response, UiBuilder};
use std::borrow::Cow;

/// Dialog response and a command for the host to execute.
#[non_exhaustive]
pub struct CommandPaletteResponse<T> {
    /// Native modal response/geometry.
    pub response: Response,
    /// Editable command search input.
    pub input: Response,
    /// Activated command, if any. No command is executed by the library.
    pub action: Option<T>,
    /// Whether this frame closed the dialog.
    pub dismissed: bool,
}

#[derive(Clone, Default)]
struct Memory {
    was_open: bool,
    return_focus: Option<Id>,
    pending_focus: bool,
    input: Option<Id>,
    restore_pending: bool,
}

/// Modal command search using native egui input blocking and focus confinement.
/// Call once per frame, including while closed, to support the optional shortcut.
#[must_use = "Call show to render or handle the palette shortcut"]
pub struct CommandPalette<'a, T> {
    id: IdSalt,
    state: &'a mut SearchState,
    items: &'a [MenuItem<'a, T>],
    name: Cow<'a, str>,
    shortcut: Option<KeyboardShortcut>,
}
impl<'a, T: Clone> CommandPalette<'a, T> {
    /// Bind a globally unique palette ID, host query/open state, and commands.
    /// The default shortcut is logical Command+K (Ctrl+K off macOS).
    pub fn new(
        id: impl egui::AsIdSalt,
        state: &'a mut SearchState,
        items: &'a [MenuItem<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            state,
            items,
            name: Cow::Borrowed("Command palette"),
            shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::K)),
        }
    }
    /// Dialog name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Change or disable automatic shortcut handling; the host can set open itself.
    pub fn shortcut(mut self, shortcut: Option<KeyboardShortcut>) -> Self {
        self.shortcut = shortcut;
        self
    }
    /// Filter commands, navigate with Up/Down/Home/End, activate with Enter, or
    /// dismiss with Escape/backdrop. Closing restores the preceding host focus.
    pub fn show(self, ctx: &Context) -> Option<CommandPaletteResponse<T>> {
        let leaves = crate::widgets::search::tree::leaves(self.items);
        let root = Id::unique(("dutchy-command-palette", self.id));
        let key = root.with("transient-focus");
        let mut memory = ctx.data(|data| data.get_temp::<Memory>(key).unwrap_or_default());
        if let Some(shortcut) = self.shortcut {
            let another_modal = ctx
                .memory(|memory| memory.top_modal_layer())
                .is_some_and(|layer| layer.id != root);
            if !another_modal && ctx.input_mut(|input| input.consume_shortcut(&shortcut)) {
                self.state.open = !self.state.open;
            }
        }
        if self.state.open && !memory.was_open {
            memory.restore_pending = false;
            memory.return_focus = ctx.memory(|memory| memory.focused());
            memory.pending_focus = true;
        }
        if !self.state.open {
            if memory.was_open && (ctx.memory(|m| m.focused()) == memory.input) {
                memory.restore_pending = true;
            }
            if memory.restore_pending {
                match ctx.memory(|m| m.top_modal_layer()) {
                    None => {
                        restore(ctx, memory.return_focus);
                        memory.restore_pending = false;
                        ctx.request_repaint();
                    }
                    // egui reports the palette's own layer for one more pass.
                    Some(layer) if layer.id == root => ctx.request_repaint(),
                    // Another modal took over; focus now belongs to it.
                    Some(_) => memory.restore_pending = false,
                }
            }
            memory.was_open = false;
            ctx.data_mut(|data| data.insert_temp(key, memory));
            let mut cursor = Cursor::load(ctx, root);
            cursor.was_open = false;
            cursor.store(ctx, root);
            return None;
        }
        let theme = Theme::from_context(ctx);
        let width =
            geometry::PALETTE_WIDTH.min((ctx.content_rect().width() - space::LG * 2.0).max(0.0));
        let frame = egui::Frame::NONE
            .fill(theme.palette.background)
            .stroke(egui::Stroke::new(border::ACCENT, theme.palette.border));
        let mut cursor = Cursor::load(ctx, root);
        let modal = egui::Modal::new(root)
            .area(egui::Modal::default_area(root).accessible_name(self.name.to_string()))
            .frame(frame)
            .backdrop_color(
                theme
                    .palette
                    .foreground
                    .gamma_multiply(geometry::BACKDROP_OPACITY),
            )
            .show(ctx, |ui| {
                ui.set_width((width - border::ACCENT * 2.0).max(0.0));
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.scope_builder(UiBuilder::new().scope_id(root.with("content")), |ui| {
                    let input = egui::Frame::NONE
                        .inner_margin(egui::Margin::same(geometry::SEARCH_PADDING))
                        .show(ui, |ui| {
                            let keys = SearchKeys::take(
                                ui,
                                ui.make_persistent_id(IdSalt::new("query")),
                                true,
                            );
                            let input = ui.add(
                                Input::new("query", &mut self.state.query)
                                    .accessible_name("Search commands")
                                    .placeholder("Type a command or search…"),
                            );
                            if memory.pending_focus && !ui.is_sizing_pass() {
                                input.request_focus();
                                memory.pending_focus = false;
                            }
                            (input, keys)
                        })
                        .inner;
                    memory.input = Some(input.0.id);
                    paint::divider(ui, true);
                    let height = geometry::COMMANDS_HEIGHT.min(
                        (ctx.content_rect().height()
                            - input.0.rect.height()
                            - f32::from(geometry::SEARCH_PADDING) * 2.0
                            - border::ACCENT * 2.0
                            - space::LG * 2.0)
                            .max(0.0),
                    );
                    let results = Results {
                        items: &leaves,
                        query: &self.state.query,
                        name: self.name.as_ref(),
                        selection: crate::widgets::search::Selection::Single(None),
                        height,
                    }
                    .show(ui, root.with("results"), &mut cursor, &input.1);
                    input_metadata(&input.0, true, Some(results.response.id), results.active);
                    (input.0, input.1.escape, results.action)
                })
                .inner
            });
        ctx.accesskit_node_builder(root, |node| {
            node.set_modal();
            node.set_bounds(egui::accesskit::Rect {
                x0: modal.response.rect.min.x.into(),
                y0: modal.response.rect.min.y.into(),
                x1: modal.response.rect.max.x.into(),
                y1: modal.response.rect.max.y.into(),
            });
        });
        let dismissed = modal.should_close() || modal.inner.1 || modal.inner.2.is_some();
        if dismissed {
            self.state.open = false;
            memory.restore_pending = true;
            ctx.request_repaint();
        }
        memory.was_open = self.state.open;
        ctx.data_mut(|data| data.insert_temp(key, memory));
        cursor.was_open = self.state.open;
        cursor.store(ctx, root);
        Some(CommandPaletteResponse {
            response: modal.response,
            input: modal.inner.0,
            action: modal.inner.2,
            dismissed,
        })
    }
}

fn restore(ctx: &Context, id: Option<Id>) {
    if let Some(id) = id {
        ctx.memory_mut(|memory| memory.request_focus(id));
    }
}
