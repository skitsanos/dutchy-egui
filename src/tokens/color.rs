use egui::Color32;

/// Brand accent, independent of light/dark mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    /// Dutch Orange, HSL(25, 95%, 53%).
    #[default]
    Orange,
    /// Royal Purple, HSL(263, 70%, 58%).
    Purple,
    /// Crimson Red, HSL(0, 84%, 50%).
    Crimson,
    /// Delft Blue, HSL(221, 83%, 53%). A port addition; see `docs/reference.md`.
    Blue,
    /// Emerald Green, HSL(160, 84%, 31%). A port addition; see `docs/reference.md`.
    Green,
}

impl Accent {
    /// Every preset in documented order, for theme pickers.
    pub const ALL: [Self; 5] = [
        Self::Orange,
        Self::Purple,
        Self::Crimson,
        Self::Blue,
        Self::Green,
    ];

    /// Preset's sRGB color, rounded from the canonical HSL token.
    pub const fn color(self) -> Color32 {
        match self {
            Self::Orange => Color32::from_rgb(249, 116, 21),
            Self::Purple => Color32::from_rgb(130, 73, 223),
            Self::Crimson => Color32::from_rgb(235, 20, 20),
            Self::Blue => Color32::from_rgb(36, 99, 235),
            Self::Green => Color32::from_rgb(13, 145, 101),
        }
    }

    /// Human-readable preset name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Orange => "Dutch Orange",
            Self::Purple => "Royal Purple",
            Self::Crimson => "Crimson Red",
            Self::Blue => "Delft Blue",
            Self::Green => "Emerald Green",
        }
    }
}

/// Neutral color scheme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Current web implementation's warm neutrals.
    #[default]
    Light,
    /// Dark neutrals documented by the web design system.
    Dark,
}

/// Semantic colors. Public fields allow application-specific palette overrides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Surface background.
    pub background: Color32,
    /// Main text and dark blocks.
    pub foreground: Color32,
    /// Dividers and control borders.
    pub border: Color32,
    /// Brand accent.
    pub primary: Color32,
    /// Text on the brand accent.
    pub on_primary: Color32,
    /// Secondary surface.
    pub secondary: Color32,
    /// Secondary surface text.
    pub on_secondary: Color32,
    /// Subtle surface.
    pub muted: Color32,
    /// Secondary text.
    pub muted_foreground: Color32,
    /// Destructive/error fill.
    pub destructive: Color32,
    /// Successful status fill.
    pub success: Color32,
    /// Warning status fill.
    pub warning: Color32,
    /// Informational status fill.
    pub info: Color32,
    /// Text on semantic status fills, matching the web reference.
    pub on_status: Color32,
}

impl Palette {
    /// Resolve a documented preset. See `docs/reference.md` for source differences.
    pub const fn new(accent: Accent, mode: Mode) -> Self {
        let mut palette = Self {
            background: Color32::from_rgb(250, 248, 245),
            foreground: Color32::from_rgb(18, 15, 12),
            border: Color32::from_rgb(231, 223, 218),
            primary: accent.color(),
            on_primary: Color32::WHITE,
            secondary: Color32::from_rgb(241, 234, 228),
            on_secondary: Color32::from_rgb(48, 37, 29),
            muted: Color32::from_rgb(243, 241, 237),
            muted_foreground: Color32::from_rgb(112, 100, 92),
            destructive: Color32::from_rgb(239, 67, 67),
            success: Color32::from_rgb(22, 162, 73),
            warning: Color32::from_rgb(245, 159, 10),
            info: Color32::from_rgb(9, 115, 220),
            on_status: Color32::WHITE,
        };
        if matches!(mode, Mode::Dark) {
            palette.background = Color32::from_rgb(10, 10, 10);
            palette.foreground = Color32::from_rgb(250, 250, 250);
            palette.border = Color32::from_rgb(51, 51, 51);
            palette.muted = Color32::from_rgb(38, 38, 38);
            palette.muted_foreground = Color32::from_rgb(163, 163, 163);
            palette.secondary = palette.muted;
            palette.on_secondary = palette.foreground;
        }
        palette
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new(Accent::Orange, Mode::Light)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_listed_once_with_distinct_colors_and_names() {
        assert_eq!(
            Accent::ALL,
            [
                Accent::Orange,
                Accent::Purple,
                Accent::Crimson,
                Accent::Blue,
                Accent::Green
            ]
        );
        for (index, accent) in Accent::ALL.iter().enumerate() {
            for other in &Accent::ALL[index + 1..] {
                assert_ne!(accent.color(), other.color(), "{accent:?} vs {other:?}");
                assert_ne!(accent.name(), other.name(), "{accent:?} vs {other:?}");
            }
        }
    }

    #[test]
    fn blue_and_green_round_their_hsl_tokens() {
        assert_eq!(Accent::Blue.color(), Color32::from_rgb(36, 99, 235));
        assert_eq!(Accent::Blue.name(), "Delft Blue");
        assert_eq!(Accent::Green.color(), Color32::from_rgb(13, 145, 101));
        assert_eq!(Accent::Green.name(), "Emerald Green");
    }

    #[test]
    fn new_accents_drive_the_primary_token_in_both_modes() {
        for mode in [Mode::Light, Mode::Dark] {
            assert_eq!(
                Palette::new(Accent::Blue, mode).primary,
                Accent::Blue.color()
            );
            assert_eq!(
                Palette::new(Accent::Green, mode).primary,
                Accent::Green.color()
            );
            assert_eq!(Palette::new(Accent::Green, mode).on_primary, Color32::WHITE);
        }
    }
}
