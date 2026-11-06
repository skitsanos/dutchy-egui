use super::style;
use crate::{
    ControlSize, Text, TextColor, Theme,
    theme::widget_visuals,
    tokens::{control, space},
};
use egui::{IdSalt, Response, Ui, Widget, vec2};
use std::borrow::Cow;

/// One typed option shared by Select and RadioGroup.
#[derive(Clone, Debug)]
pub struct SelectOption<'a, T> {
    /// Stored application value.
    pub value: T,
    /// Visible/accessibility label.
    pub label: Cow<'a, str>,
    /// Whether this option can be selected.
    pub enabled: bool,
    pub(super) id: IdSalt,
}

impl<'a, T> SelectOption<'a, T> {
    /// Construct an enabled option. Its label is the default stable option salt.
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        Self {
            id: IdSalt::new(label.as_ref()),
            value,
            label,
            enabled: true,
        }
    }
    /// Disable an option without removing it from the list.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Supply an explicit option salt, e.g. when labels repeat.
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id = IdSalt::new(id);
        self
    }
}

/// Typed single selection using egui's ComboBox/popup machinery.
#[must_use = "Add this select to a Ui"]
pub struct Select<'a, T> {
    id: IdSalt,
    value: &'a mut Option<T>,
    options: &'a [SelectOption<'a, T>],
    placeholder: Cow<'a, str>,
    name: Option<Cow<'a, str>>,
    size: ControlSize,
    enabled: bool,
}

impl<'a, T: Clone + PartialEq> Select<'a, T> {
    /// Bind a stable ID and host selection. None displays the placeholder.
    pub fn new(
        id: impl egui::AsIdSalt,
        value: &'a mut Option<T>,
        options: &'a [SelectOption<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            options,
            placeholder: Cow::Borrowed("Select an option…"),
            name: None,
            size: ControlSize::Medium,
            enabled: true,
        }
    }
    /// Empty-selection text.
    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    /// Name for standalone use; Field supplies one otherwise.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Shared control height and type scale.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disable the trigger and all options.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn show(self, ui: &mut Ui, width: f32) -> Response {
        let theme = Theme::from_context(ui.ctx());
        configure(ui, theme, self.size);
        if !self.enabled {
            ui.disable();
        }
        let selected = self
            .options
            .iter()
            .find(|option| self.value.as_ref() == Some(&option.value));
        let label = selected.map_or(self.placeholder.as_ref(), |option| option.label.as_ref());
        let color = if selected.is_some() {
            TextColor::Foreground
        } else {
            TextColor::Muted
        };
        let job = Text::new(label)
            .size(self.size.text_size())
            .color(color)
            .layout_job(&theme, f32::INFINITY);
        let mut changed = false;
        let mut chosen = false;
        let mut response = egui::ComboBox::from_id_salt(self.id)
            .width(width)
            .height(control::SELECT_POPUP_HEIGHT)
            .selected_text(job)
            .wrap_mode(egui::TextWrapMode::Truncate)
            .icon(chevron)
            .show_ui(ui, |ui| {
                popup_style(ui, theme);
                for option in self.options {
                    ui.push_id(option.id, |ui| {
                        let response = ui.add_enabled(
                            option.enabled,
                            egui::Button::selectable(
                                self.value.as_ref() == Some(&option.value),
                                Text::new(option.label.as_ref())
                                    .size(crate::TextSize::Sm)
                                    .weight(if self.value.as_ref() == Some(&option.value) {
                                        crate::Weight::Bold
                                    } else {
                                        crate::Weight::Regular
                                    })
                                    .color(egui::Color32::PLACEHOLDER)
                                    .layout_job(&theme, width),
                            ),
                        );
                        response.ctx.accesskit_node_builder(response.id, |node| {
                            node.set_selected(self.value.as_ref() == Some(&option.value));
                        });
                        if response.clicked() {
                            chosen = true;
                            changed = self.value.as_ref() != Some(&option.value);
                            *self.value = Some(option.value.clone());
                            ui.close();
                        }
                    });
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        if chosen {
            response.request_focus();
        }
        if let Some(name) = self.name {
            response = response.accessible_name(name.into_owned());
        }
        style::finish(ui, &response);
        response
    }
}

impl<T: Clone + PartialEq> Widget for Select<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        ui.scope_builder(egui::UiBuilder::new().id_salt(self.id), |ui| {
            ui.allocate_ui_with_layout(
                vec2(width, self.size.height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show(ui, width),
            )
            .inner
        })
        .inner
    }
}

fn configure(ui: &mut Ui, theme: Theme, size: ControlSize) {
    let inherited = style::inherited_border(ui);
    let normal = widget_visuals(
        theme.palette.background,
        inherited,
        theme.palette.foreground,
    );
    let style = ui.style_mut();
    style.spacing.interact_size.y = size.height();
    style.spacing.button_padding = vec2(space::MD, space::XS);
    style.visuals.widgets.inactive = normal;
    style.visuals.widgets.hovered = normal;
    style.visuals.widgets.active = normal;
    style.visuals.widgets.open = normal;
}

fn chevron(ui: &Ui, rect: egui::Rect, visuals: &egui::style::WidgetVisuals, open: bool) {
    let center = rect.center();
    let half = space::XS;
    let sign = if open { -1.0 } else { 1.0 };
    ui.painter().add(egui::Shape::line(
        vec![
            center + vec2(-half, -half * sign / 2.0),
            center + vec2(0.0, half * sign / 2.0),
            center + vec2(half, -half * sign / 2.0),
        ],
        visuals.fg_stroke,
    ));
}

fn popup_style(ui: &mut Ui, theme: Theme) {
    let p = theme.palette;
    let style = ui.style_mut();
    style.spacing.button_padding = vec2(space::LG, space::MD);
    style.visuals.selection.bg_fill = p.muted;
    style.visuals.selection.stroke = egui::Stroke::new(crate::tokens::border::THIN, p.foreground);
    let mut hover = widget_visuals(p.primary, p.primary, p.on_primary);
    hover.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered = hover;
    style.visuals.widgets.active = hover;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
}
