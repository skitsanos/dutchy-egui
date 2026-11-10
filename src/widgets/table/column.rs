use crate::tokens::table as geometry;
use egui::IdSalt;
use std::borrow::Cow;
/// Content alignment within a table column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TableAlign {
    /// Leading edge.
    #[default]
    Left,
    /// Centered content.
    Center,
    /// Trailing edge, useful for quantities/actions.
    Right,
}
impl TableAlign {
    pub(super) fn align(self) -> egui::Align {
        match self {
            Self::Left => egui::Align::Min,
            Self::Center => egui::Align::Center,
            Self::Right => egui::Align::Max,
        }
    }
}
/// Preferred width; flexible columns share extra viewport space equally.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum TableColumnWidth {
    /// Fixed logical-point width, with horizontal overflow when needed.
    Fixed(f32),
    /// Minimum logical-point width; receives an equal share of remaining space.
    Flexible(f32),
}
impl Default for TableColumnWidth {
    fn default() -> Self {
        Self::Flexible(geometry::COLUMN_WIDTH)
    }
}
/// Keyed data column. Keys must be unique and immutable within the table.
#[derive(Clone, Debug)]
pub struct TableColumn<'a, C> {
    /// Host key passed to the cell closure and sort requests.
    pub key: C,
    /// Visible heading and accessible column name.
    pub label: Cow<'a, str>,
    pub(super) id: IdSalt,
    pub(super) width: TableColumnWidth,
    pub(super) align: TableAlign,
    pub(super) sortable: bool,
    pub(super) enabled: bool,
}
impl<'a, C: egui::AsIdSalt> TableColumn<'a, C> {
    /// Flexible, leading-aligned, nonsortable column.
    pub fn new(key: C, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&key),
            key,
            label: label.into(),
            width: TableColumnWidth::default(),
            align: TableAlign::Left,
            sortable: false,
            enabled: true,
        }
    }
}
impl<C> TableColumn<'_, C> {
    /// Fixed width or flexible minimum. Finite values clamp to shared 96–4096
    /// bounds; nonfinite values use the default 160-point minimum.
    pub fn width(mut self, width: TableColumnWidth) -> Self {
        self.width = width;
        self
    }
    /// Header and cell content alignment.
    pub fn align(mut self, align: TableAlign) -> Self {
        self.align = align;
        self
    }
    /// Expose a native keyboard/pointer sort button.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
    /// Enable this column's sort button; does not disable its cell content.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
