//! Vertical centering must include native hitboxes and semantic child bounds.
mod common;

use dutchy_egui::{
    Badge, Button, ControlSize, DataTable, Family, TableAlign, TableColumn, TableColumnWidth,
    TableDensity, Text, TextSize, Theme,
};
use egui::{Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const HEADER: &str = "A heading that wraps across several lines in a narrow column";
const NOTE: &str = "A longer note wraps within its column and sets the row height. The other cells must share its vertical center, including selection and action controls.";

struct Alignment {
    compact: bool,
    tall: bool,
    narrow: bool,
    selection: bool,
    selected: bool,
    clicks: usize,
    sorts: usize,
    row: Option<Rect>,
    header: Option<Rect>,
}

impl Default for Alignment {
    fn default() -> Self {
        Self {
            compact: false,
            tall: true,
            narrow: false,
            selection: true,
            selected: false,
            clicks: 0,
            sorts: 0,
            row: None,
            header: None,
        }
    }
}

fn harness(scale: f32) -> Harness<'static, Alignment> {
    Harness::builder()
        .with_size([1000.0, 1000.0])
        .with_pixels_per_point(scale)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(show, Alignment::default())
}

fn show(ui: &mut egui::Ui, state: &mut Alignment) {
    let columns = [
        TableColumn::new(0, "Name")
            .sortable(true)
            .width(TableColumnWidth::Fixed(160.0)),
        TableColumn::new(1, HEADER)
            .sortable(true)
            .width(TableColumnWidth::Fixed(if state.narrow {
                120.0
            } else {
                240.0
            })),
        TableColumn::new(2, "Status").width(TableColumnWidth::Fixed(120.0)),
        TableColumn::new(3, "Amount")
            .sortable(true)
            .align(TableAlign::Right)
            .width(TableColumnWidth::Fixed(140.0)),
        TableColumn::new(4, "Actions").width(TableColumnWidth::Fixed(120.0)),
    ];
    let rows = [1];
    let note = if state.tall { NOTE } else { "Short note" };
    let mut table = DataTable::new("alignment", &columns, &rows)
        .max_height(700.0)
        .density(if state.compact {
            TableDensity::Compact
        } else {
            TableDensity::Comfortable
        });
    if state.selection {
        table = table.selection(|_| state.selected);
    }
    let mut clicked = false;
    let result = table.show(
        ui,
        |row| *row,
        |ui, _, column| match column {
            0 => {
                ui.add(Text::new("Name 1").size(TextSize::Sm));
            }
            1 => {
                ui.add(Text::new(note).size(TextSize::Sm));
            }
            2 => {
                ui.add(Badge::new("Active"));
            }
            3 => {
                ui.add(Text::new("$123.45").size(TextSize::Sm).family(Family::Mono));
            }
            _ => {
                clicked = ui
                    .add(Button::new("View").size(ControlSize::Small))
                    .clicked();
            }
        },
    );
    state.clicks += usize::from(clicked);
    state.sorts += usize::from(result.sort_requested.is_some());
    if let Some(request) = result.selection_requested.first() {
        state.selected = request.selected;
    }
    state.row = Some(result.rows[0].response.rect);
    state.header = Some(result.headers[0].rect);
}

fn centered(actual: Rect, row: Rect, scale: f32) {
    assert!(
        (actual.center().y - row.center().y).abs() <= 0.6 / scale,
        "child {actual:?} must be centered within {row:?}"
    );
    assert!(actual.top() >= row.top() && actual.bottom() <= row.bottom());
}

fn assert_alignment(h: &Harness<'_, Alignment>, scale: f32) {
    let state = h.state();
    let row = state.row.unwrap();
    let note = if state.tall { NOTE } else { "Short note" };
    for label in ["Name 1", note, "ACTIVE", "$123.45", "VIEW"] {
        centered(h.get_by_label(label).rect(), row, scale);
    }
    for label in [
        "Sort by Name",
        "Sort by Amount",
        &format!("Sort by {HEADER}"),
    ] {
        centered(h.get_by_label(label).rect(), state.header.unwrap(), scale);
    }
    if state.selection {
        centered(h.get_by_label("Select row 1").rect(), row, scale);
        centered(
            h.get_by_label("Select all visible rows").rect(),
            state.header.unwrap(),
            scale,
        );
    }
}

#[test]
fn mixed_cell_content_and_wrapped_headers_are_centered_at_each_density_and_scale() {
    for scale in [1.0, 1.5, 2.0] {
        for compact in [false, true] {
            let mut h = harness(scale);
            h.state_mut().compact = compact;
            h.run_steps(5);
            assert_alignment(&h, scale);
            h.check_accessibility();
        }
    }
}

#[test]
fn centering_reflows_and_shrinks_without_geometry_drift_or_losing_focus() {
    let mut h = harness(1.5);
    h.run_steps(4);
    h.get_by_label("VIEW").focus();
    h.run_steps(2);
    let id = h.get_by_label("VIEW").accesskit_node().id();
    let initial = h.state().row.unwrap();
    h.state_mut().narrow = true;
    h.run_steps(5);
    assert!(h.state().row.unwrap().height() > initial.height());
    assert_alignment(&h, 1.5);
    h.state_mut().tall = false;
    h.state_mut().compact = true;
    h.state_mut().selection = false;
    h.run_steps(5);
    assert_alignment(&h, 1.5);
    assert!(h.state().row.unwrap().height() < initial.height());
    let settled = h.state().row.unwrap();
    let control = h.get_by_label("VIEW").rect();
    h.run_steps(12);
    assert_eq!(h.state().row.unwrap(), settled);
    assert_eq!(h.get_by_label("VIEW").rect(), control);
    assert_eq!(h.get_by_label("VIEW").accesskit_node().id(), id);
    assert!(h.get_by_label("VIEW").is_focused());
}

#[test]
fn centered_pointer_targets_and_keyboard_controls_keep_independent_actions() {
    let mut h = harness(1.0);
    h.run_steps(4);
    assert_alignment(&h, 1.0);
    h.get_by_label("VIEW").click();
    h.run_steps(3);
    assert_eq!(h.state().clicks, 1);
    assert!(!h.state().selected);
    h.get_by_label("Select row 1").click();
    h.run_steps(3);
    assert!(h.state().selected);
    h.get_by_label("Sort by Name").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Space);
    h.run_steps(3);
    assert_eq!(h.state().sorts, 1);
    assert_eq!(h.state().clicks, 1);
    let header = h.get_by_role_and_label(Role::ColumnHeader, "Name").rect();
    centered(h.get_by_label("Sort by Name").rect(), header, 1.0);
}
