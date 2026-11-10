//! Modal and drawer geometry in logical points.
use super::space;
/// Centered dialog width, constrained by the viewport.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModalWidth {
    /// 448-point reference confirmation dialog.
    #[default]
    Small,
    /// 512-point content dialog.
    Medium,
    /// 672-point extended content dialog.
    Large,
}
impl ModalWidth {
    /// Preferred outer width.
    pub const fn points(self) -> f32 {
        match self {
            Self::Small => 448.0,
            Self::Medium => 512.0,
            Self::Large => 672.0,
        }
    }
}
/// Overlay drawer width, constrained by the viewport.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DrawerWidth {
    /// 320-point reference drawer.
    #[default]
    Small,
    /// 384-point panel.
    Medium,
    /// 448-point panel.
    Large,
}
impl DrawerWidth {
    /// Preferred outer width.
    pub const fn points(self) -> f32 {
        match self {
            Self::Small => 320.0,
            Self::Medium => 384.0,
            Self::Large => 448.0,
        }
    }
}
/// Minimum modal clearance from viewport edges.
pub const VIEWPORT_GUTTER: f32 = space::LG;
/// Header and footer horizontal padding.
pub const PADDING_X: i8 = 24;
/// Header and footer vertical padding.
pub const PADDING_Y: i8 = 16;
/// Body padding.
pub const BODY_PADDING: i8 = 24;
/// Header title/close spacing.
pub const TITLE_GAP: f32 = space::LG;
/// Shared close glyph extent.
pub const CLOSE_ICON: f32 = space::LG;
/// Foreground opacity of the blocking backdrop.
pub const BACKDROP_OPACITY: f32 = 0.5;
