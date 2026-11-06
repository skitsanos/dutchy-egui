//! Shared design tokens, expressed in egui logical points.

pub mod close;
mod color;
pub mod disclosure;
pub mod layout;
pub mod loading;
pub mod menu;
pub mod metrics;
pub mod navigation;
pub mod overlay;
pub mod stepper;
pub mod surface;
pub mod toast;
pub mod tooltip;
mod typography;

pub use color::{Accent, Mode, Palette};
pub use layout::{ColumnWidth, ContainerWidth, Spacing};
pub use loading::ProgressSize;
pub use menu::MenuWidth;
pub use overlay::{DrawerWidth, ModalWidth};
pub use surface::CardPadding;
pub use typography::{Family, Leading, TextSize, Tracking, Weight};

/// Four-point spacing rhythm, including the documented two-point micro gap.
pub mod space {
    /// Micro gap.
    pub const MICRO: f32 = 2.0;
    /// Base unit.
    pub const XS: f32 = 4.0;
    /// Tight gap, also used for tooltip vertical padding.
    pub const TIGHT: f32 = 6.0;
    /// Small gap.
    pub const SM: f32 = 8.0;
    /// Compact padding.
    pub const MD: f32 = 12.0;
    /// Standard gap.
    pub const LG: f32 = 16.0;
    /// Medium spacing.
    pub const XL: f32 = 20.0;
    /// Card padding.
    pub const XXL: f32 = 24.0;
    /// Section gap.
    pub const SECTION: f32 = 32.0;
    /// Large section gap.
    pub const LARGE: f32 = 40.0;
    /// Hero inset.
    pub const HERO: f32 = 48.0;
    /// Major section spacing.
    pub const MAJOR: f32 = 64.0;
    /// Page section spacing.
    pub const PAGE: f32 = 80.0;
    /// Large hero spacing.
    pub const HERO_LARGE: f32 = 96.0;
}

/// Border and focus geometry.
pub mod border {
    /// Subtle divider.
    pub const THIN: f32 = 1.0;
    /// Standard control border.
    pub const STANDARD: f32 = 2.0;
    /// Accent edge.
    pub const ACCENT: f32 = 4.0;
    /// Major divider.
    pub const HEAVY: f32 = 8.0;
    /// Focus outline width.
    pub const FOCUS_WIDTH: f32 = 2.0;
    /// Gap between the control and focus outline.
    pub const FOCUS_OFFSET: f32 = 2.0;
}

/// Shared motion timing in seconds.
pub mod motion {
    /// Frame interval shared by continuous loading animations.
    pub const FRAME_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);
    /// Fast color transition.
    pub const FAST: f32 = 0.15;
    /// Standard transition.
    pub const NORMAL: f32 = 0.2;
    /// Longer transition.
    pub const SLOW: f32 = 0.3;
    /// One square-spinner rotation.
    pub const SPIN: f64 = 0.6;
    /// Live web skeleton pulse duration.
    pub const SKELETON_PULSE: f64 = 2.0;
    /// One indeterminate bar traversal and return.
    pub const PROGRESS_TRAVEL: f64 = 1.5;
}

/// Form-control dimensions, shared across implementations.
pub mod control {
    /// Checkbox/radio square side.
    pub const INDICATOR: f32 = 20.0;
    /// Space between an indicator and its label.
    pub const LABEL_GAP: f32 = 12.0;
    /// Minimum multiline input height.
    pub const TEXTAREA_MIN_HEIGHT: f32 = 120.0;
    /// Default number of multiline rows.
    pub const TEXTAREA_ROWS: usize = 4;
    /// Maximum select popup height.
    pub const SELECT_POPUP_HEIGHT: f32 = 240.0;
}

/// Shared sizes for buttons and future input/select controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    /// Compact 32-point control.
    Small,
    /// Default 48-point control.
    #[default]
    Medium,
    /// Large 56-point control.
    Large,
}

impl ControlSize {
    /// Exact minimum control height.
    pub const fn height(self) -> f32 {
        match self {
            Self::Small => 32.0,
            Self::Medium => 48.0,
            Self::Large => 56.0,
        }
    }

    /// Horizontal button padding.
    pub const fn padding(self) -> f32 {
        match self {
            Self::Small => space::LG,
            Self::Medium => space::SECTION,
            Self::Large => space::LARGE,
        }
    }

    /// Button label scale.
    pub const fn text_size(self) -> TextSize {
        match self {
            Self::Small => TextSize::Sm,
            Self::Medium => TextSize::Base,
            Self::Large => TextSize::Lg,
        }
    }
}
