use dutchy_egui::{TableAlign, TableColumn, TableColumnWidth};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Column {
    Name,
    Email,
    Status,
    Budget,
    Actions,
}
#[derive(Clone)]
pub struct Record {
    pub id: u8,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub cents: u32,
}
impl Record {
    pub fn budget(&self) -> String {
        format!("${}.{:02}", self.cents / 100, self.cents % 100)
    }
}
pub fn records() -> Vec<Record> {
    [
        "Ada Lovelace",
        "Grace Hopper",
        "Alan Turing",
        "Katherine Johnson",
        "Margaret Hamilton",
        "Donald Knuth",
        "Barbara Liskov",
        "Edsger Dijkstra",
        "Frances Allen",
        "John McCarthy",
        "Mary Jackson",
        "Claude Shannon",
        "Dorothy Vaughan",
        "Ken Thompson",
        "Radia Perlman",
        "Dennis Ritchie",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, name)| Record {
        id: i as u8 + 1,
        name: name.into(),
        email: format!("{}@example.test", name.to_lowercase().replace(' ', ".")),
        active: i % 4 != 3,
        cents: ((i as u32 * 731 + 1275) % 18000) * 100 + 25,
    })
    .collect()
}
pub fn columns() -> [TableColumn<'static, Column>; 5] {
    [
        TableColumn::new(Column::Name, "Name")
            .sortable(true)
            .width(TableColumnWidth::Flexible(192.0)),
        TableColumn::new(Column::Email, "Email")
            .sortable(true)
            .width(TableColumnWidth::Flexible(240.0)),
        TableColumn::new(Column::Status, "Status").width(TableColumnWidth::Fixed(128.0)),
        TableColumn::new(Column::Budget, "Budget")
            .sortable(true)
            .align(TableAlign::Right)
            .width(TableColumnWidth::Fixed(144.0)),
        TableColumn::new(Column::Actions, "Actions")
            .align(TableAlign::Right)
            .width(TableColumnWidth::Fixed(128.0)),
    ]
}
