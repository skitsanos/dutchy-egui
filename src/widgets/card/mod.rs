mod style;

use crate::{
    CardPadding, Leading, Text, TextColor, TextSize, Theme,
    interaction::paint_focus,
    tokens::{border, space},
};
use egui::{InnerResponse, Response, Sense, Ui, UiBuilder, WidgetInfo};

/// Card surface treatments from the Dutchy reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    /// Plain background.
    #[default]
    Default,
    /// Four-point brand accent along the left edge.
    Accent,
    /// Two-point border on all sides.
    Bordered,
    /// Muted surface.
    Muted,
    /// Inverted foreground surface (light in a dark theme).
    Dark,
    /// Primary surface with on-primary text.
    Primary,
}

impl CardVariant {
    /// All supported surfaces.
    pub const ALL: [Self; 6] = [
        Self::Default,
        Self::Accent,
        Self::Bordered,
        Self::Muted,
        Self::Dark,
        Self::Primary,
    ];

    /// Human-readable variant name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Accent => "Accent",
            Self::Bordered => "Bordered",
            Self::Muted => "Muted",
            Self::Dark => "Dark",
            Self::Primary => "Primary",
        }
    }
}

/// Full-width composable surface. Durable state stays with the application.
#[derive(Clone, Copy, Debug)]
#[must_use = "Call show or show_interactive to render the card"]
pub struct Card {
    variant: CardVariant,
    padding: CardPadding,
    enabled: bool,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            variant: CardVariant::Default,
            padding: CardPadding::Medium,
            enabled: true,
        }
    }
}

impl Card {
    /// Plain surface with medium padding.
    pub fn new() -> Self {
        Self::default()
    }
    /// Surface treatment.
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Shared padding tier.
    pub fn padding(mut self, padding: CardPadding) -> Self {
        self.padding = padding;
        self
    }
    /// Disable this card and its descendants without enabling a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render a non-focusable container, allowing normal child controls.
    pub fn show<R>(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.render(ui, UiBuilder::new(), false, contents)
    }
    /// Render an accessible action surface with a stable ID and explicit name.
    ///
    /// Enter/Space activate the returned response. Child controls receive their
    /// own clicks instead of activating the card. The host performs any action.
    pub fn show_interactive<R>(
        self,
        ui: &mut Ui,
        id: impl egui::AsIdSalt,
        name: &str,
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let mut result = self.render(
            ui,
            UiBuilder::new()
                .scope_id(ui.make_persistent_id(("dutchy-card", egui::IdSalt::new(id))))
                .sense(Sense::click()),
            true,
            contents,
        );
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Button,
                result.response.enabled(),
                name,
            )
        });
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| node.set_label(name));
        result.response = result
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        result
    }

    fn render<R>(
        self,
        ui: &mut Ui,
        builder: UiBuilder,
        interactive: bool,
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let result = ui.scope_builder(builder, |ui| {
            if !self.enabled {
                ui.disable();
            }
            if interactive {
                // Display text belongs to the card action; explicit child
                // controls keep their own interaction and take precedence.
                ui.style_mut().interaction.selectable_labels = false;
            }
            let theme = Theme::from_context(ui.ctx());
            let hovered = interactive && ui.is_enabled() && ui.response().contains_pointer();
            let surface = style::surface(theme, self.variant, hovered);
            let result = style::frame(&surface, self.variant, self.padding).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                style::content(ui, &surface);
                contents(ui)
            });
            if self.variant == CardVariant::Accent {
                let mut edge = result.response.rect;
                edge.max.x = edge.min.x + border::ACCENT;
                ui.painter().rect_filled(edge, 0, surface.edge);
            }
            result.inner
        });
        if interactive {
            paint_focus(
                ui,
                &result.response,
                Theme::from_context(ui.ctx()).palette.primary,
            );
        }
        result
    }

    /// Display-font card heading, with heading level 3 accessibility metadata.
    pub fn title(ui: &mut Ui, title: &str) -> Response {
        // The live CardTitle keeps the display face without heading tracking.
        let response = ui.add(
            Text::heading(title)
                .size(TextSize::Xl)
                .tracking(crate::Tracking::Normal),
        );
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Heading);
            node.set_label(title);
            node.set_level(3);
        });
        response
    }
    /// Relaxed small description; inherits the surface's text color.
    pub fn description(ui: &mut Ui, text: &str) -> Response {
        ui.add(
            Text::new(text)
                .size(TextSize::Sm)
                .leading(Leading::Relaxed)
                .color(TextColor::Muted),
        )
    }
    /// Header content followed by the documented gap.
    pub fn header<R>(ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> R {
        let result = contents(ui);
        crate::layout::gap_after(ui, space::LG);
        result
    }
    /// Footer content separated by spacing and a one-point border divider
    /// (live `mt-6 pt-4 border-t border-border`, on every surface).
    pub fn footer<R>(ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> R {
        crate::layout::gap_after(ui, space::XXL);
        let (_, rect) = ui.allocate_space(egui::vec2(
            ui.available_width(),
            crate::tokens::border::THIN,
        ));
        ui.painter()
            .rect_filled(rect, 0, Theme::from_context(ui.ctx()).palette.border);
        crate::layout::gap_after(ui, space::LG);
        contents(ui)
    }
}
