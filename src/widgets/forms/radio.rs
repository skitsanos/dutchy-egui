use super::SelectOption;
use crate::tokens::space;
use egui::{Id, IdSalt, Key, Modifiers, Response, Ui, Widget, WidgetInfo};
use std::borrow::Cow;

/// Exclusive typed selection with square indicators and roving keyboard focus.
/// Arrow keys wrap across enabled options; Home/End choose the first/last option.
#[must_use = "Add this radio group to a Ui"]
pub struct RadioGroup<'a, T> {
    id: IdSalt,
    value: &'a mut Option<T>,
    options: &'a [SelectOption<'a, T>],
    name: Cow<'a, str>,
    enabled: bool,
}

impl<'a, T: Clone + PartialEq> RadioGroup<'a, T> {
    /// Bind a stable group ID and application-owned selection.
    pub fn new(
        id: impl egui::AsIdSalt,
        value: &'a mut Option<T>,
        options: &'a [SelectOption<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            options,
            name: Cow::Borrowed("Options"),
            enabled: true,
        }
    }
    /// Group name; Field can supply the visible label instead.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable the entire group.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn show(mut self, ui: &mut Ui) -> Response {
        if !self.enabled {
            ui.disable();
        }
        let id = ui.make_persistent_id(self.id);
        let mut changed = self.navigate(ui, id);
        let tab_index = self
            .options
            .iter()
            .position(|o| o.enabled && self.value.as_ref() == Some(&o.value))
            .or_else(|| self.options.iter().position(|o| o.enabled));
        let group = ui.scope_builder(egui::UiBuilder::new().id_salt(self.id), |ui| {
            ui.spacing_mut().item_spacing.y = space::XS;
            for (index, option) in self.options.iter().enumerate() {
                let mut selected = self.value.as_ref() == Some(&option.value);
                let response = ui.add_enabled(option.enabled, |ui: &mut Ui| {
                    super::toggle::show(
                        ui,
                        id.with(option.id),
                        &option.label,
                        &mut selected,
                        true,
                        Some(index) == tab_index,
                    )
                });
                if response.changed() {
                    *self.value = Some(option.value.clone());
                    changed = true;
                }
            }
        });
        let mut response = group.response;
        response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::RadioGroup,
                ui.is_enabled(),
                self.name.as_ref(),
            )
        });
        if changed {
            response.mark_changed();
        }
        response
    }

    fn navigate(&mut self, ui: &mut Ui, id: Id) -> bool {
        if !ui.is_enabled() {
            return false;
        }
        let enabled: Vec<_> = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, option)| option.enabled)
            .collect();
        let focused = ui.memory(|memory| memory.focused());
        let Some(current) = enabled
            .iter()
            .position(|(_, option)| focused == Some(id.with(option.id)))
        else {
            return false;
        };
        let direction = ui.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowDown)
                || input.consume_key(Modifiers::NONE, Key::ArrowRight)
            {
                Some(1)
            } else if input.consume_key(Modifiers::NONE, Key::ArrowUp)
                || input.consume_key(Modifiers::NONE, Key::ArrowLeft)
            {
                Some(-1)
            } else if input.consume_key(Modifiers::NONE, Key::Home) {
                Some(-(current as isize))
            } else if input.consume_key(Modifiers::NONE, Key::End) {
                Some(enabled.len() as isize - 1 - current as isize)
            } else {
                None
            }
        });
        let Some(direction) = direction else {
            return false;
        };
        let next = (current as isize + direction).rem_euclid(enabled.len() as isize) as usize;
        let option = enabled[next].1;
        ui.memory_mut(|memory| {
            memory.move_focus(egui::FocusDirection::None);
            memory.request_focus(id.with(option.id));
        });
        let changed = self.value.as_ref() != Some(&option.value);
        *self.value = Some(option.value.clone());
        changed
    }
}

impl<T: Clone + PartialEq> Widget for RadioGroup<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt(self.id)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| self.show(ui),
        )
        .inner
    }
}
