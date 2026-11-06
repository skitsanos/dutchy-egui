mod icon;
mod layout;

use crate::{
    Text, TextColor, TextSize, Tracking,
    tokens::{metrics, space},
};
use egui::accesskit::{Live, Role};
use egui::{Frame, IdSalt, Margin, Response, Ui, UiBuilder};
use std::borrow::Cow;

/// Empty-state layout, with no application behavior attached.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyStateVariant {
    /// Standard centered invitation.
    #[default]
    Generic,
    /// Search/filter guidance with larger vertical spacing.
    Search,
    /// Compact body for a host-supplied table; no fabricated table header.
    Table,
    /// Dashed dashboard setup surface.
    Dashboard,
}

/// Built-in decorative square iconography.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyStateIcon {
    /// No icon.
    #[default]
    None,
    /// Sharp search symbol.
    Search,
    /// Empty tray.
    Inbox,
    /// Four square widgets.
    Dashboard,
}

/// Independent action results. The host controls content, navigation, and focus.
pub struct EmptyStateResponse {
    /// Non-focusable container response.
    pub response: Response,
    /// Primary action was activated this frame.
    pub action_clicked: bool,
    /// Secondary action was activated this frame.
    pub secondary_clicked: bool,
}

/// Centered guidance with optional actions and explicit live announcements.
#[must_use = "Call show to render the empty state"]
pub struct EmptyState<'a> {
    id: IdSalt,
    title: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    variant: EmptyStateVariant,
    icon: Option<EmptyStateIcon>,
    action: Option<Cow<'a, str>>,
    secondary: Option<Cow<'a, str>>,
    enabled: bool,
    action_enabled: bool,
    secondary_enabled: bool,
    announce: bool,
}

impl<'a> EmptyState<'a> {
    /// Stable, titled generic empty state. Generic has no icon by default.
    pub fn new(id: impl egui::AsIdSalt, title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            title: title.into(),
            description: None,
            variant: EmptyStateVariant::Generic,
            icon: None,
            action: None,
            secondary: None,
            enabled: true,
            action_enabled: true,
            secondary_enabled: true,
            announce: false,
        }
    }
    /// Layout; Search/Table/Dashboard choose an appropriate default icon.
    pub fn variant(mut self, variant: EmptyStateVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Override the decorative icon, including explicitly removing it.
    pub fn icon(mut self, icon: EmptyStateIcon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Guidance below the heading.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Primary action label; the host inspects action_clicked.
    pub fn action(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.action = Some(label.into());
        self
    }
    /// Secondary action label; the host inspects secondary_clicked.
    pub fn secondary_action(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.secondary = Some(label.into());
        self
    }
    /// Disable all controls without enabling disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Enable/disable the primary action independently.
    pub fn action_enabled(mut self, enabled: bool) -> Self {
        self.action_enabled = enabled;
        self
    }
    /// Enable/disable the secondary action independently.
    pub fn secondary_action_enabled(mut self, enabled: bool) -> Self {
        self.secondary_enabled = enabled;
        self
    }
    /// Announce replacement of expected content politely; off for static content.
    pub fn announce(mut self, announce: bool) -> Self {
        self.announce = announce;
        self
    }

    /// Render without moving focus or performing application actions.
    pub fn show(self, ui: &mut Ui) -> EmptyStateResponse {
        let result = ui.scope_builder(
            UiBuilder::new()
                .scope_id(ui.make_persistent_id(("dutchy-empty-state", self.id)))
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                let (vertical, horizontal) = match self.variant {
                    EmptyStateVariant::Search => (space::PAGE, space::XXL),
                    EmptyStateVariant::Dashboard => (space::HERO, space::HERO),
                    _ => (space::MAJOR, space::XXL),
                };
                let available = ui.available_width().max(0.0);
                let margin =
                    Margin::symmetric(horizontal.min(available / 4.0) as i8, vertical as i8);
                let body = Frame::NONE.inner_margin(margin).show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| self.body(ui)).inner
                });
                if self.variant == EmptyStateVariant::Dashboard {
                    layout::dashboard_border(ui, body.response.rect);
                }
                body.inner
            },
        );
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| {
                node.set_role(if self.announce {
                    Role::Status
                } else {
                    Role::Group
                });
                node.set_label(self.title.to_string());
                if let Some(description) = &self.description {
                    node.set_description(description.to_string());
                }
                node.set_live(if self.announce {
                    Live::Polite
                } else {
                    Live::Off
                });
            });
        EmptyStateResponse {
            response: result.response,
            action_clicked: result.inner.0,
            secondary_clicked: result.inner.1,
        }
    }

    fn body(&self, ui: &mut Ui) -> (bool, bool) {
        let (default_icon, side, title_size) = match self.variant {
            EmptyStateVariant::Generic => (EmptyStateIcon::None, metrics::EMPTY_ICON, TextSize::Xl),
            EmptyStateVariant::Search => {
                (EmptyStateIcon::Search, metrics::EMPTY_ICON, TextSize::Xl)
            }
            EmptyStateVariant::Table => (EmptyStateIcon::Inbox, metrics::TABLE_ICON, TextSize::Sm),
            EmptyStateVariant::Dashboard => (
                EmptyStateIcon::Dashboard,
                metrics::DASHBOARD_ICON,
                TextSize::Xl2,
            ),
        };
        icon::show(
            ui,
            self.icon.unwrap_or(default_icon),
            self.variant,
            side.min(ui.available_width()),
        );
        let width = layout::text_width(ui, self.variant);
        let title = layout::text(
            ui,
            Text::heading(self.title.as_ref())
                .size(title_size)
                .tracking(Tracking::Tight),
            width,
        );
        title.ctx.accesskit_node_builder(title.id, |node| {
            node.set_role(Role::Heading);
            node.set_label(self.title.to_uppercase());
            node.set_level(3);
        });
        if let Some(description) = &self.description {
            layout::text(
                ui,
                Text::new(description.as_ref())
                    .size(if self.variant == EmptyStateVariant::Dashboard {
                        TextSize::Base
                    } else {
                        TextSize::Sm
                    })
                    .color(TextColor::Muted),
                width,
            );
        }
        layout::actions(ui, self)
    }
}
