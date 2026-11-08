//! Shared action and toolbar geometry.
use super::{ControlSize, border, space};

/// Small action glyph allocation.
pub const ICON_SMALL: f32 = space::LG;
/// Medium action glyph allocation.
pub const ICON_MEDIUM: f32 = space::XL;
/// Large action glyph allocation.
pub const ICON_LARGE: f32 = space::XXL;
/// Gap between icon and visible label.
pub const ICON_GAP: f32 = space::SM;
/// Toolbar content inset.
pub const PADDING: f32 = space::SM;
/// Gap between adjacent toolbar controls.
pub const GAP: f32 = space::XS;
/// Space reserved for a toolbar separator.
pub const SEPARATOR: f32 = space::LG;
/// Held pointer fill multiplier.
pub const PRESSED_TINT: f32 = 0.8;
/// Selected-state edge for Secondary toggles, whose fill is already inverse.
pub const PRESSED_MARKER: f32 = border::ACCENT;

/// Glyph allocation for a shared control size.
pub const fn icon(size: ControlSize) -> f32 {
    match size {
        ControlSize::Small => ICON_SMALL,
        ControlSize::Medium => ICON_MEDIUM,
        ControlSize::Large => ICON_LARGE,
    }
}
/// Default narrow vertical toolbar width.
pub const fn vertical_width(size: ControlSize) -> f32 {
    size.height() + 2.0 * (PADDING + border::STANDARD)
}
