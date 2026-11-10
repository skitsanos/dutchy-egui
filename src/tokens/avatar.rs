//! Avatar sides, initials type and presence-marker geometry.
use super::{TextSize, border, space};

/// Square avatar side tier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum AvatarSize {
    /// 24-point side, for dense rows.
    Small,
    /// 32-point side, the default.
    #[default]
    Medium,
    /// 48-point side, for headers.
    Large,
    /// 64-point side, for profiles.
    XLarge,
}

impl AvatarSize {
    /// Every size in ascending order.
    pub const ALL: &[Self] = &[Self::Small, Self::Medium, Self::Large, Self::XLarge];

    /// Side length in logical points.
    pub const fn side(self) -> f32 {
        match self {
            Self::Small => space::XXL,
            Self::Medium => space::SECTION,
            Self::Large => space::HERO,
            Self::XLarge => space::MAJOR,
        }
    }

    /// Initials text size.
    pub const fn text_size(self) -> TextSize {
        match self {
            Self::Small => TextSize::Xs,
            Self::Medium => TextSize::Sm,
            Self::Large => TextSize::Lg,
            Self::XLarge => TextSize::Xl2,
        }
    }

    /// Side of the square presence marker, excluding its ring. Each tier keeps
    /// the marker clear of two centered initials.
    pub const fn marker(self) -> f32 {
        match self {
            Self::Small => space::XS,
            Self::Medium => space::SM,
            Self::Large => space::MD,
            Self::XLarge => space::LG,
        }
    }

    /// Human-readable size name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::XLarge => "Extra large",
        }
    }
}

/// Background-colored ring that separates the presence marker from the avatar.
pub const MARKER_RING: f32 = border::STANDARD;
/// Border of the outline variant.
pub const OUTLINE: f32 = border::STANDARD;
