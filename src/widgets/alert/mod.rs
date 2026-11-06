pub(super) mod icon;

use crate::{
    Button, ButtonVariant, ControlSize, Text, TextSize, Theme, Weight,
    tokens::{border, space, surface},
};
use egui::accesskit::{Live, Role};
use egui::{Color32, Frame, Id, IdSalt, Margin, Response, Ui};
use std::borrow::Cow;

/// Notification intent, expressed through a symbol, name, and color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    /// Brand-accent information; polite announcement.
    #[default]
    Info,
    /// Successful outcome; polite announcement.
    Success,
    /// Attention required; assertive announcement.
    Warning,
    /// Failed operation; assertive announcement.
    Error,
}

impl AlertVariant {
    /// All supported intents.
    pub const ALL: [Self; 4] = [Self::Info, Self::Success, Self::Warning, Self::Error];
    /// Human-readable severity.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "Information",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
    fn color(self, theme: Theme) -> Color32 {
        let p = theme.palette;
        match self {
            Self::Info => p.primary,
            Self::Success => p.success,
            Self::Warning => p.warning,
            Self::Error => p.destructive,
        }
    }
}

/// Separate action results; the application owns removal and action execution.
pub struct AlertResponse {
    /// Container response; clicking the message does not activate its controls.
    pub response: Response,
    /// The dismiss control was activated this frame.
    pub dismissed: bool,
    /// The optional action was activated this frame.
    pub action_clicked: bool,
}

/// Tinted alert or solid banner. Stable IDs keep child controls stable.
#[must_use = "Call show to render the alert"]
pub struct Alert<'a> {
    id: IdSalt,
    message: Cow<'a, str>,
    title: Option<Cow<'a, str>>,
    variant: AlertVariant,
    banner: bool,
    dismissible: bool,
    action: Option<Cow<'a, str>>,
    enabled: bool,
    announce: bool,
    return_focus: Option<Id>,
}

impl<'a> Alert<'a> {
    /// Informational alert, with a default severity title.
    pub fn new(id: impl egui::AsIdSalt, message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            message: message.into(),
            title: None,
            variant: AlertVariant::Info,
            banner: false,
            dismissible: false,
            action: None,
            enabled: true,
            announce: true,
            return_focus: None,
        }
    }
    /// Notification intent.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Replace the visible severity title. The accessible name retains severity.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// Solid full-width surface instead of a tinted left-accent alert.
    pub fn banner(mut self, banner: bool) -> Self {
        self.banner = banner;
        self
    }
    /// Show a separately named dismiss control.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
    /// Show an action; inspect action_clicked to execute it in the host.
    pub fn action(mut self, action: impl Into<Cow<'a, str>>) -> Self {
        self.action = Some(action.into());
        self
    }
    /// Disable controls, preserving disabled-parent behavior.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Disable live announcements for static or historical messages.
    pub fn announce(mut self, announce: bool) -> Self {
        self.announce = announce;
        self
    }
    /// Explicit focus destination after dismissal. The host must keep it present.
    pub fn return_focus_to(mut self, id: Id) -> Self {
        self.return_focus = Some(id);
        self
    }

    /// Render without stealing focus when the message appears or changes.
    pub fn show(self, ui: &mut Ui) -> AlertResponse {
        let theme = Theme::from_context(ui.ctx());
        let accent = self.variant.color(theme);
        let (fill, text) = if self.banner {
            (
                accent,
                if self.variant == AlertVariant::Info {
                    theme.palette.on_primary
                } else {
                    theme.palette.on_status
                },
            )
        } else {
            (
                theme
                    .palette
                    .background
                    .lerp_to_gamma(accent, surface::STATUS_TINT),
                theme.palette.foreground,
            )
        };
        let result = ui.scope_builder(
            egui::UiBuilder::new().scope_id(ui.make_persistent_id(("dutchy-alert", self.id))),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                ui.visuals_mut().override_text_color = Some(text);
                self.body(ui, fill, text, accent)
            },
        );
        let (dismissed, action_clicked) = result.inner;
        self.metadata(&result.response);
        if dismissed && let Some(id) = self.return_focus {
            ui.memory_mut(|memory| memory.request_focus(id));
        }
        AlertResponse {
            response: result.response,
            dismissed,
            action_clicked,
        }
    }

    fn body(&self, ui: &mut Ui, fill: Color32, text: Color32, accent: Color32) -> (bool, bool) {
        let mut margin = Margin::same(surface::ALERT_PADDING as i8);
        if !self.banner {
            margin.left += border::ACCENT as i8;
        }
        let result = Frame::NONE.fill(fill).inner_margin(margin).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.x = space::MD;
            let close_width = if self.dismissible {
                ControlSize::Small.height() + space::MD
            } else {
                0.0
            };
            let width =
                (ui.available_width() - surface::STATUS_ICON - space::MD - close_width).max(0.0);
            let mut dismissed = false;
            let mut action = false;
            ui.horizontal_top(|ui| {
                icon::status(ui, self.variant, if self.banner { text } else { accent });
                ui.vertical(|ui| {
                    ui.set_width(width);
                    ui.add(
                        Text::new(self.title.as_deref().unwrap_or(self.variant.name()))
                            .size(TextSize::Sm)
                            .weight(Weight::Bold),
                    );
                    ui.add(Text::new(self.message.as_ref()).size(TextSize::Sm));
                    if let Some(label) = &self.action {
                        ui.add_space(space::SM);
                        action = ui
                            .add(
                                Button::new(label.as_ref())
                                    .variant(if self.banner {
                                        ButtonVariant::Secondary
                                    } else {
                                        ButtonVariant::Outline
                                    })
                                    .size(ControlSize::Small),
                            )
                            .clicked();
                    }
                });
                if self.dismissible {
                    dismissed = icon::dismiss(
                        ui,
                        &format!(
                            "Dismiss {}",
                            self.title.as_deref().unwrap_or(self.variant.name())
                        ),
                        text,
                    )
                    .clicked();
                }
            });
            (dismissed, action)
        });
        if !self.banner {
            let mut edge = result.response.rect;
            edge.max.x = edge.min.x + border::ACCENT;
            ui.painter().rect_filled(edge, 0, accent);
        }
        result.inner
    }

    fn metadata(&self, response: &Response) {
        response.ctx.accesskit_node_builder(response.id, |node| {
            let (role, live) = if !self.announce {
                (Role::Group, Live::Off)
            } else if matches!(self.variant, AlertVariant::Warning | AlertVariant::Error) {
                (Role::Alert, Live::Assertive)
            } else {
                (Role::Status, Live::Polite)
            };
            node.set_role(role);
            node.set_live(live);
            let title = self
                .title
                .as_deref()
                .map_or(String::new(), |title| format!(": {title}"));
            node.set_label(format!("{}{title}", self.variant.name()));
            node.set_description(self.message.to_string());
        });
    }
}
