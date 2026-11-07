//! Menu, dropdown, and command-search geometry.
use super::space;

/// Shared menu widths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuWidth {
    /// 200-point dropdown.
    Compact,
    /// 320-point reference menu.
    #[default]
    Small,
    /// 384-point menu.
    Medium,
    /// 448-point menu.
    Large,
}
impl MenuWidth {
    /// Preferred width, constrained further by the parent/viewport.
    pub const fn points(self) -> f32 {
        match self {
            Self::Compact => 200.0,
            Self::Small => 320.0,
            Self::Medium => 384.0,
            Self::Large => 448.0,
        }
    }
}
/// Action row horizontal padding.
pub const PADDING_X: f32 = space::LG;
/// Action row vertical padding.
pub const PADDING_Y: f32 = space::MD;
/// Minimum row height.
pub const ROW_HEIGHT: f32 = 44.0;
/// Decorative icon allocation.
pub const ICON_SIZE: f32 = space::XL;
/// Space between icon, label, and shortcut columns.
pub const ITEM_GAP: f32 = space::MD;
/// Group-label vertical padding.
pub const GROUP_PADDING: i8 = 8;
/// Gap between trigger and popup.
pub const POPUP_GAP: f32 = space::XS;
/// Searchable result-list maximum height.
pub const RESULTS_HEIGHT: f32 = 240.0;
/// Command result-list maximum height.
pub const COMMANDS_HEIGHT: f32 = 320.0;
/// Dropdown list cap; the web dropdown has none.
pub const DROPDOWN_MAX_HEIGHT: f32 = 320.0;
/// Command dialog width.
pub const PALETTE_WIDTH: f32 = 512.0;
/// Command header padding.
pub const SEARCH_PADDING: i8 = 16;
/// Command backdrop opacity.
pub const BACKDROP_OPACITY: f32 = 0.5;
