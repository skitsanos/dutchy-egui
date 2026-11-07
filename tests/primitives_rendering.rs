//! Form-primitive baselines across accents, modes and scales, plus the showcase page.
#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::calendar::{Date, Time};
use dutchy_egui::{
    Accent, ControlSize, DatePicker, Field, Kbd, Mode, NumberInput, Segmented, SelectOption,
    Slider, Switch, Text, TextSize, Theme, TimePicker,
};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

struct Values {
    on: bool,
    off: bool,
    choice: Option<u8>,
    level: f64,
    count: f64,
    date: Option<Date>,
    time: Option<Time>,
}

fn fixture(ui: &mut egui::Ui, values: &mut Values) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Switch. Step. Pick.").size(TextSize::Xl4));
    ui.add(Switch::new("on", &mut values.on, "Email notifications"));
    ui.add(Switch::new("off", &mut values.off, "Autosave drafts"));
    ui.horizontal(|ui| {
        ui.add(Kbd::new("Ctrl"));
        ui.add(Kbd::new("K"));
    });
    let options = [
        SelectOption::new(0, "Day"),
        SelectOption::new(1, "Week"),
        SelectOption::new(2, "Month").enabled(false),
    ];
    Field::new("View").show(ui, |ui| {
        ui.add(Segmented::new("view", &mut values.choice, &options).full_width(true))
    });
    Field::new("Level").show(ui, |ui| {
        ui.add(
            Slider::new("level", &mut values.level, 0.0..=100.0)
                .step(5.0)
                .value_label(|value| format!("{value:.0} %")),
        )
    });
    Field::new("Count").show(ui, |ui| {
        ui.add(
            NumberInput::new("count", &mut values.count)
                .range(0.0..=10.0)
                .size(ControlSize::Small)
                .accessible_name("Count"),
        )
    });
    Field::new("Issued").show(ui, |ui| {
        ui.add(
            DatePicker::new("issued", &mut values.date)
                .today(Date::new(2026, 11, 7).unwrap())
                .accessible_name("Issued"),
        )
    });
    Field::new("Starts").show(ui, |ui| {
        ui.add(
            TimePicker::new("starts", &mut values.time)
                .seconds(true)
                .accessible_name("Starts"),
        )
    });
}

fn values() -> Values {
    Values {
        on: true,
        off: false,
        choice: Some(1),
        level: 40.0,
        count: 3.0,
        date: Date::new(2026, 11, 12),
        time: Time::new(9, 30, 0),
    }
}

#[test]
fn primitives_render_across_accents_modes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "primitives-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "primitives-dark-1_5x"),
        (Accent::Green, Mode::Light, 2.0, "primitives-green-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([480.0, 640.0])
            .with_render_cursor(false)
            .with_theme(match mode {
                Mode::Light => egui::Theme::Light,
                Mode::Dark => egui::Theme::Dark,
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(fixture, values());
        h.run();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn open_calendar_renders_the_month_grid() {
    let mut h = Harness::builder()
        .with_size([480.0, 900.0])
        .with_render_cursor(false)
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(fixture, values());
    h.run();
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Choose Issued")
        .click();
    h.run_steps(2);
    h.get_by_label("November 2026");
    h.snapshot("primitives-calendar-1x");
}

#[test]
fn showcase_controls_page_renders_desktop_and_narrow_with_a_valid_tree() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [
        ([1200.0, 1500.0], "controls-desktop"),
        ([480.0, 1900.0], "controls-narrow"),
    ] {
        let mut h = Harness::builder()
            .with_size(size)
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| {
                common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                gallery::Showcase {
                    page: gallery::Page::Controls,
                    reduced_motion: true,
                    ..Default::default()
                }
            });
        h.run();
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/{name}.png"))
            .unwrap();
        assert!(h.get_by_label("Email notifications").rect().right() <= size[0]);
    }
}
