use super::{
    DataTable, TableHeader, TableSort, TableSortDirection, cells,
    model::{Layout, Row, next_sort},
    selection::{self, Checked},
};
use crate::{
    Family, Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus_inside,
    tokens::{border, table as geometry},
};
use egui::{Id, Response, Sense, Ui, UiBuilder, WidgetInfo, pos2, vec2};
pub(super) struct Header<C> {
    pub responses: Vec<Response>,
    pub sort: Option<TableSort<C>>,
    pub all: Option<bool>,
}
enum Request<C> {
    Sort(TableSort<C>),
    All(bool),
}
pub(super) fn show<R, C: Clone + PartialEq, K>(
    ui: &mut Ui,
    root: Id,
    config: &DataTable<'_, R, C>,
    layout: &Layout,
    rows: &[Row<K>],
) -> Header<C> {
    let ids: Vec<_> = if layout.selection {
        std::iter::once(root.with("selection-header"))
            .chain(
                config
                    .columns
                    .iter()
                    .map(|col| root.with(("header", col.id))),
            )
            .collect()
    } else {
        config
            .columns
            .iter()
            .map(|col| root.with(("header", col.id)))
            .collect()
    };
    let result = ui.scope_builder(UiBuilder::new().scope_id(root.with("header-row")), |ui| {
        let shape = ui.painter().add(egui::Shape::Noop);
        let result = cells::show(ui, &layout.widths, &ids, true, 0, |ui, index| {
            if layout.selection && index == 0 {
                let count = rows.iter().filter(|r| r.enabled).count();
                let selected = rows.iter().filter(|r| r.enabled && r.selected).count();
                let state = if selected == 0 {
                    Checked::No
                } else if selected == count {
                    Checked::Yes
                } else {
                    Checked::Mixed
                };
                let response = selection_cell(
                    ui,
                    root.with("select-all"),
                    "Select all visible rows",
                    state,
                    count > 0 && !config.loading,
                    config.density.margin(true).top,
                );
                return response
                    .clicked()
                    .then_some(Request::All(state != Checked::Yes));
            }
            let column = &config.columns[index - usize::from(layout.selection)];
            sort_cell(ui, root.with(("sort", column.id)), column, config).map(Request::Sort)
        });
        let rect = ui.min_rect();
        let p = Theme::from_context(ui.ctx()).palette;
        let fill = match config.header {
            TableHeader::Muted => p.muted,
            TableHeader::Inverse => p.foreground,
            TableHeader::Plain => p.background,
        };
        ui.painter()
            .set(shape, egui::Shape::rect_filled(rect, 0, fill));
        result
    });
    cells::row_metadata(&result.response, 0, None);
    let mut sort = None;
    let mut all = None;
    for request in result.inner.1.into_iter().flatten() {
        match request {
            Request::Sort(value) => sort = Some(value),
            Request::All(value) => all = Some(value),
        }
    }
    Header {
        responses: result.inner.0,
        sort,
        all,
    }
}
fn sort_cell<R, C: Clone + PartialEq>(
    ui: &mut Ui,
    id: Id,
    column: &super::TableColumn<'_, C>,
    config: &DataTable<'_, R, C>,
) -> Option<TableSort<C>> {
    let width = ui.available_width();
    let margin = config.density.margin(true);
    let reserve = if column.sortable {
        geometry::SORT_ICON + geometry::SORT_GAP
    } else {
        0.0
    };
    // Live `th`: bold body face, 12 points, uppercase, 0.05 em tracking.
    let job = Text::new(column.label.as_ref())
        .family(Family::Body)
        .size(TextSize::Xs)
        .weight(Weight::Bold)
        .tracking(Tracking::Wider)
        .uppercase(true)
        .layout_job(
            &Theme::from_context(ui.ctx()),
            (width - f32::from(margin.left + margin.right) - reserve).max(0.0),
        );
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let height = galley.size().y + f32::from(margin.top + margin.bottom);
    let (_, rect) = ui.allocate_space(vec2(width, height));
    let current = config.sort.filter(|sort| sort.column == column.key);
    ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
        node.set_label(column.label.to_string());
        if let Some(sort) = current {
            node.set_sort_direction(match sort.direction {
                TableSortDirection::Ascending => egui::accesskit::SortDirection::Ascending,
                TableSortDirection::Descending => egui::accesskit::SortDirection::Descending,
            });
        }
    });
    let response = ui.add_enabled(column.enabled && !config.loading, |ui: &mut Ui| {
        let response = ui.interact(
            rect,
            id,
            if column.sortable {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        if column.sortable {
            response.widget_info(|| {
                WidgetInfo::labeled(
                    egui::accesskit::Role::Button,
                    ui.is_enabled(),
                    format!("Sort by {}", column.label),
                )
            });
        }
        response
    });
    let p = Theme::from_context(ui.ctx()).palette;
    let color = if response.enabled() && response.hovered() && column.sortable {
        p.primary
    } else {
        match config.header {
            TableHeader::Inverse => p.background,
            TableHeader::Muted => p.foreground,
            TableHeader::Plain => p.muted_foreground,
        }
    };
    let content = rect.shrink2(vec2(f32::from(margin.left), f32::from(margin.top)));
    let text_width = galley.size().x + reserve;
    let x = match column.align {
        super::TableAlign::Left => content.left(),
        super::TableAlign::Center => content.center().x - text_width / 2.0,
        super::TableAlign::Right => content.right() - text_width,
    };
    ui.painter().galley_with_override_text_color(
        pos2(x, rect.top() + f32::from(margin.top)),
        galley.clone(),
        color,
    );
    if column.sortable {
        let center = pos2(
            x + galley.size().x + geometry::SORT_GAP + geometry::SORT_ICON / 2.0,
            rect.center().y,
        );
        arrow(
            ui,
            center,
            current.map(|sort| sort.direction),
            if current.is_some() { p.primary } else { color },
        );
        paint_focus_inside(ui, &response, p.primary);
        if response.enabled() && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    response
        .clicked()
        .then(|| next_sort(&column.key, config.sort))
}
fn arrow(ui: &Ui, center: egui::Pos2, direction: Option<TableSortDirection>, color: egui::Color32) {
    // Live 12-point double-arrow glyph: 30 % at rest, full strength when the
    // column is sorted, rotated a half turn for descending order.
    let scale = geometry::SORT_ICON / 24.0;
    let descending = direction == Some(TableSortDirection::Descending);
    let point = |x: f32, y: f32| {
        let offset = vec2(x - 12.0, y - 12.0) * scale;
        if descending {
            center - offset
        } else {
            center + offset
        }
    };
    let color = if direction.is_some() {
        color
    } else {
        color.gamma_multiply(geometry::SORT_IDLE_OPACITY)
    };
    let stroke = egui::Stroke::new(border::THIN, color);
    for points in [
        vec![point(7.0, 16.0), point(7.0, 4.0)],
        vec![point(3.0, 8.0), point(7.0, 4.0), point(11.0, 8.0)],
        vec![point(17.0, 4.0), point(17.0, 16.0)],
        vec![point(13.0, 12.0), point(17.0, 16.0), point(21.0, 12.0)],
    ] {
        ui.painter().add(egui::Shape::line(points, stroke));
    }
}
pub(super) fn selection_cell(
    ui: &mut Ui,
    id: Id,
    label: &str,
    state: Checked,
    enabled: bool,
    padding: i8,
) -> Response {
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(0, padding))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                selection::show(ui, id, label, state, enabled)
            })
            .inner
        })
        .inner
}
