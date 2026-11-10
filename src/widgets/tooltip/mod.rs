mod paint;

use crate::{Text, TextColor, TextSize, Theme, Weight, tokens::tooltip as geometry};
use egui::{Frame, InnerResponse, Margin, RectAlign, Response};
use std::borrow::Cow;

/// Preferred side of the anchor. Native egui placement may flip at screen edges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TooltipPosition {
    /// Above the anchor.
    #[default]
    Top,
    /// Below the anchor.
    Bottom,
    /// Left of the anchor.
    Left,
    /// Right of the anchor.
    Right,
}

impl TooltipPosition {
    fn align(self) -> RectAlign {
        match self {
            Self::Top => RectAlign::TOP,
            Self::Bottom => RectAlign::BOTTOM,
            Self::Left => RectAlign::LEFT,
            Self::Right => RectAlign::RIGHT,
        }
    }
}

/// Concise, noninteractive help on hover or keyboard focus.
///
/// The hint is also the anchor's accessible description while closed. Keep an
/// accessible name and essential instructions on the control itself. Escape
/// dismisses the hint until both hover and focus leave; it is not consumed.
/// Native egui Escape handling (including clearing button focus) still applies.
#[derive(Clone, Debug)]
#[must_use = "Call show with the control's response"]
pub struct Tooltip<'a> {
    text: Cow<'a, str>,
    position: TooltipPosition,
}

impl<'a> Tooltip<'a> {
    /// Inverted tooltip above the anchor, using native hover delay and grace.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            position: TooltipPosition::default(),
        }
    }
    /// Preferred placement, constrained to the viewport by egui.
    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }
    /// Attach help to a control without taking focus or changing its action.
    /// Disabled controls support pointer help but do not gain a keyboard stop.
    pub fn show(self, response: &Response) -> Option<InnerResponse<()>> {
        if self.text.is_empty() {
            return None;
        }
        self.describe(response);
        if !should_show(response) {
            return None;
        }
        let theme = Theme::from_context(&response.ctx);
        let width = geometry::MAX_WIDTH
            .min((response.ctx.content_rect().width() - geometry::VIEWPORT_INSET).max(0.0));
        let frame = Frame::NONE
            .fill(theme.palette.foreground)
            .inner_margin(Margin::symmetric(geometry::PADDING_X, geometry::PADDING_Y));
        let mut tooltip = egui::Tooltip::for_widget(response)
            .gap(geometry::GAP)
            .width(width)
            .accessible_name(self.text.to_string());
        tooltip.popup = tooltip.popup.align(self.position.align()).frame(frame);
        let result = tooltip.show(|ui| {
            ui.set_max_width(width);
            let mut text = Text::new(self.text)
                .size(TextSize::Xs)
                .weight(Weight::Medium)
                .color(TextColor::Background)
                .layout_job(&theme, width);
            for section in &mut text.sections {
                section.format.line_height = Some(TextSize::Xs.points() * geometry::LINE_HEIGHT);
                section.format.extra_letter_spacing = TextSize::Xs.points() * geometry::TRACKING_EM;
            }
            ui.add(egui::Label::new(text).wrap());
        });
        if let Some(result) = &result {
            paint::arrow(response, &result.response, theme.palette.foreground);
        }
        result
    }

    fn describe(&self, response: &Response) {
        response.ctx.accesskit_node_builder(response.id, |node| {
            let description = match node.description() {
                Some(existing) if existing != self.text => format!("{existing}\n{}", self.text),
                _ => self.text.to_string(),
            };
            node.set_description(description);
        });
    }
}

fn should_show(response: &Response) -> bool {
    let ctx = &response.ctx;
    let dismissal = response.id.with("dutchy-tooltip-dismissed");
    let engaged = response.has_focus() || response.contains_pointer();
    if !engaged {
        ctx.data_mut(|data| data.remove::<bool>(dismissal));
    }
    let dismissed = ctx.data(|data| data.get_temp::<bool>(dismissal).unwrap_or(false));
    let visible = response.interact_rect.is_positive()
        && ctx.content_rect().intersects(response.interact_rect);
    let requested =
        visible && (response.has_focus() || egui::Tooltip::should_show_tooltip(response, false));
    if requested && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        ctx.data_mut(|data| data.insert_temp(dismissal, true));
        return false;
    }
    requested && !dismissed
}
