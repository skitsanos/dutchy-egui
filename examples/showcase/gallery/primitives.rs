use super::section;
use dutchy_egui::calendar::{Date, Time};
use dutchy_egui::tokens::space;
use dutchy_egui::{
    ColumnWidth, ControlSize, DatePicker, Field, Kbd, NumberInput, RangeSlider, ResponsiveGrid,
    Segmented, SelectOption, Slider, Switch, Text, TextColor, TextSize, TimePicker,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Density {
    Comfortable,
    Compact,
    Dense,
}

pub struct PrimitivesState {
    pub notifications: bool,
    pub autosave: bool,
    pub locked: bool,
    pub density: Option<Density>,
    pub alignment: Option<&'static str>,
    pub volume: f64,
    pub threshold: f64,
    pub latency: (f64, f64),
    pub retries: f64,
    pub timeout: f64,
    pub issued: Option<Date>,
    pub review_by: Option<Date>,
    pub starts: Option<Time>,
    pub ends: Option<Time>,
    pub today: Date,
}

impl Default for PrimitivesState {
    fn default() -> Self {
        let today = host_today();
        Self {
            notifications: true,
            autosave: false,
            locked: true,
            density: Some(Density::Compact),
            alignment: Some("left"),
            volume: 40.0,
            threshold: 0.75,
            latency: (100.0, 400.0),
            retries: 3.0,
            timeout: 2.5,
            issued: Some(today),
            review_by: today.add_days(30),
            starts: Time::new(9, 30, 0),
            ends: Time::new(17, 45, 30),
            today,
        }
    }
}

/// The host clock, read once; the library itself reads no clock.
fn host_today() -> Date {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    Date::from_days_since_epoch((seconds / 86_400) as i64)
        .unwrap_or_else(|| Date::new(2000, 1, 1).expect("valid"))
}

pub fn show(ui: &mut egui::Ui, state: &mut PrimitivesState) {
    ui.add(Text::label("Native additions / Form primitives").color(TextColor::Primary));
    ui.add(Text::heading("Switch. Step. Pick.").size(TextSize::Xl5));
    ui.add(
        Text::new("Switches, segments, key chips, sliders, stepped numbers and date or time entry, all over values your application owns.")
            .color(TextColor::Muted),
    );
    ResponsiveGrid::new("primitive-sections")
        .columns(2)
        .min_column_width(ColumnWidth::Form)
        .show(
            ui,
            &["toggles", "numbers"],
            |key| *key,
            |ui, key| {
                if *key == "toggles" {
                    section(ui, "01", "Switches and segments", |ui| toggles(ui, state));
                } else {
                    section(ui, "02", "Sliders and numbers", |ui| numbers(ui, state));
                }
            },
        );
    section(ui, "03", "Dates and times", |ui| pickers(ui, state));
    summary(ui, state);
}

fn toggles(ui: &mut egui::Ui, state: &mut PrimitivesState) {
    ui.add(Switch::new(
        "notifications",
        &mut state.notifications,
        "Email notifications",
    ));
    ui.add(Switch::new(
        "autosave",
        &mut state.autosave,
        "Autosave drafts every minute",
    ));
    ui.add(Switch::new("locked", &mut state.locked, "Locked by policy").enabled(false));
    ui.add_space(space::SM);
    let densities = [
        SelectOption::new(Density::Comfortable, "Comfortable"),
        SelectOption::new(Density::Compact, "Compact"),
        SelectOption::new(Density::Dense, "Dense"),
    ];
    Field::new("Density").show(ui, |ui| {
        ui.add(Segmented::new("density", &mut state.density, &densities).full_width(true))
    });
    let alignments = [
        SelectOption::new("left", "Left"),
        SelectOption::new("center", "Center"),
        SelectOption::new("right", "Right"),
        SelectOption::new("justify", "Justify").enabled(false),
    ];
    Field::new("Alignment")
        .help("Small segments at natural width; the disabled option is skipped by the arrow keys.")
        .show(ui, |ui| {
            ui.add(
                Segmented::new("alignment", &mut state.alignment, &alignments)
                    .size(ControlSize::Small),
            )
        });
    ui.add_space(space::SM);
    ui.horizontal(|ui| {
        ui.add(Text::new("Command palette").size(TextSize::Sm));
        ui.add(Kbd::new("Ctrl"));
        ui.add(Text::new("+").size(TextSize::Sm));
        ui.add(Kbd::new("K"));
    });
    ui.horizontal(|ui| {
        ui.add(Text::new("Close it with").size(TextSize::Sm));
        ui.add(Kbd::new("Esc"));
    });
}

fn numbers(ui: &mut egui::Ui, state: &mut PrimitivesState) {
    Field::new("Volume").show(ui, |ui| {
        ui.add(
            Slider::new("volume", &mut state.volume, 0.0..=100.0)
                .step(5.0)
                .value_label(|value| format!("{value:.0} %")),
        )
    });
    Field::new("Match threshold")
        .help("Steps of 0.05; the label is host-formatted.")
        .show(ui, |ui| {
            ui.add(
                Slider::new("threshold", &mut state.threshold, 0.0..=1.0)
                    .step(0.05)
                    .value_label(|value| format!("{value:.2}")),
            )
        });
    let (fastest, slowest) = &mut state.latency;
    Field::new("Response time")
        .help("Two handles that meet but never cross, at least 50 ms apart.")
        .show(ui, |ui| {
            ui.add(
                RangeSlider::new("latency", fastest, slowest, 0.0..=1000.0)
                    .step(50.0)
                    .min_separation(50.0)
                    .accessible_name("Response time")
                    .value_label(|low, high| format!("{low:.0}–{high:.0} ms")),
            )
        });
    Field::new("Retries").show(ui, |ui| {
        ui.add(
            NumberInput::new("retries", &mut state.retries)
                .range(0.0..=10.0)
                .accessible_name("Retries"),
        )
    });
    Field::new("Timeout in seconds").show(ui, |ui| {
        ui.add(
            NumberInput::new("timeout", &mut state.timeout)
                .range(0.5..=60.0)
                .step(0.5)
                .decimals(1)
                .size(ControlSize::Small)
                .accessible_name("Timeout"),
        )
    });
    let mut fixed = 8.0;
    Field::new("Workers")
        .help("Read-only values still allow selection and copy.")
        .show(ui, |ui| {
            ui.add(
                NumberInput::new("workers", &mut fixed)
                    .read_only(true)
                    .accessible_name("Workers"),
            )
        });
}

fn pickers(ui: &mut egui::Ui, state: &mut PrimitivesState) {
    let today = state.today;
    let issued = state.issued;
    ResponsiveGrid::new("picker-grid")
        .columns(2)
        .min_column_width(ColumnWidth::Form)
        .show(
            ui,
            &["issued", "review"],
            |key| *key,
            |ui, key| {
                if *key == "issued" {
                    Field::new("Issued")
                        .help("Type an ISO date or pick one; Up and Down step the caret's segment.")
                        .show(ui, |ui| {
                            ui.add(
                                DatePicker::new("issued", &mut state.issued)
                                    .today(today)
                                    .accessible_name("Issued"),
                            )
                        });
                } else {
                    let mut picker = DatePicker::new("review", &mut state.review_by)
                        .today(today)
                        .accessible_name("Review by");
                    if let Some(issued) = issued {
                        picker = picker.min(issued);
                        if let Some(limit) = issued.add_days(90) {
                            picker = picker.max(limit);
                        }
                    }
                    Field::new("Review by")
                        .help("Bounded to ninety days after the issue date.")
                        .show(ui, |ui| ui.add(picker));
                }
            },
        );
    ResponsiveGrid::new("time-grid")
        .columns(2)
        .min_column_width(ColumnWidth::Form)
        .show(
            ui,
            &["starts", "ends"],
            |key| *key,
            |ui, key| {
                if *key == "starts" {
                    Field::new("Starts").show(ui, |ui| {
                        ui.add(
                            TimePicker::new("starts", &mut state.starts).accessible_name("Starts"),
                        )
                    });
                } else {
                    Field::new("Ends")
                        .help("With seconds; Up and Down wrap inside the caret's segment.")
                        .show(ui, |ui| {
                            ui.add(
                                TimePicker::new("ends", &mut state.ends)
                                    .seconds(true)
                                    .accessible_name("Ends"),
                            )
                        });
                }
            },
        );
}

fn summary(ui: &mut egui::Ui, state: &PrimitivesState) {
    ui.add_space(space::LG);
    let issued = state
        .issued
        .map_or_else(|| "none".to_owned(), |d| d.to_string());
    let review = state
        .review_by
        .map_or_else(|| "none".to_owned(), |d| d.to_string());
    let starts = state
        .starts
        .map_or_else(|| "none".to_owned(), |t| t.format(false));
    let ends = state
        .ends
        .map_or_else(|| "none".to_owned(), |t| t.format(true));
    ui.add(
        Text::new(format!(
            "Host state: notifications {}, autosave {}, density {:?}, volume {:.0}, threshold {:.2}, response time {:.0} to {:.0} ms, retries {:.0}, timeout {:.1}, issued {issued}, review by {review}, {starts} to {ends}.",
            state.notifications, state.autosave, state.density, state.volume, state.threshold, state.latency.0, state.latency.1, state.retries, state.timeout
        ))
        .size(TextSize::Sm)
        .color(TextColor::Muted),
    );
}
