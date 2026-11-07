//! Image, gallery and lightbox geometry.
use super::{Accent, Mode, Palette, space};
/// Fallback width for unconstrained layouts.
pub const DEFAULT_WIDTH: f32 = 640.0;
/// Maximum requested image dimension.
pub const MAX_DIMENSION: f32 = 4096.0;
/// Fallback aspect while intrinsic dimensions are unknown.
pub const FALLBACK_ASPECT: f32 = 16.0 / 9.0;
/// Caption separation from the image.
pub const CAPTION_GAP: f32 = space::MD;
/// Expand control inset.
pub const EXPAND_INSET: f32 = space::MD;
/// Expand control target.
pub const EXPAND_SIZE: f32 = space::LARGE;
/// Expand glyph extent.
pub const EXPAND_ICON: f32 = 18.0;
/// Compact gallery expansion target.
pub const GALLERY_EXPAND_SIZE: f32 = super::ControlSize::Small.height();
/// Gallery expansion inset.
pub const GALLERY_EXPAND_INSET: f32 = space::SM;
/// Gallery expansion glyph extent.
pub const GALLERY_EXPAND_ICON: f32 = 14.0;
/// Inverse expand background opacity (web image `bg-foreground/80`).
pub const EXPAND_OPACITY: f32 = 0.8;
/// Gallery tile expand background opacity (web gallery `bg-foreground/70`).
pub const GALLERY_EXPAND_OPACITY: f32 = 0.7;
/// Lightbox clearance on each viewport edge.
pub const VIEWPORT_GUTTER: f32 = space::XXL;
/// Lightbox close/navigation targets.
pub const VIEWER_CONTROL: f32 = space::HERO;
/// Lightbox glyph extent.
pub const VIEWER_ICON: f32 = space::XXL;
/// Space reserved for ordinary captions below the lightbox stage.
pub const CAPTION_RESERVE: f32 = space::MAJOR;
/// Minimum preferred lightbox image stage height.
pub const MIN_STAGE: f32 = space::HERO_LARGE;
/// Source's consistently dark lightbox surface, independent of host mode.
pub const VIEWER_BACKGROUND: egui::Color32 = Palette::new(Accent::Orange, Mode::Light).foreground;
/// Text on the lightbox surface.
pub const VIEWER_FOREGROUND: egui::Color32 = Palette::new(Accent::Orange, Mode::Light).background;
/// Full-viewport backdrop opacity.
pub const BACKDROP_OPACITY: f32 = 0.95;

/// Preferred error-retry target width.
pub const RETRY_WIDTH: f32 = 160.0;
