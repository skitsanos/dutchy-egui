use super::section;
use dutchy_egui::{
    Accent, ColorPicker, ColorPreset, ColumnWidth, ControlSize, Field, Mode, ResponsiveGrid,
    Switch, Text, TextColor, TextSize, Theme,
    tokens::{border, space},
};
use egui::{Color32, Frame, Margin, Stroke, Ui, vec2};

pub struct ColorsState {
    color: Color32,
    overlay: Color32,
    brand: Color32,
    disabled: bool,
    read_only: bool,
    changes: usize,
}

impl Default for ColorsState {
    fn default() -> Self {
        let p = Theme::default().palette;
        let [r, g, b, _] = p.primary.to_srgba_unmultiplied();
        Self {
            color: p.primary,
            overlay: Color32::from_rgba_unmultiplied(r, g, b, 128),
            brand: Theme::new(Accent::Blue, Mode::Light).palette.primary,
            disabled: false,
            read_only: false,
            changes: 0,
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut ColorsState) {
    ui.add(Text::label("Application essentials / Color").color(TextColor::Primary));
    ui.add(Text::heading("Make it your color.").size(TextSize::Xl5));
    ui.add(Text::new("Named palettes, exact hex entry and channel controls. Your application owns the color and decides what it changes.")
        .color(TextColor::Muted));
    ResponsiveGrid::new("color-sections")
        .columns(2)
        .min_column_width(ColumnWidth::Form)
        .show(
            ui,
            &["rgb", "alpha", "brand", "states"],
            |key| *key,
            |ui, key| match *key {
                "rgb" => section(ui, "01", "Choose / preview / commit", |ui| rgb(ui, state)),
                "alpha" => section(ui, "02", "Transparency", |ui| alpha(ui, state)),
                "brand" => section(ui, "03", "Application palette", |ui| brand(ui, state)),
                _ => section(ui, "04", "Shared sizes / controlled states", |ui| {
                    states(ui, state)
                }),
            },
        );
    ui.add(
        Text::new(format!("Host observed {} color changes.", state.changes))
            .color(TextColor::Muted),
    );
}

fn rgb(ui: &mut Ui, state: &mut ColorsState) {
    let response = Field::new("Project color").show(ui, |ui| {
        ui.add(ColorPicker::new("project", &mut state.color).accessible_name("Project color"))
    });
    state.changes += usize::from(response.changed());
    ui.add_space(space::LG);
    preview(ui, state.color, "Live project color");
}

fn alpha(ui: &mut Ui, state: &mut ColorsState) {
    let response = Field::new("Overlay color")
        .help("Alpha is the last byte: #RRGGBBAA.")
        .show(ui, |ui| {
            ui.add(
                ColorPicker::new("overlay", &mut state.overlay)
                    .alpha(true)
                    .accessible_name("Overlay color"),
            )
        });
    state.changes += usize::from(response.changed());
    ui.add_space(space::LG);
    preview(ui, state.overlay, "Live overlay color");
}

fn brand(ui: &mut Ui, state: &mut ColorsState) {
    let presets = Accent::ALL.map(|accent| {
        ColorPreset::new(
            accent.name(),
            Theme::new(accent, Mode::Light).palette.primary,
        )
    });
    let response = Field::new("Brand color").show(ui, |ui| {
        ui.add(
            ColorPicker::new("brand", &mut state.brand)
                .palette(&presets)
                .accessible_name("Brand color"),
        )
    });
    state.changes += usize::from(response.changed());
    ui.add(Text::new("The host supplies these named colors; the widget never changes the application's theme.")
        .size(TextSize::Sm).color(TextColor::Muted));
}

fn states(ui: &mut Ui, state: &mut ColorsState) {
    ui.add(Switch::new(
        "color-disabled",
        &mut state.disabled,
        "Disable the sample controls",
    ));
    ui.add(Switch::new(
        "color-read-only",
        &mut state.read_only,
        "Read-only inspection and copy",
    ));
    for (size, name) in [
        (ControlSize::Small, "Small"),
        (ControlSize::Medium, "Medium"),
        (ControlSize::Large, "Large"),
    ] {
        let response = Field::new(name).show(ui, |ui| {
            ui.add(
                ColorPicker::new(name, &mut state.color)
                    .size(size)
                    .enabled(!state.disabled)
                    .read_only(state.read_only)
                    .accessible_name(name),
            )
        });
        state.changes += usize::from(response.changed());
    }
}

fn preview(ui: &mut Ui, color: Color32, label: &str) {
    let p = Theme::from_context(ui.ctx()).palette;
    Frame::NONE
        .fill(p.muted)
        .stroke(Stroke::new(border::THIN, p.border))
        .inner_margin(Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let (rect, response) =
                    ui.allocate_exact_size(vec2(space::HERO, space::HERO), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0, color);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::accesskit::Role::Image, true, label)
                });
                let [r, g, b, a] = color.to_srgba_unmultiplied();
                ui.add(
                    Text::new(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))
                        .family(dutchy_egui::Family::Mono),
                );
            });
        });
}
