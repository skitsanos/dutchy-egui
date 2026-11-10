use super::{
    List, ListItem, ListResponse, ListRow, ListRowResponse, accessibility, hierarchy, keys,
    model::Model, paint, selection,
};
use crate::{Theme, tokens::border};
use egui::{Id, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, vec2};
use std::hash::Hash;

pub(super) fn show<'a, K: Clone + Eq + Hash>(
    config: List<'a, K>,
    ui: &mut Ui,
    mut content: impl FnMut(&mut Ui, &ListItem<'a, K>, ListRow),
) -> ListResponse<K> {
    let root = hierarchy::root(ui, config.id);
    let model = Model::new(root, config.items);
    let normalized = if config.hierarchy.is_some() {
        model.normalize_focus(config.state);
        false
    } else {
        model.normalize(config.state, config.mode)
    };
    let empty = model.enabled.is_empty();
    let sense = if empty {
        Sense::focusable_noninteractive()
    } else {
        Sense::hover()
    };
    let result = ui.scope_builder(UiBuilder::new().scope_id(root).sense(sense), |ui| {
        if !config.enabled {
            ui.disable();
        }
        ui.spacing_mut().item_spacing.y = 0.0;
        let group_id = ui.scope_id();
        let accessibility_parent = ui.response().id;
        // Make the collection the accessible ancestor before adding its options.
        accessibility::group(
            &ui.response(),
            &config.name,
            model.items.len(),
            config.mode,
            config.read_only,
        );
        let (offset, height) = ui
            .ctx()
            .data(|d| d.get_temp::<(f32, f32)>(root.with("metrics")))
            .unwrap_or((0.0, config.height));
        let last = ui.ctx().data(|d| d.get_temp::<Id>(root.with("cursor")));
        let focused = ui.memory(|m| m.focused());
        let current = model
            .index(config.state.focused.as_ref())
            .map(|i| model.ids[i]);
        let owned =
            focused.is_some_and(|id| Some(id) == last || model.ids.contains(&id) || id == group_id);
        if owned && ui.is_enabled() && focused != current {
            ui.memory_mut(|m| m.request_focus(current.unwrap_or(group_id)));
        }
        let mut outcome = keys::read(
            ui,
            &model,
            config.state,
            config.mode,
            config.read_only,
            (height / config.row_height).floor().max(1.0) as usize,
        );
        let active = model.index(config.state.focused.as_ref());
        let current = active.map(|i| model.ids[i]);
        let reveal = outcome.reveal || (current != last && current.is_some());
        let mut area = egui::ScrollArea::vertical()
            .id_salt(root.with("scroll"))
            .auto_shrink([false, true])
            .max_height(config.height);
        if !ui.is_enabled() {
            area = area.scroll_source(egui::scroll_area::ScrollSource::NONE);
        }
        if reveal && let Some(index) = active {
            area = area.vertical_scroll_offset(reveal_offset(
                offset,
                height,
                index,
                config.row_height,
            ));
        }
        let style = paint::Style {
            mode: config.mode,
            read_only: config.read_only,
        };
        let scroll = area.show_rows(ui, config.row_height, model.items.len(), |ui, range| {
            let origin = ui.max_rect().top() - range.start as f32 * config.row_height;
            let mut rows = Vec::with_capacity(range.len());
            let mut prepared = config
                .hierarchy
                .map(|metas| {
                    hierarchy::prepare(
                        ui,
                        &model,
                        hierarchy::View {
                            metas,
                            range: range.clone(),
                            active,
                            origin,
                            height: config.row_height,
                            parent: accessibility_parent,
                        },
                    )
                })
                .unwrap_or_default();
            if model.items.is_empty() {
                let (rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), config.row_height),
                    Sense::hover(),
                );
                let inset = crate::tokens::list::PADDING.min(rect.width() / 2.0);
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(rect.shrink2(vec2(inset, 0.0)))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                child.add(crate::Text::new(config.empty.as_ref()).color(crate::TextColor::Muted));
            }
            for index in range.clone() {
                let item = &model.items[index];
                let (_, rect) = ui.allocate_space(vec2(ui.available_width(), config.row_height));
                let response = prepared.remove(&index).unwrap_or_else(|| {
                    paint::interact(
                        ui,
                        model.ids[index],
                        rect,
                        active == Some(index),
                        model.is_enabled(index),
                    )
                });
                if response.clicked() && model.is_enabled(index) {
                    if !config.read_only
                        && !hierarchy::clicked(&response, config.hierarchy.map(|h| h[index]))
                    {
                        outcome.changed |= selection::select(
                            config.state,
                            &model,
                            config.mode,
                            index,
                            selection::pointer_modifiers(ui, response.rect),
                            true,
                        );
                        if response.double_clicked() {
                            outcome.activated = Some(item.key.clone());
                        }
                    }
                    config.state.focused = Some(item.key.clone());
                    response.request_focus();
                    ui.ctx().request_repaint();
                }
                rows.push(ListRowResponse {
                    key: item.key.clone(),
                    index,
                    response,
                });
            }
            // Resolve all pointer edits before painting or publishing option states.
            for result in &rows {
                let index = result.index;
                let item = &model.items[index];
                let response = &result.response;
                let rect = response.rect;
                let row = ListRow {
                    index,
                    selected: config.state.selected.contains(&item.key) && model.is_enabled(index),
                    focused: response.has_focus(),
                };
                paint::metadata(
                    ui,
                    response,
                    &item.label,
                    row,
                    &style,
                    config.hierarchy.map(|h| h[index]),
                );
                if ui.is_rect_visible(rect) {
                    let mut child = ui.new_child(
                        UiBuilder::new()
                            .id_salt(("content", model.ids[index]))
                            .max_rect(rect),
                    );
                    if !model.is_enabled(index) {
                        child.disable();
                    }
                    paint::content(
                        &mut child,
                        response,
                        row,
                        config.hierarchy.map(|h| h[index]),
                        |ui| content(ui, item, row),
                    );
                }
            }
            // Keep a manually scrolled-away row's identity and focus alive without
            // calling its content renderer or pulling the viewport back to it.
            if let Some(index) = active.filter(|i| !range.contains(i)) {
                let rect = Rect::from_min_size(
                    egui::pos2(
                        ui.max_rect().left(),
                        origin + index as f32 * config.row_height,
                    ),
                    vec2(ui.available_width(), config.row_height),
                );
                let response = prepared
                    .remove(&index)
                    .unwrap_or_else(|| paint::interact(ui, model.ids[index], rect, true, true));
                let row = ListRow {
                    index,
                    selected: config.state.selected.contains(&model.items[index].key),
                    focused: response.has_focus(),
                };
                paint::metadata(
                    ui,
                    &response,
                    &model.items[index].label,
                    row,
                    &style,
                    config.hierarchy.map(|h| h[index]),
                );
            }
            if let Some(metas) = config.hierarchy {
                for (index, response) in prepared {
                    let row = ListRow {
                        index,
                        selected: config.state.selected.contains(&model.items[index].key)
                            && model.is_enabled(index),
                        focused: response.has_focus(),
                    };
                    paint::metadata(
                        ui,
                        &response,
                        &model.items[index].label,
                        row,
                        &style,
                        Some(metas[index]),
                    );
                }
                let hits: Vec<_> = rows
                    .iter()
                    .filter(|r| metas[r.index].expanded.is_some() && r.response.enabled())
                    .map(|r| {
                        (
                            r.response.id,
                            hierarchy::disclosure(r.response.rect, metas[r.index].depth)
                                .intersect(ui.clip_rect()),
                        )
                    })
                    .filter(|(_, rect)| rect.is_positive())
                    .collect();
                ui.ctx()
                    .data_mut(|d| d.insert_temp(root.with("tree-disclosures"), hits));
            }
            (rows, range)
        });
        let current = model
            .index(config.state.focused.as_ref())
            .map(|i| model.ids[i]);
        ui.ctx().data_mut(|d| {
            d.insert_temp(
                root.with("metrics"),
                (scroll.state.clamped_offset().y, scroll.inner_rect.height()),
            );
            if let Some(id) = current {
                d.insert_temp(root.with("cursor"), id);
            } else {
                d.remove::<Id>(root.with("cursor"));
            }
        });
        (scroll.inner, scroll.inner_rect, outcome)
    });
    let ((rows, visible_range), viewport, outcome) = result.inner;
    let mut response = result.response;
    accessibility::group(
        &response,
        &config.name,
        model.items.len(),
        config.mode,
        config.read_only,
    );
    if normalized || outcome.changed {
        response.mark_changed();
    }
    if ui.is_rect_visible(response.rect) {
        ui.painter().rect_stroke(
            response.rect,
            0,
            Stroke::new(
                border::STANDARD,
                Theme::from_context(ui.ctx()).palette.border,
            ),
            StrokeKind::Inside,
        );
    }
    ListResponse {
        response,
        rows,
        visible_range,
        viewport,
        activated: outcome.activated,
    }
}

fn reveal_offset(offset: f32, height: f32, index: usize, row_height: f32) -> f32 {
    let top = index as f32 * row_height;
    if top < offset || row_height >= height {
        top
    } else if top + row_height > offset + height {
        top + row_height - height
    } else {
        offset
    }
}
