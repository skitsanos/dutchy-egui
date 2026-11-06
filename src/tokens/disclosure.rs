//! Accordion and tab geometry in logical points.

use super::{border, space};

/// Accordion header and panel horizontal padding.
pub const PADDING_X: f32 = space::XXL;
/// Default accordion header padding on the vertical axis.
pub const ACCORDION_PADDING_Y: f32 = space::XXL;
/// Bordered accordion header/panel vertical padding.
pub const BORDERED_PADDING_Y: f32 = space::LG;
/// Space between independent accent-edge accordion items.
pub const ACCORDION_GAP: f32 = space::SM;
/// Disclosure chevron size.
pub const CHEVRON: f32 = space::XL;
/// Gap between a wrapping header and its chevron.
pub const CHEVRON_GAP: f32 = space::LG;
/// Underline tab height, including its accent edge.
pub const TAB_HEIGHT: f32 = 56.0;
/// Segmented tab height before the surrounding inset.
pub const SEGMENT_HEIGHT: f32 = 44.0;
/// Segmented control's surrounding inset.
pub const SEGMENT_INSET: i8 = 4;
/// Tab-panel padding.
pub const PANEL_PADDING: i8 = 24;
/// Width of a tab close affordance.
pub const CLOSE_SIZE: f32 = space::LG;
/// Gap preceding a tab close affordance.
pub const CLOSE_GAP: f32 = space::XS;
/// Tab close icon inset.
pub const CLOSE_INSET: f32 = space::XS;
/// Active tab underline thickness.
pub const TAB_UNDERLINE: f32 = border::ACCENT;
