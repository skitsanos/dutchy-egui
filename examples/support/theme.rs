//! The accent, mode and reduced-motion switcher shared by the examples.

use dutchy_egui::{Accent, Button, ButtonVariant, Checkbox, ControlSize, Flex, Mode, Theme};
use egui::{Context, Ui};

/// Host-owned appearance settings. The theme is derived, never stored twice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeSettings {
    /// Brand accent.
    pub accent: Accent,
    /// Light or dark palette.
    pub mode: Mode,
    /// Static presentation of animated components.
    pub reduced_motion: bool,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            accent: Accent::Orange,
            mode: Mode::Light,
            reduced_motion: false,
        }
    }
}

impl ThemeSettings {
    /// The theme these settings describe.
    pub fn theme(self) -> Theme {
        Theme::new(self.accent, self.mode).with_reduced_motion(self.reduced_motion)
    }

    /// Install the theme on a context.
    pub fn apply(self, ctx: &Context) {
        self.theme().apply(ctx);
    }

    /// Short accent label for compact controls.
    pub fn accent_label(accent: Accent) -> &'static str {
        match accent {
            Accent::Orange => "Orange",
            Accent::Purple => "Purple",
            Accent::Crimson => "Crimson",
            Accent::Blue => "Blue",
            Accent::Green => "Green",
            _ => accent.name(),
        }
    }

    /// Accent buttons, a mode toggle and a reduced-motion checkbox in one
    /// wrapping row. Applies the theme when a setting changes and reports it.
    pub fn controls(&mut self, ui: &mut Ui) -> bool {
        let before = *self;
        Flex::row("theme-controls").wrap(true).show(ui, |ui| {
            for &accent in Accent::ALL {
                let variant = if accent == self.accent {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                };
                let button = Button::new(Self::accent_label(accent))
                    .variant(variant)
                    .size(ControlSize::Small);
                if ui.add(button).clicked() {
                    self.accent = accent;
                }
            }
            let mode_label = match self.mode {
                Mode::Light => "Dark mode",
                Mode::Dark => "Light mode",
            };
            let toggle = Button::new(mode_label)
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small);
            if ui.add(toggle).clicked() {
                self.mode = match self.mode {
                    Mode::Light => Mode::Dark,
                    Mode::Dark => Mode::Light,
                };
            }
            ui.add(Checkbox::new(
                "reduce-motion",
                &mut self.reduced_motion,
                "Reduce motion",
            ));
        });
        let changed = *self != before;
        if changed {
            self.apply(ui.ctx());
        }
        changed
    }
}
