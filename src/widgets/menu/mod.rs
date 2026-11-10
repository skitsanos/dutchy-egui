mod item;
mod measure;
pub(crate) mod memory;
pub(crate) mod paint;
mod render;

use crate::{MenuWidth, Theme, tokens::border};
use egui::{Id, IdSalt, Response, Ui, UiBuilder, WidgetInfo};
pub use item::{MenuIconPainter, MenuItem, MenuItemVariant};
use std::borrow::Cow;

/// Requested host change. Checked values are snapshots, never mutated by menus.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MenuEvent<T> {
    /// Execute the named host command.
    Activated(T),
    /// Accept or reject a requested checked state.
    Checked {
        /// Stable host command value.
        value: T,
        /// Requested new value.
        checked: bool,
    },
}

/// Named action results, with no application action executed by the widget.
pub struct MenuResponse<T> {
    /// Whole menu response.
    pub response: Response,
    /// Action chosen by pointer, Enter, Space, or an assistive click.
    pub action: Option<T>,
    /// Item responses in source order, including disabled items.
    pub items: Vec<Response>,
    /// Typed leaf activation or proposed checked value.
    pub event: Option<MenuEvent<T>>,
    pub(crate) focus_ids: Vec<Id>,
    pub(crate) popup_rects: Vec<egui::Rect>,
    pub(crate) direction: i8,
}

/// Grouped action list with stable item IDs and native interaction.
#[must_use = "Call show to render the menu"]
pub struct Menu<'a, T> {
    id: IdSalt,
    items: &'a [MenuItem<'a, T>],
    width: MenuWidth,
    enabled: bool,
    name: Cow<'a, str>,
    bar: bool,
    keyboard_open: bool,
}
impl<'a, T: Clone> Menu<'a, T> {
    /// A 320-point menu, constrained to its parent.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [MenuItem<'a, T>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            width: MenuWidth::default(),
            enabled: true,
            name: Cow::Borrowed("Actions"),
            bar: false,
            keyboard_open: false,
        }
    }
    /// Preferred menu width.
    pub fn width(mut self, width: MenuWidth) -> Self {
        self.width = width;
        self
    }
    /// Menu's accessible name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable the menu, respecting disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render an inline menu. Up/Down/Home/End move among enabled items.
    /// Shortcut strings are hints; they never register application key bindings.
    pub fn show(self, ui: &mut Ui) -> MenuResponse<T> {
        let root = ui.make_persistent_id(("dutchy-menu", self.id));
        let width = self.width.points().min(ui.available_width());
        self.render(ui, root, Some(width))
    }
    pub(crate) fn in_menu_bar(mut self, bar: bool) -> Self {
        self.bar = bar;
        self
    }
    pub(crate) fn keyboard_open(mut self, keyboard: bool) -> Self {
        self.keyboard_open = keyboard;
        self
    }
    pub(crate) fn show_unframed(self, ui: &mut Ui, root: Id) -> MenuResponse<T> {
        self.render(ui, root, None)
    }

    fn render(self, ui: &mut Ui, root: Id, framed_width: Option<f32>) -> MenuResponse<T> {
        crate::widgets::popup_focus::track(ui.ctx());
        let memory_id = root.with("tree-memory");
        let mut memory = ui
            .ctx()
            .data(|d| d.get_temp::<memory::Memory>(memory_id))
            .unwrap_or_default();
        memory.direction = 0;
        if self.enabled && ui.is_enabled() {
            memory.prepare(ui, root, self.items, self.bar);
            if self.keyboard_open {
                memory.keyboard();
            }
        } else {
            memory.path.clear();
            memory.levels.clear();
        }
        let mut tree = render::Tree {
            width: self.width,
            memory,
            focus_ids: Vec::new(),
            rects: vec![egui::Rect::NOTHING],
        };
        let result = ui.scope_builder(
            UiBuilder::new()
                .scope_id(root)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                ui.visuals_mut().override_text_color = None;
                ui.spacing_mut().item_spacing.y = 0.0;
                if let Some(width) = framed_width {
                    ui.set_width(width.max(0.0));
                    paint::frame(Theme::from_context(ui.ctx()))
                        .show(ui, |ui| {
                            ui.set_width((width - border::STANDARD * 2.0).max(0.0));
                            tree.level(ui, root, self.items, 0)
                        })
                        .inner
                } else {
                    tree.level(ui, root, self.items, 0)
                }
            },
        );
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Menu,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        tree.rects[0] = result.response.rect;
        if result.inner.0.is_some() {
            if let Some(parent) = tree
                .memory
                .path
                .first()
                .and_then(|id| result.inner.1.iter().find(|r| r.id == *id))
            {
                memory::focus(parent);
            }
            tree.memory.path.clear();
            tree.memory.pending = None;
        }
        if ui.input(|i| i.pointer.any_click())
            && ui
                .ctx()
                .pointer_interact_pos()
                .is_some_and(|pos| !tree.rects.iter().any(|r| r.contains(pos)))
        {
            tree.memory.path.clear();
        }
        tree.memory.rects = tree.rects.clone();
        ui.ctx()
            .data_mut(|d| d.insert_temp(memory_id, tree.memory.clone()));
        render::finish(result.response, result.inner.0, result.inner.1, &tree)
    }
}
