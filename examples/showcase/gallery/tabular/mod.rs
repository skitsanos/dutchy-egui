mod records;
use super::section;
use dutchy_egui::{
    Badge, BadgeVariant, Button, ButtonVariant, Checkbox, ControlSize, DataTable, Family, Input,
    Pagination, TableAlign, TableColumn, TableColumnWidth, TableDensity, TableHeader, TableSort,
    TableSortDirection, Text, TextColor, TextSize, Weight, tokens::space,
};
use egui::Ui;
pub use records::{Column, Record};
use std::collections::BTreeSet;
pub struct TableState {
    pub records: Vec<Record>,
    pub query: String,
    pub page: usize,
    pub sort: Option<TableSort<Column>>,
    pub selected: BTreeSet<u8>,
    pub loading: bool,
    pub striped: bool,
    pub compact: bool,
    pub inverse: bool,
    pub inspections: usize,
    pub inspected: Option<u8>,
}
impl Default for TableState {
    fn default() -> Self {
        Self {
            records: records::records(),
            query: String::new(),
            page: 1,
            sort: None,
            selected: BTreeSet::new(),
            loading: false,
            striped: false,
            compact: false,
            inverse: false,
            inspections: 0,
            inspected: None,
        }
    }
}
impl TableState {
    pub fn filtered(&self) -> Vec<Record> {
        let query = self.query.trim().to_lowercase();
        let mut rows: Vec<_> = self
            .records
            .iter()
            .filter(|row| {
                format!("{} {}", row.name, row.email)
                    .to_lowercase()
                    .contains(&query)
            })
            .cloned()
            .collect();
        if let Some(sort) = &self.sort {
            rows.sort_by(|a, b| {
                let order = match sort.column {
                    Column::Name => a.name.cmp(&b.name),
                    Column::Email => a.email.cmp(&b.email),
                    Column::Budget => a.cents.cmp(&b.cents),
                    _ => a.id.cmp(&b.id),
                };
                if sort.direction == TableSortDirection::Descending {
                    order.reverse()
                } else {
                    order
                }
            });
        }
        rows
    }
    pub fn show(&mut self, ui: &mut Ui) {
        ui.add(Text::heading("Structure. At a glance."));
        ui.add(Text::new("The host filters, sorts typed values, and applies selection. The table presents the current view."));
        section(ui, "01", "Search / host-owned presentation", |ui| {
            self.toolbar(ui)
        });
        section(ui, "02", "Keyed records / native controls", |ui| {
            self.table(ui)
        });
        section(ui, "03", "Compact / inverse header", |ui| {
            self.inventory(ui)
        });
    }
    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(
                    Input::new("table-query", &mut self.query)
                        .accessible_name("Search records")
                        .placeholder("Search name or email")
                        .width(280.0),
                )
                .changed()
            {
                self.page = 1;
            }
            if ui
                .add(
                    Button::new("Clear search")
                        .variant(ButtonVariant::Outline)
                        .size(ControlSize::Small),
                )
                .clicked()
            {
                self.query.clear();
                self.page = 1;
            }
        });
        ui.add_space(space::MD);
        ui.horizontal_wrapped(|ui| {
            ui.add(Checkbox::new(
                "table-loading",
                &mut self.loading,
                "Loading records",
            ));
            ui.add(Checkbox::new(
                "table-striped",
                &mut self.striped,
                "Striped rows",
            ));
            ui.add(Checkbox::new(
                "table-compact",
                &mut self.compact,
                "Compact cells",
            ));
            ui.add(Checkbox::new(
                "table-inverse",
                &mut self.inverse,
                "Inverse header",
            ));
        });
        ui.add(Text::new("The fourth record is disabled. Select-all affects enabled rows on the current page; filtered-out selections are retained in the host.").color(TextColor::Muted));
    }
    fn table(&mut self, ui: &mut Ui) {
        let rows = self.filtered();
        let pages = rows.len().div_ceil(6);
        self.page = self.page.clamp(1, pages.max(1));
        let start = (self.page - 1) * 6;
        let current = &rows[start.min(rows.len())..(start + 6).min(rows.len())];
        let mut actions = Vec::new();
        let fallback = ui.add(
            Button::new("Clear selection")
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Small),
        );
        if fallback.clicked() {
            self.selected.clear();
        }
        ui.add(
            Text::label(format!(
                "{} matching / {} selected across records",
                rows.len(),
                self.selected.len()
            ))
            .color(TextColor::Muted),
        );
        ui.add_space(space::SM);
        let columns = records::columns();
        let result = DataTable::new("workspace-records", &columns, current)
            .accessible_name("Workspace records")
            .sort(self.sort.as_ref())
            .selection(|row| self.selected.contains(&row.id))
            .row_enabled(|row| row.id != 4)
            .loading(self.loading)
            .striped(self.striped)
            .density(if self.compact {
                TableDensity::Compact
            } else {
                TableDensity::Comfortable
            })
            .header(if self.inverse {
                TableHeader::Inverse
            } else {
                TableHeader::Muted
            })
            .max_height(320.0)
            .return_focus_to(fallback.id)
            .empty_text(
                "No matching records",
                "Change your search to find a record.",
            )
            .show(
                ui,
                |row| row.id,
                |ui, row, column| cell(ui, row, *column, &mut actions),
            );
        if let Some(sort) = result.sort_requested {
            self.sort = Some(sort);
            self.page = 1;
            ui.ctx().request_repaint();
        }
        for request in result.selection_requested {
            if request.selected {
                self.selected.insert(request.key);
            } else {
                self.selected.remove(&request.key);
            }
        }
        for id in actions {
            self.inspected = Some(id);
            self.inspections += 1;
        }
        ui.add_space(space::MD);
        if let Some(page) = Pagination::new(
            "record-pages",
            if pages == 0 { 0 } else { self.page },
            pages,
        )
        .accessible_name("Record pages")
        .show(ui)
        .requested_page
        {
            self.page = page;
        }
        if let Some(record) = self
            .records
            .iter()
            .find(|row| Some(row.id) == self.inspected)
        {
            ui.add_space(space::SM);
            ui.add(
                Text::new(format!(
                    "Host inspected: {} · {} · {}",
                    record.name,
                    record.email,
                    record.budget()
                ))
                .size(TextSize::Sm),
            );
        }
    }
    fn inventory(&self, ui: &mut Ui) {
        let columns = [
            TableColumn::new(Column::Name, "Project").width(TableColumnWidth::Flexible(240.0)),
            TableColumn::new(Column::Budget, "Budget")
                .align(TableAlign::Right)
                .width(TableColumnWidth::Fixed(160.0)),
        ];
        DataTable::new("compact-budget", &columns, &self.records[..3])
            .accessible_name("Compact budget")
            .density(TableDensity::Compact)
            .header(TableHeader::Inverse)
            .striped(true)
            .show(
                ui,
                |row| row.id,
                |ui, row, column| {
                    if *column == Column::Name {
                        ui.add(Text::new(&row.name).size(TextSize::Sm).weight(Weight::Bold));
                    } else {
                        ui.add(
                            Text::new(row.budget())
                                .family(Family::Mono)
                                .size(TextSize::Sm),
                        );
                    }
                },
            );
    }
}
fn cell(ui: &mut Ui, row: &Record, column: Column, actions: &mut Vec<u8>) {
    match column {
        Column::Name => {
            ui.add(Text::new(&row.name).size(TextSize::Sm).weight(Weight::Bold));
        }
        Column::Email => {
            ui.add(
                Text::new(&row.email)
                    .size(TextSize::Sm)
                    .color(TextColor::Muted),
            );
        }
        Column::Status => {
            ui.add(
                Badge::new(if row.active { "Active" } else { "Inactive" }).variant(if row.active {
                    BadgeVariant::Success
                } else {
                    BadgeVariant::Muted
                }),
            );
        }
        Column::Budget => {
            ui.add(
                Text::new(row.budget())
                    .family(Family::Mono)
                    .size(TextSize::Sm),
            );
        }
        Column::Actions => {
            let response = ui.add(
                Button::new("View")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::accesskit::Role::Button,
                    ui.is_enabled(),
                    format!("View {}", row.name),
                )
            });
            if response.clicked() {
                actions.push(row.id);
            }
        }
    }
}
