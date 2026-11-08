//! Markdown document rhythm, list markers, quote rules and table cells.
use super::{border, space};

/// Vertical gap between consecutive blocks.
pub const BLOCK_GAP: f32 = space::LG;
/// Extra gap above a heading that follows another block.
pub const HEADING_GAP: f32 = space::SM;
/// Vertical gap between list items.
pub const ITEM_GAP: f32 = space::XS;
/// Gap between the blocks of one loose list item.
pub const ITEM_BLOCK_GAP: f32 = space::SM;
/// Width of the marker column in front of list content.
pub const MARKER_WIDTH: f32 = space::XXL;
/// Gap between the marker column and item content.
pub const MARKER_GAP: f32 = space::SM;
/// Side of a filled bullet square.
pub const BULLET: f32 = 6.0;
/// Side of a read-only task box.
pub const TASK_BOX: f32 = space::LG;
/// Width of the blockquote accent rule.
pub const QUOTE_RULE: f32 = border::ACCENT;
/// Padding inside a blockquote, after the rule.
pub const QUOTE_PADDING: f32 = space::LG;
/// Thematic break thickness.
pub const RULE: f32 = border::STANDARD;
/// Horizontal table cell padding.
pub const CELL_PADDING_X: f32 = space::LG;
/// Vertical table cell padding.
pub const CELL_PADDING_Y: f32 = space::SM;
/// Background expansion around inline code.
pub const INLINE_CODE_PAD: f32 = space::MICRO;
/// Resting link underline.
pub const LINK_UNDERLINE: f32 = border::THIN;
/// Hovered or focused link underline.
pub const LINK_HOVER_UNDERLINE: f32 = border::STANDARD;
