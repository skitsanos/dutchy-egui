use crate::{
    Button, ButtonVariant, ControlSize, Dropdown, DropdownAlign, DropdownResponse, MenuItem,
    MenuWidth,
    interaction::paint_focus,
    tokens::{border, space},
};
use egui::{IdSalt, Response, Ui};
use std::borrow::Cow;

/// Independent primary and alternative requests from a split action.
pub struct SplitButtonResponse<T> {
    /// Primary native button, retaining its own focus and accessible name.
    pub primary: Response,
    /// Separately named arrow and popup.
    pub dropdown: DropdownResponse<T>,
    /// Primary activation; opening alternatives never sets this flag.
    pub primary_clicked: bool,
    /// Alternative leaf action; the host executes it.
    pub action: Option<T>,
}
/// Primary action joined to an independently focusable alternatives menu.
#[must_use = "Call show to render the split button"]
pub struct SplitButton<'a, T> {
    id: IdSalt,
    label: Cow<'a, str>,
    name: Option<Cow<'a, str>>,
    open: &'a mut bool,
    items: &'a [MenuItem<'a, T>],
    variant: ButtonVariant,
    size: ControlSize,
    enabled: bool,
    loading: bool,
    width: MenuWidth,
    align: DropdownAlign,
}
impl<'a, T: Clone> SplitButton<'a, T> {
    /// Bind stable identity, the primary label, popup visibility and alternatives.
    pub fn new(
        id: impl egui::AsIdSalt,
        label: impl Into<Cow<'a, str>>,
        open: &'a mut bool,
        items: &'a [MenuItem<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            label: label.into(),
            name: None,
            open,
            items,
            variant: ButtonVariant::Primary,
            size: ControlSize::Medium,
            enabled: true,
            loading: false,
            width: MenuWidth::Small,
            align: DropdownAlign::Start,
        }
    }
    /// Primary/arrow surface treatment.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Shared target height.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disable both actions and close the popup.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Busy primary with an inactive alternatives arrow.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Accessible arrow/menu name; defaults to "More {label} options".
    pub fn alternatives_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Preferred alternatives popup width.
    pub fn menu_width(mut self, width: MenuWidth) -> Self {
        self.width = width;
        self
    }
    /// Preferred popup edge alignment.
    pub fn align(mut self, align: DropdownAlign) -> Self {
        self.align = align;
        self
    }
    /// Compose two adjacent targets and the native Dutchy Dropdown lifecycle.
    pub fn show(self, ui: &mut Ui) -> SplitButtonResponse<T> {
        let name = self
            .name
            .unwrap_or_else(|| format!("More {} options", self.label).into());
        let arrow = self.size.height();
        let button = Button::new(self.label)
            .variant(self.variant)
            .size(self.size)
            .enabled(self.enabled)
            .loading(self.loading);
        let overlap = border::STANDARD;
        let width = button
            .natural_width(ui)
            .min((ui.available_width() - arrow + overlap).max(0.0));
        let total = (width + arrow - overlap).max(0.0).min(ui.available_width());
        ui.allocate_ui_with_layout(
            egui::vec2(total, self.size.height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.push_id(("dutchy-split-button", self.id), |ui| {
                    ui.spacing_mut().item_spacing.x = -overlap;
                    let primary = ui
                        .allocate_ui_with_layout(
                            egui::vec2(width, self.size.height()),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| ui.add(button),
                        )
                        .inner;
                    if primary.has_focus()
                        && primary.enabled()
                        && !self.items.is_empty()
                        && ui.input_mut(|input| {
                            input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                                || input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown)
                        })
                    {
                        *self.open = true;
                    }
                    // The arrow overlaps the primary by one border so borders join.
                    // Only the primary paints (and hit-tests) that strip, so faded
                    // disabled/busy fills are never blended twice.
                    let joint = primary.rect.right();
                    let dropdown = Dropdown::new("alternatives", self.open, self.items)
                        .enabled(self.enabled && !self.loading && !self.items.is_empty())
                        .width(self.width)
                        .align(self.align)
                        .accessible_name(name.clone())
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(arrow, self.size.height()),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let mut clip = ui.clip_rect();
                                    clip.min.x = clip.min.x.max(joint);
                                    ui.set_clip_rect(clip);
                                    let response = ui
                                        .add(
                                            Button::new("")
                                                .variant(self.variant)
                                                .size(self.size)
                                                .full_width(true),
                                        )
                                        .accessible_name(name.to_string());
                                    let extent = crate::tokens::actions::icon(self.size);
                                    let rect = egui::Rect::from_center_size(
                                        response.rect.center(),
                                        egui::vec2(extent, extent),
                                    );
                                    // Match the Button's own state: focus and press use
                                    // its highlighted visuals, like hover.
                                    let lit = response.hovered()
                                        || response.has_focus()
                                        || response.is_pointer_button_down_on();
                                    let ink = self
                                        .variant
                                        .visuals(crate::Theme::from_context(ui.ctx()), lit)
                                        .fg_stroke
                                        .color;
                                    chevron(ui.painter(), rect, ink);
                                    response
                                },
                            )
                            .inner
                        });
                    let theme = crate::Theme::from_context(ui.ctx());
                    let p = theme.palette;
                    let x = dropdown.trigger.rect.left();
                    let mut painter = ui.painter().clone();
                    if (!self.enabled || self.loading) && ui.is_enabled() {
                        painter.multiply_opacity(ui.visuals().disabled_alpha());
                    }
                    if self.variant != ButtonVariant::Outline {
                        painter.line_segment(
                            [
                                egui::pos2(x, primary.rect.top() + space::SM),
                                egui::pos2(x, primary.rect.bottom() - space::SM),
                            ],
                            egui::Stroke::new(border::STANDARD, p.border),
                        );
                    }
                    // Rings extend past each half; repaint them over the joint.
                    let focus = self.variant.focus_color(theme);
                    paint_focus(ui, &primary, focus);
                    paint_focus(ui, &dropdown.trigger, focus);
                    SplitButtonResponse {
                        primary_clicked: primary.clicked(),
                        primary,
                        action: dropdown.action.clone(),
                        dropdown,
                    }
                })
                .inner
            },
        )
        .inner
    }
}
fn chevron(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let d = rect.width() / 4.0;
    let c = rect.center();
    painter.line_segment(
        [c + egui::vec2(-d, -d / 2.0), c + egui::vec2(0.0, d / 2.0)],
        egui::Stroke::new(border::STANDARD, color),
    );
    painter.line_segment(
        [c + egui::vec2(0.0, d / 2.0), c + egui::vec2(d, -d / 2.0)],
        egui::Stroke::new(border::STANDARD, color),
    );
}
