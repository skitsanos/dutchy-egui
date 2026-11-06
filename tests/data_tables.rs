//! Table semantics, host requests, keyed identity and controlled state.
mod common;
use dutchy_egui::{
    DataTable, TableColumn, TableColumnWidth, TableSelection, TableSort, TableSortDirection, Theme,
};
use egui::{
    Key,
    accesskit::{Role, SortDirection, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Column {
    Name,
    Value,
    Edit,
    Extra,
}
struct Record {
    id: u8,
    name: String,
    value: i32,
}
struct State {
    records: Vec<Record>,
    sort: Option<TableSort<Column>>,
    selected: BTreeSet<u8>,
    edits: BTreeMap<u8, String>,
    sorts: Vec<TableSort<Column>>,
    selections: Vec<TableSelection<u8>>,
    calls: usize,
    inspections: usize,
    loading: bool,
    disabled: bool,
    no_columns: bool,
    extra: bool,
    renamed: bool,
    disable_value: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            records: (1..=5)
                .map(|id| Record {
                    id,
                    name: format!("Record {id}"),
                    value: i32::from(id) * 10,
                })
                .collect(),
            sort: None,
            selected: BTreeSet::new(),
            edits: BTreeMap::new(),
            sorts: Vec::new(),
            selections: Vec::new(),
            calls: 0,
            inspections: 0,
            loading: false,
            disabled: false,
            no_columns: false,
            extra: false,
            renamed: false,
            disable_value: false,
        }
    }
}
fn harness(size: [f32; 2]) -> Harness<'static, State> {
    Harness::builder()
        .with_size(size)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                let outside = ui.button("Outside");
                let mut columns = vec![
                    TableColumn::new(
                        Column::Name,
                        if state.renamed { "Record name" } else { "Name" },
                    )
                    .sortable(true)
                    .width(TableColumnWidth::Flexible(160.0)),
                    TableColumn::new(Column::Value, "Value")
                        .sortable(true)
                        .enabled(!state.disable_value)
                        .width(TableColumnWidth::Fixed(140.0)),
                    TableColumn::new(Column::Edit, "Edit").width(TableColumnWidth::Fixed(180.0)),
                ];
                if state.extra {
                    columns.insert(
                        0,
                        TableColumn::new(Column::Extra, "Extra")
                            .width(TableColumnWidth::Fixed(100.0)),
                    );
                }
                if state.no_columns {
                    columns.clear();
                }
                state.calls = 0;
                let result = DataTable::new("records", &columns, &state.records)
                    .accessible_name("Records")
                    .sort(state.sort.as_ref())
                    .selection(|row| state.selected.contains(&row.id))
                    .row_enabled(|row| row.id != 3)
                    .loading(state.loading)
                    .enabled(!state.disabled)
                    .max_height(240.0)
                    .return_focus_to(outside.id)
                    .show(
                        ui,
                        |row| row.id,
                        |ui, row, column| {
                            state.calls += 1;
                            match column {
                                Column::Name => {
                                    ui.label(&row.name);
                                }
                                Column::Value => {
                                    if ui
                                        .button(format!("Inspect {}: {}", row.id, row.value))
                                        .clicked()
                                    {
                                        state.inspections += 1;
                                    }
                                }
                                Column::Edit => {
                                    let edit = state.edits.entry(row.id).or_default();
                                    ui.add(
                                        egui::TextEdit::singleline(edit)
                                            .desired_width(ui.available_width()),
                                    )
                                    .widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            Role::TextInput,
                                            ui.is_enabled(),
                                            format!("Edit {}", row.id),
                                        )
                                    });
                                }
                                Column::Extra => {
                                    ui.label("Extra");
                                }
                            }
                        },
                    );
                state.sorts.extend(result.sort_requested);
                state.selections.extend(result.selection_requested);
            },
            State::default(),
        )
}
#[test]
fn native_table_rows_cells_and_headers_expose_indices_and_sort_requests() {
    let mut h = harness([840.0, 600.0]);
    h.run_steps(3);
    let table = h.get_by_role_and_label(Role::Table, "Records");
    assert_eq!(table.accesskit_node().data().row_count(), Some(6));
    assert_eq!(table.accesskit_node().data().column_count(), Some(4));
    let header = table.get_by_role_and_label(Role::ColumnHeader, "Name");
    assert_eq!(header.accesskit_node().data().column_index(), Some(1));
    header.get_by_label("Sort by Name").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(
        h.state().sorts,
        [TableSort {
            column: Column::Name,
            direction: TableSortDirection::Ascending
        }]
    );
    assert!(h.state().sort.is_none());
    h.state_mut().sort = Some(h.state().sorts[0].clone());
    h.run_steps(3);
    assert_eq!(
        h.get_by_role_and_label(Role::ColumnHeader, "Name")
            .accesskit_node()
            .data()
            .sort_direction(),
        Some(SortDirection::Ascending)
    );
    h.key_press(Key::Space);
    h.run_steps(3);
    assert_eq!(h.state().sorts[1].direction, TableSortDirection::Descending);
    h.check_accessibility();
}
#[test]
fn selection_requests_are_host_owned_and_select_all_skips_disabled_rows() {
    let mut h = harness([840.0, 600.0]);
    h.state_mut().selected.insert(1);
    h.run_steps(3);
    let all = h.get_by_label("Select all visible rows");
    assert_eq!(all.accesskit_node().data().toggled(), Some(Toggled::Mixed));
    all.focus();
    h.run();
    h.key_press(Key::Space);
    h.run_steps(3);
    assert_eq!(
        h.state()
            .selections
            .iter()
            .map(|r| r.key)
            .collect::<Vec<_>>(),
        [2, 4, 5]
    );
    assert_eq!(h.state().selected.iter().copied().collect::<Vec<_>>(), [1]);
    h.get_by_label("Select row 1").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(
        h.state().selections.last(),
        Some(&TableSelection {
            key: 1,
            selected: false
        })
    );
    h.get_by_label("Select row 3").click();
    h.run_steps(2);
    assert_eq!(h.state().selections.len(), 4);
}
#[test]
fn controls_remain_independent_and_disabled_table_or_column_cannot_request_changes() {
    let mut h = harness([840.0, 600.0]);
    h.get_by_label("Inspect 1: 10").click();
    h.run_steps(3);
    assert_eq!(h.state().inspections, 1);
    assert!(h.state().selections.is_empty());
    h.state_mut().disable_value = true;
    h.run();
    h.get_by_label("Sort by Value").click();
    h.run_steps(2);
    assert!(h.state().sorts.is_empty());
    h.state_mut().disabled = true;
    h.run();
    h.get_by_label("Sort by Name").click();
    h.run_steps(2);
    h.get_by_label("Select row 1").click();
    h.run_steps(2);
    assert!(h.state().sorts.is_empty());
    assert!(h.state().selections.is_empty());
}
#[test]
fn keyed_editors_preserve_values_and_focus_across_reorder_filter_and_column_insertion() {
    let mut h = harness([840.0, 600.0]);
    h.get_by_label("Edit 1").focus();
    h.run();
    h.get_by_label("Edit 1").type_text("Retained");
    h.run_steps(2);
    let id = h.get_by_label("Edit 1").accesskit_node().id();
    h.state_mut().records.reverse();
    h.state_mut().extra = true;
    h.state_mut().renamed = true;
    h.run_steps(4);
    assert_eq!(h.get_by_label("Edit 1").accesskit_node().id(), id);
    assert!(h.get_by_label("Edit 1").is_focused());
    assert_eq!(
        h.state().edits.get(&1).map(String::as_str),
        Some("Retained")
    );
    h.state_mut().records.retain(|row| row.id == 1);
    h.run_steps(3);
    assert!(h.get_by_label("Edit 1").is_focused());
}
#[test]
fn removed_focused_records_restore_the_surviving_host_target() {
    let mut h = harness([840.0, 600.0]);
    h.get_by_label("Edit 1").focus();
    h.run_steps(2);
    h.state_mut().records.retain(|row| row.id != 1);
    h.run_steps(4);
    assert!(h.get_by_label("Outside").is_focused());
}
#[test]
fn loading_empty_and_missing_columns_do_not_invoke_data_cells() {
    let mut h = harness([840.0, 600.0]);
    assert_eq!(h.state().calls, 15);
    h.state_mut().loading = true;
    h.run_steps(3);
    assert_eq!(h.state().calls, 0);
    assert!(
        h.get_by_role_and_label(Role::Table, "Records")
            .accesskit_node()
            .data()
            .is_busy()
    );
    assert!(h.query_by_label("Loading Records").is_some());
    h.state_mut().loading = false;
    h.state_mut().records.clear();
    h.run_steps(3);
    assert_eq!(h.state().calls, 0);
    assert!(h.query_by_label("NO DATA AVAILABLE").is_some());
    let table = h.get_by_role_and_label(Role::Table, "Records");
    assert_eq!(table.accesskit_node().data().row_count(), Some(2));
    h.state_mut().no_columns = true;
    h.run_steps(2);
    assert!(h.query_by_role_and_label(Role::Table, "Records").is_none());
    assert!(h.query_by_label("NO COLUMNS CONFIGURED").is_some());
    h.check_accessibility();
}
