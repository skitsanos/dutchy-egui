use super::{
    DataTable, DataTableResponse, TableSelection, cells,
    focus::Focus,
    header, heights,
    model::{Layout, Row},
    row, state,
};
use crate::{
    EmptyState, EmptyStateVariant, Family, TextSize, Theme, Weight, fonts, tokens::border,
};
use egui::{IdSalt, Response, Ui, UiBuilder};
pub(super) fn show<'a, R, C: Clone + PartialEq, K: egui::AsIdSalt + Clone>(
    config: DataTable<'a, R, C>,
    ui: &mut Ui,
    key: impl Fn(&'a R) -> K,
    mut cell: impl FnMut(&mut Ui, &'a R, &C),
) -> DataTableResponse<C, K> {
    let root = ui.make_persistent_id(("dutchy-data-table", config.id));
    let rows = if config.loading || config.columns.is_empty() {
        Vec::new()
    } else {
        let mut seen = Vec::new();
        config
            .rows
            .iter()
            .map(|record| {
                let key = key(record);
                let id = IdSalt::new(&key);
                Row {
                    id,
                    scope: row::scope(root, id, &mut seen),
                    key,
                    enabled: config
                        .row_enabled
                        .as_ref()
                        .is_none_or(|enabled| enabled(record)),
                    selected: config
                        .selection
                        .as_ref()
                        .is_some_and(|selected| selected(record)),
                }
            })
            .collect::<Vec<_>>()
    };
    let mut focus = Focus::load(ui, root);
    let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
        if !config.enabled {
            ui.disable();
        }
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            fonts::font_id(Family::Body, Weight::Regular, TextSize::Sm.points()),
        );
        let p = Theme::from_context(ui.ctx()).palette;
        egui::Frame::NONE
            .fill(p.background)
            .stroke(if config.bordered {
                egui::Stroke::new(border::STANDARD, p.border)
            } else {
                egui::Stroke::NONE
            })
            .show(ui, |ui| {
                let layout = Layout::new(
                    config.columns,
                    ui.available_width(),
                    config.selection.is_some(),
                );
                if config.columns.is_empty() {
                    EmptyState::new("no-columns", "No columns configured")
                        .description("Define at least one column to display records.")
                        .variant(EmptyStateVariant::Table)
                        .show(ui);
                    return (None, Vec::new(), Vec::new(), Vec::new());
                }
                egui::ScrollArea::horizontal()
                    .id_salt("table-horizontal")
                    .auto_shrink([true, true])
                    .show_viewport(ui, |ui, viewport| {
                        ui.set_width(layout.width);
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                        let table =
                            ui.scope_builder(UiBuilder::new().scope_id(root.with("table")), |ui| {
                                let headers = header::show(ui, root, &config, &layout, &rows);
                                cells::separator(ui, layout.width, border::STANDARD);
                                let body = egui::ScrollArea::vertical()
                                    .id_salt("table-body")
                                    .max_height(config.max_height)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        ui.set_width(layout.width);
                                        if config.loading || rows.is_empty() {
                                            state::show(ui, root, &config, &layout, viewport);
                                            return Vec::new();
                                        }
                                        let view = row::View {
                                            config: &config,
                                            layout: &layout,
                                        };
                                        let mut rendered = Vec::with_capacity(rows.len());
                                        for (index, (record, state)) in
                                            config.rows.iter().zip(&rows).enumerate()
                                        {
                                            if index > 0 {
                                                cells::separator(ui, layout.width, border::THIN);
                                            }
                                            rendered.push(row::show(
                                                ui, &view, record, state, index, &mut focus,
                                                &mut cell,
                                            ));
                                        }
                                        let responses: Vec<_> = rendered
                                            .iter()
                                            .map(|row| row.response.clone())
                                            .collect();
                                        row::reveal_focus(ui, &responses, &headers.responses);
                                        rendered
                                    });
                                // Reveal a focused header only after the body scroll area has consumed its own target.
                                row::reveal_focus(ui, &headers.responses, &[]);
                                (headers, body.inner)
                            });
                        ui.ctx().accesskit_node_builder(table.response.id, |node| {
                            node.set_role(egui::accesskit::Role::Table);
                            node.set_label(config.name.to_string());
                            node.set_column_count(layout.widths.len());
                            node.set_row_count(if rows.is_empty() { 2 } else { rows.len() + 1 });
                            if layout.selection {
                                node.set_multiselectable();
                            }
                            if config.loading {
                                node.set_busy();
                            }
                        });
                        let (headers, rendered) = table.inner;
                        let mut selections = Vec::new();
                        if let Some(selected) = headers.all {
                            selections.extend(
                                rows.iter()
                                    .filter(|row| row.enabled && row.selected != selected)
                                    .map(|row| TableSelection {
                                        key: row.key.clone(),
                                        selected,
                                    }),
                            );
                        }
                        for (response, state) in rendered.iter().zip(&rows) {
                            if response.selection.as_ref().is_some_and(Response::clicked) {
                                selections.push(TableSelection {
                                    key: state.key.clone(),
                                    selected: !state.selected,
                                });
                            }
                        }
                        (headers.sort, selections, headers.responses, rendered)
                    })
                    .inner
            })
            .inner
    });
    let scopes: Vec<_> = rows.iter().map(|row| row.scope).collect();
    heights::evict_stale(ui.ctx(), root, &scopes);
    result.response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::accesskit::Role::Group,
            ui.is_enabled() && config.enabled,
            format!("{} viewport", config.name),
        )
    });
    let row_ids: Vec<_> = rows.iter().map(|row| row.id).collect();
    let mut column_ids: Vec<_> = config
        .columns
        .iter()
        .map(|column| IdSalt::new(("column", column.id)))
        .collect();
    if config.selection.is_some() {
        column_ids.push(IdSalt::new("selection"));
    }
    Focus::finish(focus, ui, root, &row_ids, &column_ids, config.return_focus);
    DataTableResponse {
        response: result.response,
        sort_requested: result.inner.0,
        selection_requested: result.inner.1,
        headers: result.inner.2,
        rows: result.inner.3,
    }
}
