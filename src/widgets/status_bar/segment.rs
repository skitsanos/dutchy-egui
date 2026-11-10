//! Measuring, painting and describing one status segment.
use super::item::{Content, StatusItem, StatusTone};
use crate::{
    Kbd, ProgressSize, ProgressVariant, Text, TextColor, Theme, Tooltip,
    interaction::paint_focus_inside,
    tokens::{Palette, status_bar as tokens},
};
use egui::{
    Color32, Galley, Id, Rect, Response, Sense, Ui, UiBuilder, WidgetInfo,
    accesskit::{Live, Role},
    pos2, vec2,
};
use std::sync::Arc;

/// Laid-out parts of one segment, in painting order.
pub(super) struct Parts {
    lead: f32,
    label: Option<Arc<Galley>>,
    kbd: Option<f32>,
    text: Arc<Galley>,
    elided: bool,
}

impl Parts {
    /// Lay out an item; `budget` elides the text so the segment fits it.
    pub fn measure<K>(ui: &Ui, item: &StatusItem<'_, K>, budget: Option<f32>) -> Self {
        let lead = if item.icon.is_some() {
            tokens::ICON + tokens::PART_GAP
        } else if item.tone != StatusTone::Default {
            tokens::MARKER + tokens::PART_GAP
        } else {
            0.0
        };
        let label = match &item.content {
            Content::Labeled(label) => Some(galley(ui, label, TextColor::Muted, None)),
            _ => None,
        };
        let kbd = match &item.content {
            Content::Hint(chord) => Some(Kbd::new(chord.as_ref()).desired_size(ui).x),
            _ => None,
        };
        let mut parts = Self {
            lead,
            label,
            kbd,
            text: galley(ui, &item.text, text_color(item), None),
            elided: false,
        };
        if let Some(budget) = budget {
            let room = budget - (parts.width(item) - parts.text.size().x);
            parts.text = galley(ui, &item.text, text_color(item), Some(room.max(0.0)));
            parts.elided = true;
        }
        parts
    }

    /// Natural segment width including its padding.
    pub fn width<K>(&self, item: &StatusItem<'_, K>) -> f32 {
        let label = self
            .label
            .as_ref()
            .map_or(0.0, |g| g.size().x + tokens::PART_GAP);
        let kbd = self.kbd.map_or(0.0, |w| w + tokens::PART_GAP);
        let progress = if item.progress.is_some() {
            tokens::PART_GAP + tokens::PROGRESS_WIDTH
        } else {
            0.0
        };
        2.0 * tokens::PADDING + self.lead + label + kbd + self.text.size().x + progress
    }
}

fn text_color<K>(item: &StatusItem<'_, K>) -> TextColor {
    if matches!(item.content, Content::Hint(_)) {
        TextColor::Muted
    } else {
        TextColor::Foreground
    }
}

fn galley(ui: &Ui, text: &str, color: TextColor, width: Option<f32>) -> Arc<Galley> {
    let theme = Theme::from_context(ui.ctx());
    let mut job = Text::label(text)
        .color(color)
        .layout_job(&theme, f32::INFINITY);
    if let Some(width) = width {
        job.wrap.max_width = width;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
    }
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

fn tone_color(tone: StatusTone, p: &Palette) -> Color32 {
    match tone {
        StatusTone::Default => p.foreground,
        StatusTone::Primary => p.primary,
        StatusTone::Success => p.success,
        StatusTone::Warning => p.warning,
        StatusTone::Destructive => p.destructive,
        StatusTone::Info => p.info,
    }
}

/// Interact with, describe and paint one visible segment.
pub(super) fn show<K: std::hash::Hash>(
    ui: &mut Ui,
    id: Id,
    item: &StatusItem<'_, K>,
    parts: Parts,
    rect: Rect,
) -> Response {
    let mut segment = ui.new_child(UiBuilder::new().id_salt(id).max_rect(rect));
    if !item.enabled {
        segment.disable();
    }
    let sense = if item.clickable {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = segment.interact(rect, id, sense);
    describe(&response, item);
    let tooltip = item
        .tooltip
        .as_deref()
        .or(parts.elided.then_some(item.text.as_ref()));
    if let Some(tooltip) = tooltip {
        Tooltip::new(tooltip).show(&response);
    }
    paint(&segment, &response, item, parts);
    response
}

fn describe<K>(response: &Response, item: &StatusItem<'_, K>) {
    let role = if item.clickable {
        Role::Button
    } else if item.progress.is_some() {
        Role::ProgressIndicator
    } else if item.live {
        Role::Status
    } else {
        Role::Label
    };
    // AccessKit reads a Label's text from its value, so passive and live
    // segments speak their full text; buttons and progress keep label/value.
    let passive = matches!(role, Role::Label | Role::Status);
    let name = if passive {
        item.spoken()
    } else {
        item.name().to_owned()
    };
    response.widget_info(|| WidgetInfo::labeled(role, response.enabled(), &name));
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_label(name.clone());
        match &item.content {
            _ if passive => node.set_value(name.clone()),
            Content::Labeled(_) => node.set_value(item.text.as_ref()),
            Content::Text | Content::Hint(_) => {}
        }
        if let Content::Hint(chord) = &item.content {
            node.set_keyboard_shortcut(chord.as_ref());
        }
        match item.progress {
            Some(Some(percent)) if !item.clickable => {
                node.set_numeric_value(percent);
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(100.0);
            }
            Some(None) if !item.clickable => node.set_busy(),
            _ => {}
        }
        if item.live {
            node.set_live(Live::Polite);
        }
    });
}

fn paint<K>(ui: &Ui, response: &Response, item: &StatusItem<'_, K>, parts: Parts) {
    let rect = response.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    if item.clickable
        && response.enabled()
        && (response.hovered() || response.is_pointer_button_down_on())
    {
        response
            .clone()
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(rect, 0, p.muted);
    }
    let middle = rect.center().y;
    let mut x = rect.left() + tokens::PADDING;
    paint_lead(ui, item, pos2(x, middle), &p);
    x += parts.lead;
    if let Some(label) = parts.label {
        let width = label.size().x;
        ui.painter().galley(
            pos2(x, middle - label.size().y / 2.0),
            label,
            p.muted_foreground,
        );
        x += width + tokens::PART_GAP;
    }
    if let (Some(width), Content::Hint(chord)) = (parts.kbd, &item.content) {
        let chip = Kbd::new(chord.as_ref());
        let size = chip.desired_size(ui);
        chip.paint(
            ui,
            Rect::from_min_size(pos2(x, middle - size.y / 2.0), size),
        );
        x += width + tokens::PART_GAP;
    }
    let width = parts.text.size().x;
    ui.painter().galley(
        pos2(x, middle - parts.text.size().y / 2.0),
        parts.text,
        p.foreground,
    );
    x += width;
    if let Some(progress) = item.progress {
        let height = ProgressSize::Small.height();
        let track = Rect::from_min_size(
            pos2(x + tokens::PART_GAP, middle - height / 2.0),
            vec2(tokens::PROGRESS_WIDTH, height),
        );
        let fraction = progress.map(|percent| (percent / 100.0) as f32);
        super::super::progress::paint::track(ui, track, fraction, ProgressVariant::Primary, true);
    }
    paint_focus_inside(ui, response, p.primary);
}

/// Icon or tone marker, starting at `start` on the segment's middle line.
fn paint_lead<K>(ui: &Ui, item: &StatusItem<'_, K>, start: egui::Pos2, p: &Palette) {
    let tone = tone_color(item.tone, p);
    if let Some(icon) = item.icon {
        let side = vec2(tokens::ICON, tokens::ICON);
        let center = pos2(start.x + tokens::ICON / 2.0, start.y);
        icon.paint(ui.painter(), Rect::from_center_size(center, side), tone);
    } else if item.tone != StatusTone::Default {
        let side = vec2(tokens::MARKER, tokens::MARKER);
        let center = pos2(start.x + tokens::MARKER / 2.0, start.y);
        ui.painter()
            .rect_filled(Rect::from_center_size(center, side), 0, tone);
    }
}
