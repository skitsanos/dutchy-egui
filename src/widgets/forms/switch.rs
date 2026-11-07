use super::style;
use crate::{
    ControlSize, Text, TextColor, TextSize, Theme,
    interaction::paint_focus,
    tokens::{border, control, motion},
};
use egui::accesskit::Role;
use egui::{
    Id, IdSalt, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget, WidgetInfo, pos2, vec2,
};
use std::borrow::Cow;

/// On/off control with a sliding square thumb and a wrapped, clickable label.
///
/// Space and Enter toggle the focused switch through egui's native activation.
#[must_use = "Add this switch to a Ui"]
pub struct Switch<'a> {
    id: IdSalt,
    on: &'a mut bool,
    label: Cow<'a, str>,
    enabled: bool,
}

impl<'a> Switch<'a> {
    /// Bind a stable ID, host value, and accessible visible label.
    pub fn new(id: impl egui::AsIdSalt, on: &'a mut bool, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            on,
            label: label.into(),
            enabled: true,
        }
    }
    /// Disable interaction, also honoring a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id);
        ui.add_enabled(self.enabled, |ui: &mut Ui| {
            show(ui, id, &self.label, self.on)
        })
    }
}

fn show(ui: &mut Ui, id: Id, label: &str, on: &mut bool) -> Response {
    let theme = Theme::from_context(ui.ctx());
    let width = control::SWITCH_WIDTH;
    let gap = control::LABEL_GAP;
    let text_width = (ui.available_width() - width - gap).max(0.0);
    // Inverse surfaces install a scoped text color; the label follows it.
    let color = ui
        .visuals()
        .override_text_color
        .map_or(TextColor::Foreground, TextColor::Custom);
    let job = Text::new(label)
        .size(TextSize::Sm)
        .color(color)
        .layout_job(&theme, text_width);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let size = vec2(
        (width + gap + galley.size().x).min(ui.available_width()),
        ControlSize::Small.height().max(galley.size().y),
    );
    let (_, rect) = ui.allocate_space(size);
    let mut response = ui.interact(rect, id, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
        response.request_focus();
    }
    response.widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), *on, label));
    if ui.is_rect_visible(rect) {
        let track = Rect::from_min_size(
            pos2(rect.left(), rect.center().y - control::INDICATOR / 2.0),
            vec2(width, control::INDICATOR),
        );
        paint_track(ui, id, track, *on, response.hovered());
        ui.painter().galley(
            pos2(track.right() + gap, rect.center().y - galley.size().y / 2.0),
            galley,
            theme.palette.foreground,
        );
        let mut focus = response.clone();
        focus.rect = track;
        paint_focus(ui, &focus, theme.palette.primary);
    }
    response
}

fn paint_track(ui: &Ui, id: Id, track: Rect, on: bool, hovered: bool) {
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    let progress = if theme.reduced_motion {
        f32::from(u8::from(on))
    } else {
        ui.ctx()
            .animate_bool_with_time(id.with("thumb"), on, motion::FAST)
    };
    let edge = if on || hovered {
        p.primary
    } else {
        style::inherited_border(ui)
    };
    let fill = if on { p.primary } else { p.background };
    ui.painter().rect(
        track,
        0,
        fill,
        Stroke::new(border::STANDARD, edge),
        StrokeKind::Inside,
    );
    let inset = (control::INDICATOR - control::SWITCH_THUMB) / 2.0;
    let travel = track.width() - control::SWITCH_THUMB - 2.0 * inset;
    let thumb = Rect::from_min_size(
        pos2(
            track.left() + inset + travel * progress,
            track.top() + inset,
        ),
        vec2(control::SWITCH_THUMB, control::SWITCH_THUMB),
    );
    let thumb_color = if on { p.on_primary } else { p.muted_foreground };
    ui.painter().rect_filled(thumb, 0, thumb_color);
}
