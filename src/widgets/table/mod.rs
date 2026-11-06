mod cells;
mod column;
mod focus;
mod header;
mod heights;
mod model;
mod render;
mod row;
mod selection;
mod state;
mod types;
use crate::tokens::table as geometry;
pub use column::{TableAlign, TableColumn, TableColumnWidth};
use egui::{Id, IdSalt, Ui};
use std::borrow::Cow;
pub use types::{
    DataTableResponse, TableDensity, TableHeader, TableRowResponse, TableSelection, TableSort,
    TableSortDirection,
};
type Predicate<'a, R> = Box<dyn Fn(&R) -> bool + 'a>;
/// Keyed, composable table with a sticky header and native scrolling.
/// The host owns record order, filtering, selection, actions, and loading.
#[must_use = "Call show to render the table"]
pub struct DataTable<'a, R, C> {
    id: IdSalt,
    columns: &'a [TableColumn<'a, C>],
    rows: &'a [R],
    name: Cow<'a, str>,
    sort: Option<&'a TableSort<C>>,
    selection: Option<Predicate<'a, R>>,
    row_enabled: Option<Predicate<'a, R>>,
    header: TableHeader,
    density: TableDensity,
    striped: bool,
    bordered: bool,
    enabled: bool,
    loading: bool,
    max_height: f32,
    empty_title: Cow<'a, str>,
    empty_description: Cow<'a, str>,
    return_focus: Option<Id>,
}
impl<'a, R, C: Clone + PartialEq> DataTable<'a, R, C> {
    /// Bind stable column keys and immutable current-view records.
    pub fn new(id: impl egui::AsIdSalt, columns: &'a [TableColumn<'a, C>], rows: &'a [R]) -> Self {
        Self {
            id: IdSalt::new(id),
            columns,
            rows,
            name: Cow::Borrowed("Data table"),
            sort: None,
            selection: None,
            row_enabled: None,
            header: TableHeader::default(),
            density: TableDensity::default(),
            striped: false,
            bordered: true,
            enabled: true,
            loading: false,
            max_height: geometry::BODY_HEIGHT,
            empty_title: Cow::Borrowed("No data available"),
            empty_description: Cow::Borrowed("Try adjusting your search or filters."),
            return_focus: None,
        }
    }
    /// Table and overflow viewport name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Current host-applied sort; clicking a header returns its next requested state.
    pub fn sort(mut self, sort: Option<&'a TableSort<C>>) -> Self {
        self.sort = sort;
        self
    }
    /// Add row/select-all controls bound to host-supplied current selection values.
    pub fn selection(mut self, selected: impl Fn(&R) -> bool + 'a) -> Self {
        self.selection = Some(Box::new(selected));
        self
    }
    /// Disable selection and cell controls for particular records.
    pub fn row_enabled(mut self, enabled: impl Fn(&R) -> bool + 'a) -> Self {
        self.row_enabled = Some(Box::new(enabled));
        self
    }
    /// Header surface.
    pub fn header(mut self, header: TableHeader) -> Self {
        self.header = header;
        self
    }
    /// Cell/header padding density.
    pub fn density(mut self, density: TableDensity) -> Self {
        self.density = density;
        self
    }
    /// Alternate muted row backgrounds.
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }
    /// Outer table border; row/header separators remain present.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
    /// Disable table controls, respecting disabled parent UIs.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Replace stale records with a named busy skeleton body; cell closures are not run.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Body viewport height cap, clamped to 96–4096 points. Nonfinite uses 384.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = if height.is_finite() {
            height.clamp(geometry::MIN_BODY_HEIGHT, geometry::MAX_BODY_HEIGHT)
        } else {
            geometry::BODY_HEIGHT
        };
        self
    }
    /// Replace the default empty-state guidance.
    pub fn empty_text(
        mut self,
        title: impl Into<Cow<'a, str>>,
        description: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.empty_title = title.into();
        self.empty_description = description.into();
        self
    }
    /// Surviving host destination if a focused data row/column is removed or replaced by a state body.
    pub fn return_focus_to(mut self, id: Id) -> Self {
        self.return_focus = Some(id);
        self
    }
    /// Render current-view records once per cell per egui pass. Keys must be unique
    /// and stable. The cell closure composes normal egui/Dutchy controls; it is not
    /// called while loading, empty, or without columns. This first version is not virtualized.
    pub fn show<K: egui::AsIdSalt + Clone>(
        self,
        ui: &mut Ui,
        key: impl Fn(&'a R) -> K,
        cell: impl FnMut(&mut Ui, &'a R, &C),
    ) -> DataTableResponse<C, K> {
        render::show(self, ui, key, cell)
    }
}
