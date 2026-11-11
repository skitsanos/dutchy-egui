use super::{Cursor, SearchKeys};
use crate::{
    MenuItem, Text, TextColor, TextSize, Tracking, Weight, tokens::menu as geometry,
    widgets::menu::paint,
};
use egui::{Id, IdSalt, Response, Ui, UiBuilder, WidgetInfo};

pub(crate) struct Results<'a, T> {
    pub items: &'a [MenuItem<'a, T>],
    pub query: &'a str,
    pub name: &'a str,
    pub selected: Option<IdSalt>,
    pub height: f32,
}
pub(crate) struct ResultsResponse<T> {
    pub response: Response,
    pub action: Option<T>,
    pub label: Option<String>,
    pub active: Option<Id>,
}
impl<T: Clone> Results<'_, T> {
    pub fn show(
        self,
        ui: &mut Ui,
        root: Id,
        cursor: &mut Cursor,
        keys: &SearchKeys,
    ) -> ResultsResponse<T> {
        let ranked = !self.query.trim().is_empty();
        let matches = super::rank::rank(self.items, self.query);
        let enabled: Vec<_> = matches
            .iter()
            .filter(|hit| hit.item.enabled)
            .map(|hit| hit.item.id)
            .collect();
        let old_active = cursor.active;
        cursor.update(self.query, &enabled, keys);
        let mut action = None;
        let mut label = None;
        let list = ui.scope_builder(UiBuilder::new().scope_id(root.with("list")), |ui| {
            egui::ScrollArea::vertical()
                .max_height(self.height.max(0.0))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if matches.is_empty() {
                        egui::Frame::NONE
                            .inner_margin(egui::Margin::same(geometry::SEARCH_PADDING))
                            .show(ui, |ui| {
                                let response = ui.add(
                                    Text::new("No matches")
                                        .size(TextSize::Sm)
                                        .color(TextColor::Muted),
                                );
                                response.ctx.accesskit_node_builder(response.id, |node| {
                                    node.set_role(egui::accesskit::Role::Status);
                                    node.set_label("No matches");
                                    node.set_live(egui::accesskit::Live::Polite);
                                });
                            });
                    }
                    let mut previous_group = None;
                    for (index, hit) in matches.iter().enumerate() {
                        let item = hit.item;
                        if item.group.as_deref() != previous_group {
                            if index > 0 {
                                paint::divider(ui, true);
                            }
                            if let Some(group) = &item.group {
                                egui::Frame::NONE
                                    .inner_margin(egui::Margin::symmetric(
                                        geometry::PADDING_X as i8,
                                        geometry::GROUP_PADDING,
                                    ))
                                    .show(ui, |ui| {
                                        ui.add(
                                            Text::new(group.as_ref())
                                                .size(TextSize::Xs)
                                                .weight(Weight::Bold)
                                                .tracking(Tracking::Wider)
                                                .uppercase(true)
                                                .color(TextColor::Muted),
                                        );
                                    });
                            }
                        } else if index > 0 {
                            // Host separators describe the host order, not a ranking.
                            paint::divider(ui, item.separator_before && !ranked);
                        }
                        previous_group = item.group.as_deref();
                        let active = cursor.active == Some(item.id);
                        let response = ui
                            .add_enabled_ui(item.enabled, |ui| {
                                paint::row_marked(
                                    ui,
                                    root.with(item.id),
                                    item,
                                    &hit.marks,
                                    paint::RowState {
                                        role: egui::accesskit::Role::ListBoxOption,
                                        active,
                                        selected: self.selected == Some(item.id),
                                        focusable: false,
                                        check_slot: item.checked.is_some(),
                                        pointer: true,
                                        branch_open: false,
                                    },
                                )
                            })
                            .inner;
                        if response.hovered()
                            && item.enabled
                            && cursor.active != Some(item.id)
                            && ui.input(|input| input.pointer.delta() != egui::Vec2::ZERO)
                        {
                            cursor.active = Some(item.id);
                            ui.ctx().request_repaint();
                        }
                        if active && (old_active != cursor.active || keys.opens()) {
                            response
                                .scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
                        }
                        if response.clicked() || (keys.activate && active && item.enabled) {
                            action = Some(item.value.clone());
                            label = Some(item.label.to_string());
                        }
                    }
                });
        });
        list.response.widget_info(|| {
            WidgetInfo::labeled(egui::accesskit::Role::ListBox, ui.is_enabled(), self.name)
        });
        ResultsResponse {
            response: list.response,
            action,
            label,
            active: cursor.active.map(|salt| root.with(salt)),
        }
    }
}
