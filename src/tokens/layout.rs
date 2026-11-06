//! Layout geometry from the Dutchy spacing and container scales.

use super::space;

/// Typed gaps and padding in logical points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spacing {
    /// No gap.
    None,
    /// Two points.
    Micro,
    /// Four points.
    Xs,
    /// Six points.
    Tight,
    /// Eight points.
    #[default]
    Sm,
    /// Twelve points.
    Md,
    /// Sixteen points.
    Lg,
    /// Twenty points.
    Xl,
    /// Twenty-four points.
    Xxl,
    /// Thirty-two points.
    Section,
    /// Forty points.
    Large,
    /// Forty-eight points.
    Hero,
    /// Sixty-four points.
    Major,
}

impl Spacing {
    /// Resolve the shared spacing token.
    pub const fn points(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Micro => space::MICRO,
            Self::Xs => space::XS,
            Self::Tight => space::TIGHT,
            Self::Sm => space::SM,
            Self::Md => space::MD,
            Self::Lg => space::LG,
            Self::Xl => space::XL,
            Self::Xxl => space::XXL,
            Self::Section => space::SECTION,
            Self::Large => space::LARGE,
            Self::Hero => space::HERO,
            Self::Major => space::MAJOR,
        }
    }
}

/// Maximum outer container widths, including horizontal gutters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContainerWidth {
    /// 384 points.
    Sm,
    /// 448 points.
    Md,
    /// 512 points.
    Lg,
    /// 576 points.
    Xl,
    /// 672 points.
    Xl2,
    /// 896 points.
    Xl4,
    /// 1152 points.
    Xl6,
    /// 1280 points.
    #[default]
    Xl7,
    /// All available width.
    Available,
}

impl ContainerWidth {
    /// Maximum width; `None` uses all available space.
    pub const fn points(self) -> Option<f32> {
        match self {
            Self::Sm => Some(384.0),
            Self::Md => Some(448.0),
            Self::Lg => Some(512.0),
            Self::Xl => Some(576.0),
            Self::Xl2 => Some(672.0),
            Self::Xl4 => Some(896.0),
            Self::Xl6 => Some(1152.0),
            Self::Xl7 => Some(1280.0),
            Self::Available => None,
        }
    }
}

/// Minimum responsive column widths before collapsing to fewer columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColumnWidth {
    /// Compact 160-point tiles.
    Compact,
    /// Standard 240-point columns.
    Standard,
    /// Reference auto-fit grid: 280 points.
    #[default]
    Card,
    /// Wide 320-point content.
    Wide,
    /// Form controls: 336 points.
    Form,
}

impl ColumnWidth {
    /// Minimum desired width; a single column may be narrower.
    pub const fn points(self) -> f32 {
        match self {
            Self::Compact => 160.0,
            Self::Standard => 240.0,
            Self::Card => 280.0,
            Self::Wide => 320.0,
            Self::Form => 336.0,
        }
    }
}

/// Viewport width at which default container gutters grow from 16 to 24 points.
pub const WIDE_GUTTER_BREAKPOINT: f32 = 768.0;
