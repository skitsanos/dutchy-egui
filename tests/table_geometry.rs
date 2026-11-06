//! Fixed column alignment, sticky headers, focus reveal and colspan state rows.
mod common;
use dutchy_egui::{DataTable, TableColumn, TableColumnWidth, Theme};
use egui::{Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
#[derive(Default)]
struct Geometry {
    headers: Vec<Rect>,
    cells: Vec<Vec<Rect>>,
    table: Option<Rect>,
    empty: bool,
    loading: bool,
}
fn harness(width: f32, wide: bool, oversized: bool) -> Harness<'static, Geometry> {
    Harness::builder()
        .with_size([width, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            move |ui, state: &mut Geometry| {
                let columns = [
                    TableColumn::new(0, "Long heading that wraps")
                        .sortable(true)
                        .width(if wide {
                            TableColumnWidth::Fixed(240.0)
                        } else {
                            TableColumnWidth::Flexible(110.0)
                        }),
                    TableColumn::new(1, "Value").width(TableColumnWidth::Fixed(140.0)),
                    TableColumn::new(2, "Actions")
                        .sortable(true)
                        .width(TableColumnWidth::Fixed(180.0)),
                ];
                let rows: Vec<u8> = if state.empty {
                    Vec::new()
                } else {
                    (0..10).collect()
                };
                let result = DataTable::new("geometry", &columns, &rows)
                    .accessible_name("Geometry")
                    .max_height(180.0)
                    .loading(state.loading)
                    .show(
                        ui,
                        |row| *row,
                        |ui, row, column| cell(ui, row, *column, oversized),
                    );
                state.headers = result.headers.iter().map(|r| r.rect).collect();
                state.cells = result
                    .rows
                    .iter()
                    .map(|row| row.cells.iter().map(|r| r.rect).collect())
                    .collect();
                state.table = Some(result.response.rect);
            },
            Geometry::default(),
        )
}
fn cell(ui: &mut egui::Ui, row: &u8, column: u8, oversized: bool) {
    match column {
        0 => {
            ui.label(if *row == 0 {
                "Long content wraps inside the column and increases only the row height."
            } else {
                "Content"
            });
            if oversized {
                ui.allocate_space(egui::vec2(900.0, 20.0));
            }
        }
        1 => {
            ui.label(row.to_string());
        }
        _ => {
            let _ = ui.button(format!("Action {row}"));
        }
    }
}

#[test]
fn fractional_widths_and_oversized_content_do_not_shift_later_columns() {
    for width in [600.1, 813.7] {
        let mut h = harness(width, false, true);
        h.run_steps(4);
        for cells in &h.state().cells {
            for (cell, header) in cells.iter().zip(&h.state().headers) {
                assert!((cell.left() - header.left()).abs() < 0.5);
                assert!((cell.right() - header.right()).abs() < 0.5);
            }
            assert!(cells.iter().all(|cell| cell.height() == cells[0].height()));
        }
    }
}
#[test]
fn keyboard_focus_reveals_horizontal_overflow_with_headers_and_cells_aligned() {
    let mut h = harness(300.0, true, false);
    h.run_steps(4);
    let before = h.get_by_label("Sort by Actions").rect();
    assert!(before.right() > 300.0);
    h.get_by_label("Sort by Actions").focus();
    h.run_steps(6);
    let after = h.get_by_label("Sort by Actions").rect();
    assert!(after.right() <= 292.5, "{after:?}");
    assert!(after.left() < before.left());
    assert!((h.state().cells[0][2].left() - h.state().headers[2].left()).abs() < 0.5);
}
#[test]
fn body_focus_scrolls_without_moving_the_sticky_header_or_resetting_on_header_focus() {
    let mut h = harness(840.0, false, false);
    h.run_steps(4);
    let header = h.state().headers[0];
    h.get_by_label("Action 9").focus();
    h.run_steps(6);
    let action = h.get_by_label("Action 9").rect();
    assert!(
        action.bottom() <= h.state().table.unwrap().bottom(),
        "{action:?}"
    );
    assert_eq!(h.state().headers[0], header);
    let last = h.state().cells[9][0];
    h.get_by_label("Sort by Long heading that wraps").focus();
    h.run_steps(4);
    assert_eq!(h.state().cells[9][0], last);
    h.check_accessibility();
}
#[test]
fn empty_state_spans_all_data_and_selection_columns() {
    let mut h = Harness::builder()
        .with_size([700.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let columns = [TableColumn::new(0, "Name"), TableColumn::new(1, "Email")];
            let rows: [u8; 0] = [];
            DataTable::new("empty", &columns, &rows)
                .accessible_name("Empty records")
                .selection(|_| false)
                .show(ui, |row| *row, |_, _, _| unreachable!());
        });
    h.run_steps(3);
    let row = h
        .get_by_role_and_label(Role::Table, "Empty records")
        .get_by_role_and_label(Role::Row, "Row 1");
    let cell = row.get_by_role(Role::Cell);
    assert_eq!(cell.accesskit_node().data().column_span(), Some(3));
    h.check_accessibility();
}

#[test]
fn replacement_guidance_stays_in_the_viewport_after_horizontal_scrolling() {
    let mut h = harness(280.0, true, false);
    h.run_steps(3);
    h.get_by_label("Sort by Actions").focus();
    h.run_steps(5);
    assert!(h.state().headers[0].left() < 0.0);
    h.state_mut().empty = true;
    h.run_steps(4);
    let text = h
        .get_by_label("Try adjusting your search or filters.")
        .rect();
    assert!(text.left() >= 8.0 && text.right() <= 272.5, "{text:?}");
    assert!(h.state().headers[0].left() < 0.0);
    h.state_mut().loading = true;
    h.run_steps(4);
    let skeleton = h.get_by_label("Loading Geometry").rect();
    assert!(
        skeleton.left() >= 8.0 && skeleton.right() <= 272.5,
        "{skeleton:?}"
    );
    h.check_accessibility();
}
