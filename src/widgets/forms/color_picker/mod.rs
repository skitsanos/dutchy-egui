mod editor;
mod paint;
mod state;

use crate::{
    ControlSize, Theme,
    tokens::{border, color_picker as geometry, space},
};
use egui::accesskit::{Color, HasPopup, Role};
use egui::{
    Color32, Frame, Id, IdSalt, Key, Margin, Modifiers, Response, Sense, Stroke, Ui, UiBuilder,
    Widget, WidgetInfo, vec2,
};
use state::{State, format_hex};
use std::borrow::Cow;

/// A named, application-supplied color in a picker's palette.
#[derive(Clone, Debug)]
pub struct ColorPreset<'a> {
    label: Cow<'a, str>,
    color: Color32,
}

impl<'a> ColorPreset<'a> {
    /// A descriptive name and color. Use distinct names within one palette.
    pub fn new(label: impl Into<Cow<'a, str>>, color: Color32) -> Self {
        Self {
            label: label.into(),
            color,
        }
    }
}

/// Flat palette, hex and channel editing over an application-owned sRGBA color.
///
/// Changes commit immediately. Escape and Done close the popup without reverting
/// committed edits. Invalid hex text never changes the value and resets on blur.
/// RGB-only mode preserves the existing alpha; `.alpha(true)` exposes alpha edits.
/// Temporary straight RGB is retained at zero alpha, where Color32 cannot store it.
#[must_use = "Add the color picker to a Ui"]
pub struct ColorPicker<'a> {
    id: IdSalt,
    value: &'a mut Color32,
    name: Cow<'a, str>,
    palette: Option<&'a [ColorPreset<'a>]>,
    size: ControlSize,
    width: Option<f32>,
    alpha: bool,
    enabled: bool,
    read_only: bool,
}

impl<'a> ColorPicker<'a> {
    /// Bind a stable ID and host color. The default palette follows the theme.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut Color32) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            name: Cow::Borrowed("Color"),
            palette: None,
            size: ControlSize::Medium,
            width: None,
            alpha: false,
            enabled: true,
            read_only: false,
        }
    }
    /// Name for standalone use and the popup; Field may label the trigger.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Replace the theme palette with named host colors; an empty slice hides it.
    pub fn palette(mut self, palette: &'a [ColorPreset<'a>]) -> Self {
        self.palette = Some(palette);
        self
    }
    /// Show alpha controls and RGBA hex. RGB hex in this mode means opaque.
    pub fn alpha(mut self, alpha: bool) -> Self {
        self.alpha = alpha;
        self
    }
    /// Shared Small/Medium/Large trigger height.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Preferred trigger width, constrained to the available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }
    /// Disable interaction and close an open editor, honoring disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Allow inspection and copying; block palette and channel edits.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    fn popup(&self, ui: &mut Ui, id: Id, trigger: &Response, state: &mut State) {
        if !state.open {
            return;
        }
        if state
            .layer
            .is_some_and(|layer| !ui.memory(|m| m.allows_interaction(layer)))
        {
            state.open = false;
            return;
        }
        if owns_focus(ui, state, trigger)
            && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
        {
            state.open = false;
            trigger.request_focus();
            return;
        }
        let p = Theme::from_context(ui.ctx()).palette;
        let viewport = ui.ctx().content_rect();
        let width = geometry::POPUP_WIDTH.min((viewport.width() - 2.0 * space::XS).max(0.0));
        let frame = Frame::NONE
            .fill(p.background)
            .stroke(Stroke::new(border::STANDARD, p.border))
            .inner_margin(Margin::same(space::LG as i8));
        let mut open = state.open;
        let done = egui::Popup::from_response(trigger)
            .id(id.with("popup"))
            .open_bool(&mut open)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .align(egui::RectAlign::BOTTOM_START)
            .gap(space::XS)
            .width(width)
            .frame(frame)
            .accessible_name(format!("Edit {}", self.name))
            .show(|ui| {
                ui.set_width((width - 2.0 * (space::LG + border::STANDARD)).max(0.0));
                let height =
                    geometry::POPUP_MAX_HEIGHT.min((viewport.height() - 2.0 * space::XXL).max(0.0));
                // Areas initially remember a 400-point height. Give the scrolling
                // editor its viewport budget so alpha/footer need not be clipped.
                ui.set_max_height(height);
                egui::ScrollArea::vertical()
                    .id_salt(id.with("scroll"))
                    .max_height(height)
                    .show(ui, |ui| editor::show(self, ui, id, state))
                    .inner
            })
            .is_some_and(|shown| shown.inner);
        state.open = open && !done;
        if done {
            trigger.request_focus();
        }
    }
}

impl Widget for ColorPicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id);
        let state_id = id.with("color-editor");
        let mut state = ui
            .ctx()
            .data(|data| data.get_temp::<State>(state_id))
            .unwrap_or_else(|| State::new(*self.value, self.alpha));
        state.sync(*self.value, self.alpha);
        let before = state.rgba;
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());
        let (rect, _) = ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());
        let mut child = ui.new_child(UiBuilder::new().scope_id(id.with("control")).max_rect(rect));
        if !self.enabled {
            child.disable();
        }
        let ui = &mut child;
        let mut trigger = ui.interact(rect, id, Sense::click());
        trigger.widget_info(|| {
            WidgetInfo::labeled(Role::ColorWell, ui.is_enabled(), self.name.as_ref())
        });
        if !ui.is_enabled() {
            state.open = false;
        }
        if trigger.clicked() {
            state.open = !state.open;
            state.pending_focus = state.open;
            state.hex = format_hex(state.rgba, self.alpha);
            trigger.request_focus();
        }
        self.popup(ui, id, &trigger, &mut state);
        let next = state.color();
        if before != state.rgba && next != *self.value {
            *self.value = next;
            trigger.mark_changed();
            ui.ctx().request_repaint();
        }
        state.last = *self.value;
        if !state.open {
            state.pending_focus = false;
            state.hex = format_hex(state.rgba, self.alpha);
        }
        paint::trigger(ui, &trigger, state.rgba, self.alpha, self.size);
        let [red, green, blue, alpha] = state.rgba;
        trigger.ctx.accesskit_node_builder(trigger.id, |node| {
            node.set_color_value(Color {
                red,
                green,
                blue,
                alpha,
            });
            node.set_value(format_hex(state.rgba, self.alpha));
            node.set_has_popup(HasPopup::Dialog);
            node.set_expanded(state.open);
            if self.read_only {
                node.set_read_only();
            }
        });
        ui.ctx().data_mut(|data| data.insert_temp(state_id, state));
        trigger
    }
}

fn owns_focus(ui: &Ui, state: &State, trigger: &Response) -> bool {
    if let Some(layer) = state.layer {
        if !ui.memory(|m| m.allows_interaction(layer)) {
            return false;
        }
        if ui
            .memory(|m| m.focused())
            .and_then(|id| ui.ctx().read_response(id))
            .is_some_and(|response| response.layer_id == layer)
        {
            return true;
        }
        if state
            .focused
            .is_some_and(|id| ui.memory(|m| m.had_focus_last_frame(id)))
        {
            return true;
        }
    }
    trigger.has_focus() && ui.memory(|m| m.allows_interaction(ui.layer_id()))
}
