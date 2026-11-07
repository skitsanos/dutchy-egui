use super::{
    DataTable, TableRowResponse, cells,
    focus::Focus,
    header,
    model::{Layout, Row},
    selection::Checked,
};
use crate::{
    Theme,
    tokens::{border, table as geometry},
};
use egui::{Id, Response, Ui, UiBuilder};
pub(super) struct View<'v, 'a, R, C> {
    pub config: &'v DataTable<'a, R, C>,
    pub layout: &'v Layout,
}
/// Row scopes derive from host keys; a repeated key gets an occurrence salt so
/// two rows never share identity, measurements, or widget IDs.
pub(super) fn scope(root: Id, id: egui::IdSalt, seen: &mut Vec<(Id, usize)>) -> Id {
    let base = root.with(("row", id));
    match seen.iter_mut().find(|(seen, _)| *seen == base) {
        Some((_, count)) => {
            *count += 1;
            base.with(("duplicate", *count))
        }
        None => {
            seen.push((base, 0));
            base
        }
    }
}
pub(super) fn show<'a, R, C: Clone + PartialEq, K: Clone>(
    ui: &mut Ui,
    view: &View<'_, 'a, R, C>,
    record: &'a R,
    state: &Row<K>,
    index: usize,
    focus: &mut Focus,
    cell: &mut impl FnMut(&mut Ui, &'a R, &C),
) -> TableRowResponse<K> {
    let config = view.config;
    let layout = view.layout;
    let row_id = state.scope;
    let ids: Vec<_> = if layout.selection {
        std::iter::once(row_id.with("selection"))
            .chain(
                config
                    .columns
                    .iter()
                    .map(|col| row_id.with(("column", col.id))),
            )
            .collect()
    } else {
        config
            .columns
            .iter()
            .map(|col| row_id.with(("column", col.id)))
            .collect()
    };
    let result = ui.scope_builder(UiBuilder::new().scope_id(row_id), |ui| {
        if !state.enabled {
            ui.disable();
        }
        let background = ui.painter().add(egui::Shape::Noop);
        let result = cells::show(ui, &layout.widths, &ids, false, index + 1, |ui, column| {
            if layout.selection && column == 0 {
                let response = header::selection_cell(
                    ui,
                    row_id.with("toggle"),
                    &format!("Select row {}", index + 1),
                    if state.selected {
                        Checked::Yes
                    } else {
                        Checked::No
                    },
                    true,
                    config.density.margin(false).top,
                );
                focus.capture(ui, state.id, egui::IdSalt::new("selection"));
                return Some(response);
            }
            let column = &config.columns[column - usize::from(layout.selection)];
            egui::Frame::NONE
                .inner_margin(config.density.margin(false))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(column.align.align()), |ui| {
                        cell(ui, record, &column.key)
                    });
                });
            Focus::capture(
                focus,
                ui,
                state.id,
                egui::IdSalt::new(("column", column.id)),
            );
            None
        });
        paint(
            ui,
            background,
            ui.min_rect(),
            config.striped && index % 2 == 1,
            state.selected,
        );
        result
    });
    cells::row_metadata(
        &result.response,
        index + 1,
        layout.selection.then_some(state.selected),
    );
    TableRowResponse {
        key: state.key.clone(),
        response: result.response,
        cells: result.inner.0,
        selection: result.inner.1.into_iter().flatten().next(),
    }
}
fn paint(ui: &Ui, shape: egui::layers::ShapeIdx, rect: egui::Rect, striped: bool, selected: bool) {
    let p = Theme::from_context(ui.ctx()).palette;
    let hover = ui.rect_contains_pointer(rect);
    let fill = if selected {
        p.background
            .lerp_to_gamma(p.primary, geometry::SELECTED_OPACITY)
    } else if hover {
        p.background.lerp_to_gamma(p.muted, geometry::HOVER_OPACITY)
    } else if striped {
        p.background
            .lerp_to_gamma(p.muted, geometry::STRIPE_OPACITY)
    } else {
        p.background
    };
    ui.painter()
        .set(shape, egui::Shape::rect_filled(rect, 0, fill));
    if selected {
        let mut edge = rect;
        edge.max.x = edge.min.x + border::ACCENT;
        ui.painter().rect_filled(edge, 0, p.primary);
    }
}
/// Reveal a focused control inside `owners` once it gains focus or layout
/// moves it; user scrolling never pulls it back. Controls inside `excluded`
/// cells (the sticky header) are skipped even when scrolled rows overlap them.
pub(super) fn reveal_focus(ui: &Ui, owners: &[Response], excluded: &[Response]) {
    if let Some(response) = ui
        .memory(|m| m.focused())
        .and_then(|id| ui.ctx().read_response(id))
        && response.layer_id == ui.layer_id()
        && !excluded
            .iter()
            .any(|cell| cell.rect.contains_rect(response.rect))
        && owners
            .iter()
            .any(|owner| owner.rect.contains_rect(response.rect))
    {
        crate::interaction::reveal_focused(ui, &response, ui.max_rect().min);
    }
}
