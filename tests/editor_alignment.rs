//! Single-line editors centre their text row in the control frame.
#![cfg(feature = "bundled-fonts")]

mod common;

use dutchy_egui::calendar::Time;
use dutchy_egui::{ControlSize, Input, Textarea, Theme, TimePicker};
use egui_kittest::Harness;

/// Vertical centre of the painted text and of the 2-point control frame.
fn centres(build: impl FnMut(&mut egui::Ui) + 'static) -> (f32, f32) {
    let mut h = Harness::builder()
        .with_size([320.0, 200.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(build);
    h.run();
    let mut text = None;
    let mut frame = None;
    for shape in &h.output().shapes {
        match &shape.shape {
            egui::Shape::Text(t) if !t.galley.job.text.is_empty() => {
                text = Some(t.pos.y + t.galley.size().y / 2.0);
            }
            egui::Shape::Rect(r) if r.stroke.width > 1.5 && r.rect.width() > 200.0 => {
                frame = Some(r.rect);
            }
            _ => {}
        }
    }
    (text.expect("text"), frame.expect("frame").center().y)
}

#[test]
fn single_line_inputs_centre_their_text_at_every_size() {
    for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
        let mut value = String::from("Ada Developer");
        let (text, frame) = centres(move |ui| {
            ui.add(
                Input::new("name", &mut value)
                    .size(size)
                    .accessible_name("Name"),
            );
        });
        assert!(
            (text - frame).abs() < 0.75,
            "{size:?}: text centre {text} vs frame centre {frame}"
        );
    }
}

#[test]
fn time_picker_centres_its_text() {
    let mut value = Time::new(9, 30, 0);
    let (text, frame) = centres(move |ui| {
        ui.add(TimePicker::new("starts", &mut value).accessible_name("Starts"));
    });
    assert!(
        (text - frame).abs() < 0.75,
        "text centre {text} vs frame centre {frame}"
    );
}

#[test]
fn textareas_keep_their_text_at_the_top() {
    let mut value = String::from("A note.");
    let (text, frame) = centres(move |ui| {
        ui.add(Textarea::new("notes", &mut value).accessible_name("Notes"));
    });
    assert!(
        text < frame - 20.0,
        "multi-line text starts at the top: {text} vs {frame}"
    );
}
