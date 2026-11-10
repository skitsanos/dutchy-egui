use crate::{
    Menu, MenuItem, MenuResponse, MenuWidth, Theme,
    tokens::{border, menu as geometry},
    widgets::{menu::paint, popup_focus},
};
use egui::{Id, IdSalt, Key, Modifiers, Pos2, Response, Ui, UiBuilder};
use std::borrow::Cow;

/// Preferred horizontal alignment; placement may flip at viewport edges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropdownAlign {
    /// Align the popup's leading edge with its trigger.
    #[default]
    Start,
    /// Align the popup's trailing edge with its trigger.
    End,
}

/// Trigger, popup geometry, and host-owned action result.
#[non_exhaustive]
pub struct DropdownResponse<T> {
    /// Original trigger, retaining its native role and accessible name.
    pub trigger: Response,
    /// Visible popup response, if open.
    pub popup: Option<Response>,
    /// Visible menu and item responses.
    pub menu: Option<MenuResponse<T>>,
    /// Selected action. The host executes it.
    pub action: Option<T>,
}

#[derive(Clone, Copy)]
enum Edge {
    First,
    Last,
}
#[derive(Clone, Default)]
struct Memory {
    last_frame: u64,
    was_open: bool,
    pending: Option<Edge>,
    items: Vec<Id>,
    trigger: Option<Id>,
}

/// Anchored action menu with host-owned open state.
#[must_use = "Call show or show_context to render the dropdown"]
pub struct Dropdown<'a, T> {
    id: IdSalt,
    open: &'a mut bool,
    items: &'a [MenuItem<'a, T>],
    width: MenuWidth,
    align: DropdownAlign,
    name: Cow<'a, str>,
    enabled: bool,
    bar: bool,
    attached: bool,
}
impl<'a, T: Clone> Dropdown<'a, T> {
    /// A 200-point popup. The caller supplies any native or Dutchy trigger.
    pub fn new(id: impl egui::AsIdSalt, open: &'a mut bool, items: &'a [MenuItem<'a, T>]) -> Self {
        Self {
            id: IdSalt::new(id),
            open,
            items,
            width: MenuWidth::Compact,
            align: DropdownAlign::default(),
            name: Cow::Borrowed("Actions"),
            enabled: true,
            bar: false,
            attached: false,
        }
    }
    /// Preferred popup width.
    pub fn width(mut self, width: MenuWidth) -> Self {
        self.width = width;
        self
    }
    /// Preferred edge alignment.
    pub fn align(mut self, align: DropdownAlign) -> Self {
        self.align = align;
        self
    }
    /// Menu accessibility name; the trigger keeps its own name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable opening and menu interaction, closing an open popup. Triggers
    /// created by show are disabled; existing context targets remain host-owned.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Click/Enter/Space toggle. Down/Up open at the first/last enabled item.
    /// Escape restores trigger focus; Tab exits to the neighboring host control.
    pub fn show(
        self,
        ui: &mut Ui,
        trigger: impl FnOnce(&mut Ui) -> Response,
    ) -> DropdownResponse<T> {
        let enabled = self.enabled;
        self.render(
            ui,
            |ui, root| {
                ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
                    if !enabled {
                        ui.disable();
                    }
                    trigger(ui)
                })
                .inner
            },
            false,
        )
    }
    /// Attach a context menu to an existing response. Secondary click or
    /// Shift+F10 opens it at a fixed pointer/trigger position. Tab dismisses
    /// back to this existing target; the next Tab follows normal host order.
    pub fn show_context(self, ui: &mut Ui, response: &Response) -> DropdownResponse<T> {
        self.render(ui, |_, _| response.clone(), true)
    }
    /// Attach to an already drawn, focusable menu button. Its name is retained.
    /// Tab closes the entire menu tree and follows the button's host tab order.
    /// The attachment allocates no additional host layout space.
    pub fn show_for(self, ui: &mut Ui, response: &Response) -> DropdownResponse<T> {
        let attached = Self {
            attached: true,
            ..self
        };
        attached.render(ui, |_, _| response.clone(), false)
    }
    pub(crate) fn in_menu_bar(mut self) -> Self {
        self.bar = true;
        self
    }

    fn render(
        self,
        ui: &mut Ui,
        trigger: impl FnOnce(&mut Ui, Id) -> Response,
        context: bool,
    ) -> DropdownResponse<T> {
        let root = ui.make_persistent_id(("dutchy-dropdown", self.id));
        let memory_id = root.with("transient-focus");
        let mut memory = ui
            .ctx()
            .data(|data| data.get_temp::<Memory>(memory_id).unwrap_or_default());
        let frame = ui.ctx().cumulative_frame_nr();
        if memory.last_frame + 1 < frame {
            memory.was_open = false;
            memory.pending = None;
        }
        memory.last_frame = frame;
        let initially_open = *self.open;
        let routed_tab = !context && prepare_tab_exit(ui, &memory, initially_open, self.attached);
        let mut trigger = trigger(ui, root);
        memory.trigger = Some(trigger.id);
        let open_edge = update_open_state(&trigger, root, context, self.enabled, self.open);
        if *self.open && (!memory.was_open || open_edge.is_some()) {
            memory.pending = Some(open_edge.unwrap_or(Edge::First));
        }
        let tab = *self.open && (routed_tab || popup_focus::tab_direction(ui).is_some());
        if tab {
            *self.open = false;
            if context {
                popup_focus::consume_tab(ui);
                ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            }
        }
        let mut popup = None;
        let mut menu = None;
        if *self.open {
            let width = self.width.points().min(ui.ctx().content_rect().width());
            let mut native = egui::Popup::from_response(&trigger)
                .id(root.with("popup"))
                .open_bool(self.open)
                .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                .gap(if self.bar {
                    geometry::ATTACHED_GAP
                } else {
                    geometry::POPUP_GAP
                })
                .width(width)
                .frame(paint::frame(Theme::from_context(ui.ctx())))
                .accessible_name(self.name.to_string())
                .align(match self.align {
                    DropdownAlign::Start => egui::RectAlign::BOTTOM_START,
                    DropdownAlign::End => egui::RectAlign::BOTTOM_END,
                });
            if context
                && let Some(position) = ui
                    .ctx()
                    .data(|data| data.get_temp::<Pos2>(root.with("position")))
            {
                native = native.at_position(position);
            }
            let shown = native.show(|ui| {
                ui.set_width((width - border::STANDARD * 2.0).max(0.0));
                let result =
                    egui::ScrollArea::vertical()
                        .max_height(geometry::DROPDOWN_MAX_HEIGHT.min(
                            (ui.ctx().content_rect().height() - border::STANDARD * 2.0).max(0.0),
                        ))
                        .show(ui, |ui| {
                            Menu::new("actions", self.items)
                                .width(self.width)
                                .in_menu_bar(self.bar)
                                .keyboard_open(open_edge.is_some())
                                .accessible_name(self.name.as_ref())
                                .show_unframed(ui, root.with("menu"))
                        })
                        .inner;
                focus_opening_item(ui, &mut memory, &result.items);
                memory.items = result.focus_ids.clone();
                result
            });
            if let Some(result) = shown {
                popup = Some(result.response);
                menu = Some(result.inner);
            }
        }
        let escape =
            initially_open && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        let outside = memory.was_open
            && ui.input(|i| i.pointer.any_click())
            && ui.ctx().pointer_interact_pos().is_some_and(|pos| {
                !trigger.rect.contains(pos)
                    && !popup
                        .as_ref()
                        .is_some_and(|r: &Response| r.rect.contains(pos))
                    && !menu
                        .as_ref()
                        .is_some_and(|m| m.popup_rects.iter().any(|r| r.contains(pos)))
            });
        if escape || outside {
            *self.open = false;
        }
        let action = menu.as_ref().and_then(|menu| menu.action.clone());
        if action.is_some() {
            *self.open = false;
        }
        restore_focus(
            &trigger,
            &memory,
            *self.open,
            tab && !context,
            escape || (tab && context),
            action.is_some(),
        );
        memory.was_open = *self.open;
        if !*self.open {
            memory.pending = None;
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(memory_id, memory));
        trigger.ctx.accesskit_node_builder(trigger.id, |node| {
            node.set_has_popup(egui::accesskit::HasPopup::Menu);
            node.set_expanded(*self.open);
            if *self.open
                && let Some(menu) = &menu
            {
                node.push_controlled(menu.response.id.accesskit_id());
            }
        });
        if *self.open {
            crate::commands::dispatch::mark_overlay(ui.ctx());
        }
        if initially_open != *self.open {
            trigger.mark_changed();
            ui.ctx().request_repaint();
        }
        DropdownResponse {
            trigger,
            popup,
            menu,
            action,
        }
    }
}

fn restore_focus(
    trigger: &Response,
    memory: &Memory,
    open: bool,
    tab: bool,
    escape: bool,
    action: bool,
) {
    if tab {
        return;
    }
    let focused_item = trigger
        .ctx
        .memory(|m| m.focused())
        .is_some_and(|id| memory.items.contains(&id));
    if open
        || !(escape || tab || action || (memory.was_open && focused_item))
        || !trigger.enabled()
        || !trigger.sense.is_focusable()
    {
        return;
    }
    trigger.request_focus();
}

fn prepare_tab_exit(ui: &Ui, memory: &Memory, open: bool, attached: bool) -> bool {
    let Some(trigger) = memory.trigger else {
        return false;
    };
    let owns_focus = ui
        .memory(|mem| mem.focused())
        .is_some_and(|id| id == trigger || memory.items.contains(&id));
    if !open || !owns_focus {
        return false;
    }
    let Some(direction) = popup_focus::tab_direction(ui) else {
        return false;
    };
    popup_focus::leave(ui, trigger, direction, attached);
    true
}

fn opening_key(trigger: &Response, context: bool) -> Option<Edge> {
    if !trigger.enabled() || !trigger.has_focus() {
        return None;
    }
    trigger
        .ctx
        .input_mut(|input| {
            if context {
                input
                    .consume_key(Modifiers::SHIFT, Key::F10)
                    .then_some(Edge::First)
            } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                Some(Edge::First)
            } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                Some(Edge::Last)
            } else {
                None
            }
        })
        .inspect(|_| {
            trigger
                .ctx
                .memory_mut(|memory| memory.move_focus(egui::FocusDirection::None))
        })
}

fn focus_opening_item(ui: &Ui, memory: &mut Memory, items: &[Response]) {
    if ui.is_sizing_pass() {
        return;
    }
    let Some(edge) = memory.pending.take() else {
        return;
    };
    let mut enabled = items.iter().filter(|response| response.enabled());
    let target = match edge {
        Edge::First => enabled.next(),
        Edge::Last => enabled.next_back(),
    };
    if let Some(target) = target {
        crate::widgets::menu::memory::focus(target);
    }
}

fn update_open_state(
    trigger: &Response,
    root: Id,
    context: bool,
    enabled: bool,
    open: &mut bool,
) -> Option<Edge> {
    if !enabled || !trigger.enabled() {
        *open = false;
        return None;
    }
    let edge = opening_key(trigger, context);
    let clicked = if context {
        trigger.secondary_clicked()
    } else {
        trigger.clicked()
    };
    if clicked || edge.is_some() {
        *open = context || edge.is_some() || !*open;
        if context {
            let pos = trigger
                .ctx
                .pointer_latest_pos()
                .filter(|_| clicked)
                .unwrap_or(trigger.rect.left_bottom());
            trigger
                .ctx
                .data_mut(|data| data.insert_temp(root.with("position"), pos));
        }
    }
    edge
}
