/// Typeface role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Family {
    /// Space Grotesk-derived display face.
    Display,
    /// Inter-derived body/UI face.
    #[default]
    Body,
    /// JetBrains Mono-derived technical face.
    Mono,
}

/// Explicit static font weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Weight {
    /// 400.
    #[default]
    Regular,
    /// 500.
    Medium,
    /// 600.
    SemiBold,
    /// 700.
    Bold,
}

/// The complete 12–96-point type scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextSize {
    /// 12 points.
    Xs,
    /// 14 points.
    Sm,
    /// 16 points.
    #[default]
    Base,
    /// 18 points.
    Lg,
    /// 20 points.
    Xl,
    /// 24 points.
    Xl2,
    /// 30 points.
    Xl3,
    /// 36 points.
    Xl4,
    /// 48 points.
    Xl5,
    /// 60 points.
    Xl6,
    /// 72 points.
    Xl7,
    /// 96 points.
    Xl8,
}

impl TextSize {
    /// Font size in logical points.
    pub const fn points(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::Sm => 14.0,
            Self::Base => 16.0,
            Self::Lg => 18.0,
            Self::Xl => 20.0,
            Self::Xl2 => 24.0,
            Self::Xl3 => 30.0,
            Self::Xl4 => 36.0,
            Self::Xl5 => 48.0,
            Self::Xl6 => 60.0,
            Self::Xl7 => 72.0,
            Self::Xl8 => 96.0,
        }
    }

    /// Default line-height multiplier from the type-scale documentation.
    pub const fn line_height(self) -> f32 {
        match self {
            Self::Xs | Self::Xl => 1.3,
            Self::Sm | Self::Lg => 1.4,
            Self::Base => 1.5,
            Self::Xl2 => 1.2,
            Self::Xl3 => 1.1,
            Self::Xl4 | Self::Xl8 => 1.0,
            Self::Xl5 => 0.95,
            Self::Xl6 | Self::Xl7 => 0.9,
        }
    }
}

/// Letter spacing relative to font size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tracking {
    /// -0.05 em.
    Tighter,
    /// -0.025 em.
    Tight,
    /// Zero additional spacing.
    #[default]
    Normal,
    /// 0.025 em.
    Wide,
    /// 0.05 em.
    Wider,
    /// 0.1 em.
    Widest,
}

impl Tracking {
    /// Tracking in em units; multiply by font size for TextFormat.
    pub const fn em(self) -> f32 {
        match self {
            Self::Tighter => -0.05,
            Self::Tight => -0.025,
            Self::Normal => 0.0,
            Self::Wide => 0.025,
            Self::Wider => 0.05,
            Self::Widest => 0.1,
        }
    }
}

/// Named line-height multipliers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leading {
    /// 0.85, matching the live hero.
    Hero,
    /// 0.9, large display text.
    Display,
    /// 0.95, section headings.
    Heading,
    /// 1.0.
    None,
    /// 1.25.
    Tight,
    /// 1.375.
    Snug,
    /// 1.5.
    Normal,
    /// 1.625.
    Relaxed,
    /// 2.0.
    Loose,
}

impl Leading {
    /// Unitless multiplier.
    pub const fn factor(self) -> f32 {
        match self {
            Self::Hero => 0.85,
            Self::Display => 0.9,
            Self::Heading => 0.95,
            Self::None => 1.0,
            Self::Tight => 1.25,
            Self::Snug => 1.375,
            Self::Normal => 1.5,
            Self::Relaxed => 1.625,
            Self::Loose => 2.0,
        }
    }
}
