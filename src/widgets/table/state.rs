use super::{DataTable, cells, model::Layout};
use crate::{EmptyState, EmptyStateVariant, Skeleton, SkeletonGroup, tokens::table as geometry};
use egui::{Id, Rect, Ui, UiBuilder, pos2, vec2};

pub(super) fn show<R, C>(
    ui: &mut Ui,
    root: Id,
    config: &DataTable<'_, R, C>,
    layout: &Layout,
    viewport: Rect,
) {
    let origin = ui.cursor().min;
    // Keep guidance readable even when a previously focused column scrolled
    // horizontally. The semantic cell still spans all table columns.
    let rect = Rect::from_min_size(
        pos2(origin.x + viewport.left(), origin.y),
        vec2(viewport.width().min(layout.width), f32::INFINITY),
    );
    let row = ui.scope_builder(UiBuilder::new().scope_id(root.with("state-row")), |ui| {
        let cell = ui.scope_builder(
            UiBuilder::new()
                .scope_id(root.with("state-cell"))
                .max_rect(rect),
            |ui| {
                ui.set_width(rect.width());
                if config.loading {
                    loading(ui, config);
                } else {
                    EmptyState::new("empty", config.empty_title.as_ref())
                        .description(config.empty_description.as_ref())
                        .variant(EmptyStateVariant::Table)
                        .show(ui);
                }
            },
        );
        ui.ctx().accesskit_node_builder(cell.response.id, |node| {
            node.set_role(egui::accesskit::Role::Cell);
            node.set_row_index(1);
            node.set_column_index(0);
            node.set_column_span(layout.widths.len());
        });
        cells::bounds(
            &cell.response,
            Rect::from_min_size(origin, vec2(layout.width, cell.response.rect.height())),
        );
    });
    cells::row_metadata(&row.response, 1, None);
}

fn loading<R, C>(ui: &mut Ui, config: &DataTable<'_, R, C>) {
    let group = SkeletonGroup::new("loading", format!("Loading {}", config.name)).show(ui, |ui| {
        egui::Frame::NONE
            .inner_margin(config.density.margin(false))
            .show(ui, |ui| {
                for index in 0..geometry::LOADING_ROWS {
                    if index > 0 {
                        ui.add_space(geometry::LOADING_GAP);
                    }
                    ui.add(Skeleton::new().width(ui.available_width()));
                }
            });
    });
    cells::bounds(&group.response, group.response.rect);
}
