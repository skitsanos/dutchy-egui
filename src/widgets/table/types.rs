use crate::tokens::table as geometry;
use egui::Response;
/// Host-selected single-column sorting direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableSortDirection {
    /// Smaller/earlier values first.
    Ascending,
    /// Larger/later values first.
    Descending,
}
/// Requested/applied sort. The host compares typed values and orders records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableSort<C> {
    /// Host column key.
    pub column: C,
    /// Desired ordering.
    pub direction: TableSortDirection,
}
/// Header surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableHeader {
    /// Documented muted header.
    #[default]
    Muted,
    /// Inverse foreground fill, as in the striped reference.
    Inverse,
    /// Live component's background fill and muted heading text.
    Plain,
}
/// Shared table padding density.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableDensity {
    /// 24x16 body and 24x12 header padding.
    #[default]
    Comfortable,
    /// 16x8 cell/header padding.
    Compact,
}
impl TableDensity {
    pub(super) fn margin(self, header: bool) -> egui::Margin {
        match self {
            Self::Comfortable => egui::Margin::symmetric(
                geometry::PADDING_X,
                if header {
                    geometry::HEADER_Y
                } else {
                    geometry::PADDING_Y
                },
            ),
            Self::Compact => egui::Margin::symmetric(geometry::COMPACT_X, geometry::COMPACT_Y),
        }
    }
}
/// Selection request for one current-view row; the host accepts or rejects it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableSelection<K> {
    /// Stable row key.
    pub key: K,
    /// Requested selection value.
    pub selected: bool,
}
/// Rendered keyed row and its native cell/selection responses.
pub struct TableRowResponse<K> {
    /// Host row key.
    pub key: K,
    /// Semantic row surface.
    pub response: Response,
    /// Cell scopes in display order, including selection when enabled.
    pub cells: Vec<Response>,
    /// Optional native row selection control.
    pub selection: Option<Response>,
}
/// Table output. No record sorting, filtering, selection or actions are applied.
pub struct DataTableResponse<C, K> {
    /// Named viewport region.
    pub response: Response,
    /// Sort request from a native column button.
    pub sort_requested: Option<TableSort<C>>,
    /// Per-row selection requests. Select-all affects enabled current-view rows only.
    pub selection_requested: Vec<TableSelection<K>>,
    /// Header scopes, including selection when enabled.
    pub headers: Vec<Response>,
    /// Current data rows. Empty/loading presentations return no data rows.
    pub rows: Vec<TableRowResponse<K>>,
}
