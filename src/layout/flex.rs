use crate::Spacing;
use egui::{Align, IdSalt, InnerResponse, Layout, Ui, UiBuilder};

/// Native main-axis direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    /// Left to right.
    Row,
    /// Top to bottom.
    Column,
    /// Right to left.
    RowReverse,
    /// Bottom to top, within the available height.
    ColumnReverse,
}

/// Native cross-axis alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossAlign {
    /// Leading edge.
    Start,
    /// Centered.
    Center,
    /// Trailing edge.
    End,
    /// Ask children to fill the cross axis.
    Stretch,
}

/// Row or column composition. Wrapping applies to rows only.
///
/// Children retain normal egui sizing and interaction. Stretch is native cross
/// justification, not CSS equal-height sizing. There is no baseline alignment or
/// CSS space-between/around/evenly distribution.
#[derive(Clone, Debug)]
#[must_use = "Call show to lay out content"]
pub struct Flex {
    id: IdSalt,
    direction: FlexDirection,
    align: CrossAlign,
    gap: Spacing,
    wrap: bool,
}

impl Flex {
    /// Left-to-right row, centered on the cross axis, with an eight-point gap.
    pub fn row(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: IdSalt::new(id),
            direction: FlexDirection::Row,
            align: CrossAlign::Center,
            gap: Spacing::Sm,
            wrap: false,
        }
    }
    /// Top-to-bottom column, aligned to the leading edge.
    pub fn column(id: impl egui::AsIdSalt) -> Self {
        Self::row(id)
            .direction(FlexDirection::Column)
            .align(CrossAlign::Start)
    }
    /// Main-axis direction; reverse columns need bounded host height.
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }
    /// Cross-axis alignment.
    pub fn align(mut self, align: CrossAlign) -> Self {
        self.align = align;
        self
    }
    /// Shared spacing on both axes, including wrapped row gaps.
    pub fn gap(mut self, gap: Spacing) -> Self {
        self.gap = gap;
        self
    }
    /// Wrap rows at the available width. Columns remain vertical.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    /// Render once in an explicitly stable ID scope.
    pub fn show<R>(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let id = ui.make_persistent_id(("dutchy-flex", self.id));
        ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::splat(self.gap.points());
            let horizontal = matches!(
                self.direction,
                FlexDirection::Row | FlexDirection::RowReverse
            );
            let layout = self.layout().with_main_wrap(horizontal && self.wrap);
            if horizontal {
                let initial = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                ui.allocate_ui_with_layout(initial, layout, contents).inner
            } else {
                ui.with_layout(layout, contents).inner
            }
        })
    }

    fn layout(&self) -> Layout {
        let align = match self.align {
            CrossAlign::Start | CrossAlign::Stretch => Align::Min,
            CrossAlign::Center => Align::Center,
            CrossAlign::End => Align::Max,
        };
        let layout = match self.direction {
            FlexDirection::Row => Layout::left_to_right(align),
            FlexDirection::Column => Layout::top_down(align),
            FlexDirection::RowReverse => Layout::right_to_left(align),
            FlexDirection::ColumnReverse => Layout::bottom_up(align),
        };
        layout.with_cross_justify(self.align == CrossAlign::Stretch)
    }
}
