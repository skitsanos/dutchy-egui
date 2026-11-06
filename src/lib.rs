//! Dutchy Design System for egui.
//!
//! Install the theme and bundled typefaces once, then compose normal egui UIs
//! with Dutchy widgets. Applications own their state.

pub use egui;

pub mod fonts;
pub mod interaction;
pub mod text;
pub mod theme;
pub mod tokens;
pub mod validation;
pub mod widgets;

pub use text::{Text, TextColor};
pub use theme::Theme;
pub use tokens::{Accent, ControlSize, Family, Leading, Mode, TextSize, Tracking, Weight};
pub use widgets::forms::{Checkbox, Field, Input, RadioGroup, Select, SelectOption, Textarea};
pub use widgets::{Badge, BadgeVariant, Button, ButtonVariant, Spinner, SpinnerSize};

/// Install bundled fonts once and apply a theme. Does not change display scaling.
///
/// ```
/// let ctx = egui::Context::default();
/// dutchy_egui::install(&ctx, dutchy_egui::Theme::default());
/// ```
pub fn install(ctx: &egui::Context, theme: Theme) {
    fonts::install(ctx);
    theme.apply(ctx);
}
