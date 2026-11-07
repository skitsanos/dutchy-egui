use super::{TableColumn, TableColumnWidth, TableSort, TableSortDirection};
use crate::tokens::table as geometry;
use egui::IdSalt;
pub(super) struct Layout {
    pub widths: Vec<f32>,
    pub width: f32,
    pub selection: bool,
}
impl Layout {
    pub fn new<C>(columns: &[TableColumn<'_, C>], available: f32, selection: bool) -> Self {
        let mut widths: Vec<_> = columns
            .iter()
            .map(|column| match column.width {
                TableColumnWidth::Fixed(width) | TableColumnWidth::Flexible(width) => {
                    normalize(width)
                }
            })
            .collect();
        if selection {
            widths.insert(0, geometry::SELECTION_WIDTH);
        }
        let minimum: f32 = widths.iter().sum();
        let available = if available.is_finite() {
            available.max(0.0)
        } else {
            minimum
        };
        let flexible = columns
            .iter()
            .filter(|c| matches!(c.width, TableColumnWidth::Flexible(_)))
            .count();
        if flexible > 0 {
            let extra = (available - minimum).max(0.0) / flexible as f32;
            for (index, column) in columns.iter().enumerate() {
                if matches!(column.width, TableColumnWidth::Flexible(_)) {
                    widths[index + usize::from(selection)] += extra;
                }
            }
        }
        Self {
            width: widths.iter().sum(),
            widths,
            selection,
        }
    }
}
fn normalize(width: f32) -> f32 {
    if width.is_finite() {
        width.clamp(geometry::MIN_COLUMN, geometry::MAX_COLUMN)
    } else {
        geometry::COLUMN_WIDTH
    }
}
pub(super) struct Row<K> {
    pub key: K,
    pub id: IdSalt,
    /// Widget scope; a repeated key receives an occurrence salt.
    pub scope: egui::Id,
    pub enabled: bool,
    pub selected: bool,
}
pub(super) fn next_sort<C: Clone + PartialEq>(
    column: &C,
    current: Option<&TableSort<C>>,
) -> TableSort<C> {
    let direction = if current.is_some_and(|sort| {
        sort.column == *column && sort.direction == TableSortDirection::Ascending
    }) {
        TableSortDirection::Descending
    } else {
        TableSortDirection::Ascending
    };
    TableSort {
        column: column.clone(),
        direction,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn widths_allocate_flexible_surplus_without_shrinking_fixed_or_minimums() {
        let columns = [
            TableColumn::new(1, "Fixed").width(TableColumnWidth::Fixed(100.0)),
            TableColumn::new(2, "Flexible").width(TableColumnWidth::Flexible(150.0)),
            TableColumn::new(3, "Flexible").width(TableColumnWidth::Flexible(150.0)),
        ];
        assert_eq!(
            Layout::new(&columns, 600.0, false).widths,
            [100.0, 250.0, 250.0]
        );
        assert_eq!(Layout::new(&columns, 200.0, false).width, 400.0);
        assert_eq!(
            Layout::new(&columns, 664.0, true).widths,
            [64.0, 100.0, 250.0, 250.0]
        );
        let invalid = [
            TableColumn::new(1, "Bad").width(TableColumnWidth::Fixed(f32::NAN)),
            TableColumn::new(2, "Negative").width(TableColumnWidth::Flexible(-100.0)),
        ];
        assert_eq!(
            Layout::new(&invalid, f32::INFINITY, false).widths,
            [160.0, 96.0]
        );
    }
    #[test]
    fn sort_requests_toggle_only_the_current_key_and_start_ascending() {
        let first = next_sort(&1, None);
        assert_eq!(first.direction, TableSortDirection::Ascending);
        let second = next_sort(&1, Some(&first));
        assert_eq!(second.direction, TableSortDirection::Descending);
        assert_eq!(next_sort(&1, Some(&second)), first);
        assert_eq!(
            next_sort(&2, Some(&first)).direction,
            TableSortDirection::Ascending
        );
    }
}
