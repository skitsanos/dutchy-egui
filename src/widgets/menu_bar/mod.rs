mod item;
use crate::{
    ButtonVariant, ControlSize, Dropdown, MenuEvent, MenuResponse, MenuWidth, Theme,
    tokens::{actions as tokens, border},
    widgets::actions::{core, toolbar::focus},
    widgets::popup_focus,
};
use egui::{
    Id, IdSalt, Key, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo,
    accesskit::{HasPopup, Orientation, Role},
};
pub use item::{MenuBarItem, MenuBarState};
use std::borrow::Cow;

/// In-window application menu geometry and a single host command request.
#[non_exhaustive]
pub struct MenuBarResponse<T> {
    /// Named MenuBar container.
    pub response: Response,
    /// Top-level trigger responses in source order.
    pub items: Vec<Response>,
    /// Open command menu, including its nested result.
    pub menu: Option<MenuResponse<T>>,
    /// Leaf value for existing value-based action handlers.
    pub action: Option<T>,
    /// Typed activation or checked-state request.
    pub event: Option<MenuEvent<T>>,
}

#[derive(Clone, Default)]
struct Memory {
    focus: focus::Memory,
    popup_ids: Vec<Id>,
    trigger: Option<Id>,
}

/// A native egui application menu bar, rendered inside the application window.
#[must_use = "Call show to render the menu bar"]
pub struct MenuBar<'a, T> {
    id: IdSalt,
    state: &'a mut MenuBarState,
    items: &'a [MenuBarItem<'a, T>],
    name: Cow<'a, str>,
    enabled: bool,
    width: MenuWidth,
    fallback: Option<Id>,
}
impl<'a, T: Clone> MenuBar<'a, T> {
    /// Bind host-owned top-level visibility and stable menu entries.
    pub fn new(
        id: impl egui::AsIdSalt,
        state: &'a mut MenuBarState,
        items: &'a [MenuBarItem<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            state,
            items,
            name: "Application menus".into(),
            enabled: true,
            width: MenuWidth::Small,
            fallback: None,
        }
    }
    /// Accessible application/group name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable all menus without enabling a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Preferred command-popup width, constrained by the viewport.
    pub fn menu_width(mut self, width: MenuWidth) -> Self {
        self.width = width;
        self
    }
    /// Host focus destination if all previously focused menus disappear.
    pub fn focus_when_empty(mut self, id: Id) -> Self {
        self.fallback = Some(id);
        self
    }
    /// Render one Tab stop, horizontal navigation and controlled command popups.
    pub fn show(mut self, ui: &mut Ui) -> MenuBarResponse<T> {
        let root = ui.make_persistent_id(("dutchy-menu-bar", self.id));
        let mut memory = ui
            .ctx()
            .data(|d| d.get_temp::<Memory>(root))
            .unwrap_or_default();
        let shown = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            let enabled: Vec<_> = self
                .items
                .iter()
                .filter(|i| i.interactive() && ui.is_enabled())
                .map(|i| root.with(i.id))
                .collect();
            self.prepare(ui, &enabled, &mut memory);
            memory.focus.prepare(
                ui,
                &enabled,
                crate::widgets::navigation::Axis::Horizontal,
                self.fallback,
            );
            let p = Theme::from_context(ui.ctx()).palette;
            let bar = egui::Frame::NONE
                .fill(p.background)
                .stroke(Stroke::new(border::STANDARD, p.border))
                .inner_margin(egui::Margin::same(tokens::PADDING as i8))
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = None;
                    ui.set_min_width(ui.available_width());
                    egui::ScrollArea::horizontal()
                        .id_salt("menu-bar-overflow")
                        .content_margin(egui::Margin::ZERO)
                        .max_height(ControlSize::Small.height())
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = tokens::GAP;
                                self.items
                                    .iter()
                                    .map(|item| self.trigger(ui, root, item, &memory))
                                    .collect::<Vec<_>>()
                            })
                            .inner
                        })
                        .inner
                });
            let triggers = bar.inner;
            let menu = self.popup(ui, root, &triggers, bar.response.rect.bottom(), &mut memory);
            memory.focus.remember(ui, &enabled);
            (triggers, menu)
        });
        shown.response.widget_info(|| {
            WidgetInfo::labeled(Role::MenuBar, shown.response.enabled(), self.name.as_ref())
        });
        shown
            .response
            .ctx
            .accesskit_node_builder(shown.response.id, |n| {
                n.set_orientation(Orientation::Horizontal)
            });
        ui.ctx().data_mut(|d| d.insert_temp(root, memory));
        let action = shown.inner.1.as_ref().and_then(|m| m.action.clone());
        let event = shown.inner.1.as_ref().and_then(|m| m.event.clone());
        MenuBarResponse {
            response: shown.response,
            items: shown.inner.0,
            menu: shown.inner.1,
            action,
            event,
        }
    }
    fn prepare(&mut self, ui: &Ui, enabled: &[Id], memory: &mut Memory) {
        let valid = self
            .items
            .iter()
            .any(|i| Some(i.id) == self.state.open && i.interactive() && ui.is_enabled());
        let owns_popup = ui
            .memory(|m| m.focused())
            .is_some_and(|id| memory.popup_ids.contains(&id));
        if !valid {
            self.state.close();
            if owns_popup {
                let next = memory
                    .focus
                    .tab
                    .filter(|id| enabled.contains(id))
                    .or_else(|| enabled.first().copied())
                    .or(self.fallback);
                ui.memory_mut(|m| {
                    if let Some(id) = next {
                        m.request_focus(id);
                    } else if let Some(id) = m.focused() {
                        m.surrender_focus(id);
                    }
                });
            }
        }
        if owns_popup && let Some(direction) = popup_focus::tab_direction(ui) {
            self.state.close();
            if let Some(id) = memory.trigger {
                popup_focus::move_past(ui, id, direction);
            }
        }
    }
    fn trigger(
        &self,
        ui: &mut Ui,
        root: Id,
        item: &MenuBarItem<'_, T>,
        memory: &Memory,
    ) -> Response {
        let id = root.with(item.id);
        let response = core::show(
            ui,
            core::Action {
                id,
                label: &item.label,
                icon: None,
                icon_only: false,
                pressed: None,
                variant: if self.state.open == Some(item.id) {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                },
                size: ControlSize::Small,
                enabled: item.interactive(),
                loading: false,
                tooltip: None,
                inside_focus: true,
                menu: false,
                role: Role::MenuItem,
            },
            if memory.focus.tab == Some(id) {
                Sense::click()
            } else {
                Sense::CLICK
            },
            None,
        );
        response.ctx.accesskit_node_builder(id, |n| {
            n.set_has_popup(HasPopup::Menu);
            n.set_expanded(self.state.open == Some(item.id));
        });
        if response.has_focus() {
            crate::widgets::menu::memory::lock(&response, false);
        }
        crate::interaction::reveal_focused(ui, &response, ui.max_rect().min);
        response
    }
    fn popup(
        &mut self,
        ui: &mut Ui,
        root: Id,
        triggers: &[Response],
        anchor_bottom: f32,
        memory: &mut Memory,
    ) -> Option<MenuResponse<T>> {
        let activate = triggers.iter().position(|r| {
            r.enabled()
                && (r.clicked()
                    || (r.has_focus()
                        && ui.input(|i| {
                            i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowUp)
                        })))
        });
        let hover = self.state.open.and_then(|_| {
            triggers.iter().position(|r| {
                r.enabled()
                    && r.hovered()
                    && ui.input(|i| {
                        i.events
                            .iter()
                            .any(|e| matches!(e, egui::Event::PointerMoved(_)))
                    })
            })
        });
        let selected = activate.or(hover).or_else(|| {
            self.items
                .iter()
                .position(|i| Some(i.id) == self.state.open)
        })?;
        let item = &self.items[selected];
        let mut open = self.state.open == Some(item.id) || (activate.is_none() && hover.is_some());
        let mut anchor = triggers[selected].clone();
        anchor.interact_rect.max.y = anchor_bottom - border::STANDARD;
        let result = Dropdown::new(root.with(item.id), &mut open, item.items)
            .in_menu_bar()
            .width(self.width)
            .enabled(item.interactive() && ui.is_enabled())
            .accessible_name(item.label.as_ref())
            .show_for(ui, &anchor);
        self.state.open = open.then_some(item.id);
        memory.trigger = Some(triggers[selected].id);
        memory.focus.tab = Some(triggers[selected].id);
        memory.popup_ids = result
            .menu
            .as_ref()
            .map_or_else(Vec::new, |m| m.focus_ids.clone());
        if let Some(menu) = &result.menu
            && menu.direction != 0
        {
            self.switch(ui, selected, menu.direction, triggers);
        }
        result.menu
    }
    fn switch(&mut self, ui: &Ui, selected: usize, direction: i8, triggers: &[Response]) {
        let enabled: Vec<_> = triggers
            .iter()
            .enumerate()
            .filter(|(_, r)| r.enabled())
            .map(|(i, _)| i)
            .collect();
        if let Some(index) = enabled.iter().position(|i| *i == selected) {
            let next =
                (index as isize + direction as isize).rem_euclid(enabled.len() as isize) as usize;
            let target = enabled[next];
            if target == selected {
                return; // The only enabled menu: keep its focused row.
            }
            self.state.open = Some(self.items[target].id);
            triggers[target].request_focus();
            ui.ctx().request_repaint();
        }
    }
}
