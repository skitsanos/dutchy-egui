use super::{
    MenuEvent, MenuItem, MenuResponse, measure,
    memory::{self, Memory},
    paint,
};
use crate::{
    MenuWidth, Text, TextColor, TextSize, Theme, Tracking, Weight,
    tokens::{border, menu as geometry},
};
use egui::{
    Id, Response, Ui, UiBuilder, WidgetInfo,
    accesskit::{HasPopup, Role},
};

pub(super) struct Tree {
    pub width: MenuWidth,
    pub memory: Memory,
    pub focus_ids: Vec<Id>,
    pub rects: Vec<egui::Rect>,
}

impl Tree {
    pub fn level<T: Clone>(
        &mut self,
        ui: &mut Ui,
        root: Id,
        items: &[MenuItem<'_, T>],
        depth: usize,
    ) -> (Option<MenuEvent<T>>, Vec<Response>) {
        let mut event = None;
        let mut responses = Vec::with_capacity(items.len());
        let check_slot = items
            .iter()
            .any(|i| i.checked.is_some() && i.children.is_none());
        for (index, group) in items.chunk_by(|a, b| a.group == b.group).enumerate() {
            if index > 0 {
                paint::divider(ui, true);
            }
            let shown = ui.scope_builder(
                UiBuilder::new().scope_id(root.with(("group", group[0].id))),
                |ui| {
                    heading(ui, group[0].group.as_deref());
                    for (index, item) in group.iter().enumerate() {
                        if index > 0 || item.separator_before {
                            paint::divider(ui, item.separator_before);
                        }
                        let response = ui
                            .add_enabled_ui(item.interactive(), |ui| {
                                paint::row(
                                    ui,
                                    root.with(item.id),
                                    item,
                                    paint::RowState {
                                        role: Role::MenuItem,
                                        active: false,
                                        selected: false,
                                        branch_open: self.memory.path.get(depth)
                                            == Some(&root.with(item.id)),
                                        focusable: true,
                                        check_slot,
                                        pointer: self.memory.pointer_mode(),
                                    },
                                )
                            })
                            .inner;
                        memory::lock(&response, true);
                        self.memory
                            .pointer(ui, &response, depth, item.children.is_some());
                        if response.clicked() && item.children.is_none() {
                            event = Some(match item.checked {
                                Some(checked) => MenuEvent::Checked {
                                    value: item.value.clone(),
                                    checked: !checked,
                                },
                                None => MenuEvent::Activated(item.value.clone()),
                            });
                            response.request_focus();
                        }
                        if response.gained_focus() {
                            response
                                .scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
                        }
                        self.focus_ids.push(response.id);
                        responses.push(response);
                    }
                },
            );
            if let Some(label) = &group[0].group {
                shown.response.widget_info(|| {
                    WidgetInfo::labeled(Role::Group, ui.is_enabled(), label.as_ref())
                });
            }
        }
        self.memory.focus(ui, depth, &responses);
        for (item, response) in items.iter().zip(&responses) {
            if let Some(children) = item.children {
                let open = self.memory.path.get(depth) == Some(&response.id) && response.enabled();
                let submenu = open
                    .then(|| self.submenu(ui, response, item.label.as_ref(), children, depth + 1))
                    .flatten();
                response.ctx.accesskit_node_builder(response.id, |node| {
                    node.set_has_popup(HasPopup::Menu);
                    node.set_expanded(open);
                    if let Some((_, menu)) = &submenu {
                        node.push_controlled(menu.id.accesskit_id());
                    }
                });
                if let Some((child_event, _)) = submenu {
                    event = event.or(child_event);
                }
            }
        }
        (event, responses)
    }

    fn submenu<T: Clone>(
        &mut self,
        ui: &Ui,
        response: &Response,
        label: &str,
        children: &[MenuItem<'_, T>],
        depth: usize,
    ) -> Option<(Option<MenuEvent<T>>, Response)> {
        let width = measure::child_width(ui, children, self.width);
        let mut anchor = response.clone();
        anchor.interact_rect.min.y -= border::STANDARD;
        anchor.interact_rect.max.y += border::STANDARD;
        let shown = egui::Popup::from_response(&anchor)
            .id(response.id.with("submenu-popup"))
            .open(true)
            .accessible_name(label)
            .align(egui::RectAlign::RIGHT_START)
            .gap(geometry::ATTACHED_GAP)
            .width(width)
            .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
            .frame(paint::frame(Theme::from_context(ui.ctx())))
            .show(|ui| {
                ui.visuals_mut().override_text_color = None;
                ui.set_width((width - border::STANDARD * 2.0).max(0.0));
                ui.spacing_mut().item_spacing.y = 0.0;
                let menu = ui.scope_builder(
                    UiBuilder::new().scope_id(response.id.with("children-menu")),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("submenu-overflow")
                            .max_height(
                                geometry::DROPDOWN_MAX_HEIGHT.min(
                                    (ui.ctx().content_rect().height() - border::STANDARD * 2.0)
                                        .max(0.0),
                                ),
                            )
                            .show(ui, |ui| {
                                self.level(ui, response.id.with("children"), children, depth)
                            })
                            .inner
                            .0
                    },
                );
                menu.response
                    .widget_info(|| WidgetInfo::labeled(Role::Menu, response.enabled(), label));
                (menu.inner, menu.response)
            })?;
        if self.rects.len() <= depth {
            self.rects.resize(depth + 1, egui::Rect::NOTHING);
        }
        self.rects[depth] = shown.response.rect;
        Some(shown.inner)
    }
}

fn heading(ui: &mut Ui, label: Option<&str>) {
    if let Some(label) = label {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(
                geometry::PADDING_X as i8,
                geometry::GROUP_PADDING,
            ))
            .show(ui, |ui| {
                ui.add(
                    Text::new(label)
                        .size(TextSize::Xs)
                        .weight(Weight::Bold)
                        .tracking(Tracking::Wider)
                        .uppercase(true)
                        .color(TextColor::Muted),
                )
            });
    }
}

pub(super) fn finish<T: Clone>(
    response: Response,
    event: Option<MenuEvent<T>>,
    items: Vec<Response>,
    tree: &Tree,
) -> MenuResponse<T> {
    let action = event.as_ref().map(|event| match event {
        MenuEvent::Activated(value) | MenuEvent::Checked { value, .. } => value.clone(),
    });
    MenuResponse {
        response,
        action,
        event,
        items,
        focus_ids: tree.focus_ids.clone(),
        popup_rects: tree.rects.clone(),
        direction: tree.memory.direction,
    }
}
