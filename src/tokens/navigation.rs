//! Breadcrumb and pagination geometry in logical points.
use super::{ControlSize, space};
/// Breadcrumb target height.
pub const BREADCRUMB_HEIGHT: f32 = ControlSize::Small.height();
/// Link inset, keeping focus inside the flow's spacing.
pub const LINK_PADDING: f32 = space::XS;
/// Space between breadcrumb segments.
pub const BREADCRUMB_GAP: f32 = space::SM;
/// Decorative separator/arrow size.
pub const ICON_SIZE: f32 = space::LG;
/// Inset within a decorative arrow.
pub const ICON_INSET: f32 = space::XS;
/// Muted breadcrumb bar horizontal padding.
pub const BAR_PADDING_X: i8 = 24;
/// Muted breadcrumb bar vertical padding.
pub const BAR_PADDING_Y: i8 = 12;
/// Minimum square numbered page/arrow target.
pub const PAGE_SIZE: f32 = space::LARGE;
/// Page-number horizontal padding, including for large page counts.
pub const PAGE_PADDING: f32 = space::SM;
/// Numbered pagination gap, matching the live component.
pub const PAGE_GAP: f32 = space::XS;
/// Gap in compact pagination.
pub const COMPACT_GAP: f32 = space::LG;
/// Largest dataset shown without an ellipsis.
pub const ALL_PAGES_LIMIT: usize = 7;
