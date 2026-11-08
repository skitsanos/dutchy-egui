use super::{
    ColorPicker, ColorPreset, paint,
    state::{State, format_hex, parse_hex},
};
use crate::{
    Button, ButtonVariant, ControlSize, Input, NumberInput, Slider, Text, TextColor, TextSize,
    Theme,
    interaction::reveal_focused,
    tokens::{color_picker as geometry, space},
};
use egui::{Id, Sense, Ui, UiBuilder, WidgetInfo, accesskit::Role, vec2};

pub(super) fn show(picker: &ColorPicker<'_>, ui: &mut Ui, id: Id, state: &mut State) -> bool {
    state.layer = Some(ui.layer_id());
    ui.spacing_mut().item_spacing = vec2(space::SM, space::SM);
    let defaults = default_palette(Theme::from_context(ui.ctx()));
    let presets = picker.palette.unwrap_or(&defaults);
    if !presets.is_empty() {
        ui.add(Text::label("Palette"));
        palette(ui, id, presets, picker.read_only, picker.alpha, state);
        ui.add_space(space::XS);
    }
    hex(ui, id, picker, state);
    ui.add_space(space::XS);
    for (index, label) in ["Red", "Green", "Blue", "Alpha"]
        .into_iter()
        .enumerate()
        .take(if picker.alpha { 4 } else { 3 })
    {
        channel(ui, id, index, label, picker, state);
    }
    ui.add_space(space::SM);
    let done = ui
        .scope_builder(UiBuilder::new().scope_id(id.with("actions")), |ui| {
            ui.horizontal_wrapped(|ui| {
                let copy = ui.add(
                    Button::new("Copy hex")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Small),
                );
                reveal_focused(ui, &copy, ui.max_rect().min);
                if copy.clicked() {
                    ui.ctx().copy_text(format_hex(state.rgba, picker.alpha));
                }
                let done = ui.add(
                    Button::new("Done")
                        .variant(ButtonVariant::Outline)
                        .size(ControlSize::Small),
                );
                reveal_focused(ui, &done, ui.max_rect().min);
                done.clicked()
            })
            .inner
        })
        .inner;
    state.focused = ui.memory(|m| m.focused()).filter(|id| {
        ui.ctx()
            .read_response(*id)
            .is_some_and(|response| response.layer_id == ui.layer_id())
    });
    done
}

fn default_palette(theme: Theme) -> [ColorPreset<'static>; 8] {
    let p = theme.palette;
    [
        ColorPreset::new("Primary", p.primary),
        ColorPreset::new("Foreground", p.foreground),
        ColorPreset::new("Background", p.background),
        ColorPreset::new("Muted", p.muted),
        ColorPreset::new("Success", p.success),
        ColorPreset::new("Warning", p.warning),
        ColorPreset::new("Destructive", p.destructive),
        ColorPreset::new("Info", p.info),
    ]
}

fn palette(
    ui: &mut Ui,
    id: Id,
    presets: &[ColorPreset<'_>],
    read_only: bool,
    alpha: bool,
    state: &mut State,
) {
    ui.add_enabled_ui(!read_only, |ui| {
        ui.horizontal_wrapped(|ui| {
            for preset in presets {
                let (rect, _) = ui
                    .allocate_exact_size(vec2(geometry::SWATCH, geometry::SWATCH), Sense::hover());
                let response = ui.interact(
                    rect,
                    id.with(("preset", preset.label.as_ref())),
                    Sense::click(),
                );
                let color = preset.color.to_srgba_unmultiplied();
                let selected = if alpha {
                    state.rgba == color
                } else {
                    state.rgba[..3] == color[..3]
                };
                response.widget_info(|| {
                    WidgetInfo::selected(
                        Role::Button,
                        ui.is_enabled(),
                        selected,
                        preset.label.as_ref(),
                    )
                });
                if response.clicked() {
                    state.set_preset(preset.color, alpha);
                    response.request_focus();
                }
                paint::preset(ui, &response, preset.color, selected);
                reveal_focused(ui, &response, ui.max_rect().min);
                response.on_hover_text(format!("{} · {}", preset.label, format_hex(color, alpha)));
            }
        });
    });
}

fn hex(ui: &mut Ui, id: Id, picker: &ColorPicker<'_>, state: &mut State) {
    ui.add(Text::label(if picker.alpha {
        "Hex RGBA"
    } else {
        "Hex RGB"
    }));
    let response = ui.add(
        Input::new(id.with("hex"), &mut state.hex)
            .size(ControlSize::Small)
            .read_only(picker.read_only)
            .accessible_name("Hex color"),
    );
    if response.changed()
        && !picker.read_only
        && let Some(rgba) = parse_hex(&state.hex, picker.alpha, state.rgba[3])
    {
        state.rgba = rgba;
    }
    if state.pending_focus
        && !ui.is_sizing_pass()
        && ui.memory(|m| m.allows_interaction(ui.layer_id()))
    {
        response.request_focus();
        state.pending_focus = false;
        ui.ctx().request_repaint();
    }
    reveal_focused(ui, &response, ui.max_rect().min);
    if response.has_focus() {
        let valid = parse_hex(&state.hex, picker.alpha, state.rgba[3]).is_some();
        if !valid {
            response.ctx.accesskit_node_builder(response.id, |node| {
                node.set_invalid(egui::accesskit::Invalid::True)
            });
            ui.add(
                Text::new(if picker.alpha {
                    "Use RGB or RGBA hex digits."
                } else {
                    "Use three or six RGB hex digits."
                })
                .size(TextSize::Xs)
                .color(TextColor::Destructive),
            );
        }
    } else {
        state.hex = format_hex(state.rgba, picker.alpha);
    }
}

fn channel(
    ui: &mut Ui,
    id: Id,
    index: usize,
    label: &str,
    picker: &ColorPicker<'_>,
    state: &mut State,
) {
    let mut value = f64::from(state.rgba[index]);
    let narrow = ui.available_width() < geometry::CHANNEL_ROW_MIN;
    ui.scope_builder(
        UiBuilder::new().scope_id(id.with(("channel", index))),
        |ui| {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [geometry::CHANNEL_LABEL_WIDTH, ControlSize::Small.height()],
                    Text::label(&label[..1]),
                );
                if !narrow {
                    let width =
                        (ui.available_width() - geometry::CHANNEL_VALUE_WIDTH - space::SM).max(0.0);
                    let response = ui.add(
                        Slider::new("slider", &mut value, 0.0..=255.0)
                            .step(1.0)
                            .width(width)
                            .enabled(!picker.read_only)
                            .accessible_name(label),
                    );
                    reveal_focused(ui, &response, ui.max_rect().min);
                }
                let response = ui.add(
                    NumberInput::new("number", &mut value)
                        .range(0.0..=255.0)
                        .size(ControlSize::Small)
                        .width(if narrow {
                            ui.available_width()
                        } else {
                            geometry::CHANNEL_VALUE_WIDTH
                        })
                        .read_only(picker.read_only)
                        .accessible_name(format!("{label} value")),
                );
                reveal_focused(ui, &response, ui.max_rect().min);
            });
            if narrow {
                let response = ui.add(
                    Slider::new("slider", &mut value, 0.0..=255.0)
                        .step(1.0)
                        .enabled(!picker.read_only)
                        .accessible_name(label),
                );
                reveal_focused(ui, &response, ui.max_rect().min);
            }
        },
    );
    let next = value.round().clamp(0.0, 255.0) as u8;
    if next != state.rgba[index] {
        state.rgba[index] = next;
        state.hex = format_hex(state.rgba, picker.alpha);
    }
}
