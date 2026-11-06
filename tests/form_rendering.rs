//! Form-control states and the application example at multiple scales/layouts.
#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{
    Checkbox, Field, Input, Mode, RadioGroup, Select, SelectOption, Text, TextSize, Textarea,
    Theme, validation::ValidationState,
};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

#[derive(Default)]
struct Values {
    name: String,
    error: String,
    note: String,
    frozen: String,
    choice: Option<u8>,
    radio: Option<u8>,
    checked: bool,
}

fn matrix(ui: &mut egui::Ui, values: &mut Values) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Form follows function.").size(TextSize::Xl4));
    Field::new("Name")
        .required(true)
        .help("A clearly associated field label.")
        .show(ui, |ui| {
            ui.add(Input::new("name", &mut values.name).placeholder("Your name"))
        });
    Field::new("Email")
        .required(true)
        .validation(&ValidationState::Invalid(
            "Enter a valid email address.".into(),
        ))
        .show(ui, |ui| ui.add(Input::new("error", &mut values.error)));
    Field::new("Reference").show(ui, |ui| {
        ui.add(Input::new("reference", &mut values.frozen).read_only(true))
    });
    let choices = [
        SelectOption::new(0, "Editor"),
        SelectOption::new(1, "Viewer"),
    ];
    Field::new("Role").show(ui, |ui| {
        ui.add(Select::new("choice", &mut values.choice, &choices))
    });
    Field::new("Digest").show(ui, |ui| {
        ui.add(RadioGroup::new("radio", &mut values.radio, &choices))
    });
    ui.add(Checkbox::new(
        "checked",
        &mut values.checked,
        "Send product updates",
    ));
    Field::new("Notes").show(ui, |ui| {
        ui.add(
            Textarea::new("notes", &mut values.note)
                .char_limit(100)
                .show_count(true),
        )
    });
    Field::new("Availability")
        .validation(&ValidationState::Pending)
        .show(ui, |ui| ui.add(Input::new("pending", &mut values.frozen)));
}

#[test]
fn form_states_render_in_light_and_dark_at_multiple_scales() {
    let mut results = SnapshotResults::new();
    for (mode, scale, name) in [
        (Mode::Light, 1.0, "forms-light-1x"),
        (Mode::Dark, 1.5, "forms-dark-1_5x"),
    ] {
        let theme = Theme::new(dutchy_egui::Accent::Orange, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([640.0, 900.0])
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(
                matrix,
                Values {
                    error: "wrong address".into(),
                    frozen: "WORKSPACE-001".into(),
                    radio: Some(0),
                    checked: true,
                    note: "A little context.".into(),
                    ..Default::default()
                },
            );
        h.run();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn example_form_renders_desktop_and_narrow_and_exposes_validation_errors() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [
        ([1200.0, 1100.0], "forms-desktop"),
        ([480.0, 1100.0], "forms-narrow"),
    ] {
        let mut h = Harness::builder()
            .with_size(size)
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| {
                common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                gallery::Showcase {
                    page: gallery::Page::Forms,
                    reduced_motion: true,
                    ..Default::default()
                }
            });
        h.run();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/{name}.png"))
            .unwrap();
        assert!(!h.state_mut().forms.validate());
        h.run_steps(2);
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/{name}-errors.png"))
            .unwrap();
    }
}

#[test]
fn application_form_validates_and_resets_through_its_public_controls() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1800.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Forms,
                reduced_motion: true,
                ..Default::default()
            }
        });
    h.get_by_label("Full name").focus();
    h.run();
    h.get_by_label("Full name").type_text("Ada Developer");
    h.run();
    h.get_by_label("Work email").focus();
    h.run();
    h.get_by_label("Work email").type_text("ada@example.test");
    h.run();
    h.get_by_label("Workspace role").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    h.key_press(egui::Key::Tab);
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(h.state().forms.role, Some("editor"));
    h.get_by_label("VALIDATE FORM").click();
    h.run();
    assert!(h.state().forms.submitted && h.state().forms.is_valid());
    h.get_by_label("RESET FORM").click();
    h.run();
    assert!(h.state().forms.email.is_empty());
    assert!(!h.state().forms.name_check.touched());
    assert!(!h.state().forms.submitted);
}
