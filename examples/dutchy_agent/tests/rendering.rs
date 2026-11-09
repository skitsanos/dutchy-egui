use super::{config, finish, harness};
use crate::{app::App, provider::Scenario};
use dutchy_egui::Mode;
use egui_kittest::SnapshotResults;

#[test]
fn short_code_stays_visible_inside_the_transcript_scroller() {
    let mut app = App::new(config());
    app.chat.current_mut().draft = "Rust".into();
    app.chat.send(&app.config, Scenario::Normal, 0.0).unwrap();
    finish(&mut app);
    let mut h = harness(app, 1160.0, Mode::Light, 1.0);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Copy")
        .scroll_to_me();
    h.run_steps(4);
    fn inspect(shape: &egui::Shape, clip: egui::Rect) -> bool {
        match shape {
            egui::Shape::Text(text) if text.galley.job.text.starts_with("fn main() {") => {
                let bottom = text.pos.y + text.galley.mesh_bounds.max.y;
                assert!(
                    bottom <= clip.bottom() + 0.5,
                    "last code glyph ends at {bottom}, clip ends at {}",
                    clip.bottom()
                );
                true
            }
            egui::Shape::Vec(children) => children.iter().any(|s| inspect(s, clip)),
            _ => false,
        }
    }
    assert!(
        h.output()
            .shapes
            .iter()
            .any(|s| inspect(&s.shape, s.clip_rect))
    );
}
use egui_kittest::kittest::Queryable;

#[test]
fn assistant_markdown_renders_in_both_themes_at_three_scales() {
    let mut snapshots = SnapshotResults::new();
    for (mode, label) in [(Mode::Light, "light"), (Mode::Dark, "dark")] {
        for (scale, suffix) in [(1.0, "1x"), (1.5, "1_5x"), (2.0, "2x")] {
            let mut app = App::new(config());
            app.chat.current_mut().draft = "Show a Rust example".into();
            app.chat.send(&app.config, Scenario::Normal, 0.0).unwrap();
            finish(&mut app);
            let mut h = harness(app, 1160.0, mode, scale);
            h.get_by_role_and_label(egui::accesskit::Role::Button, "Copy")
                .scroll_to_me();
            h.run_steps(3);
            h.check_accessibility();
            h.snapshot(format!("dutchy-agent-{label}-{suffix}"));
            snapshots.extend_harness(&mut h);
        }
    }
    snapshots.unwrap();
}

#[test]
fn narrow_waiting_error_and_settings_surfaces_render() {
    let mut snapshots = SnapshotResults::new();
    for (label, state) in [("waiting", 0), ("error", 1), ("settings", 2)] {
        let mut app = App::new(config());
        if state < 2 {
            app.chat.current_mut().draft = "Show a Rust example".into();
            app.chat
                .send(
                    &app.config,
                    if state == 1 {
                        Scenario::RateLimit
                    } else {
                        Scenario::Normal
                    },
                    0.0,
                )
                .unwrap();
            if state == 1 {
                finish(&mut app);
            }
        } else {
            app.settings_open = true;
        }
        let mut h = harness(app, 560.0, Mode::Dark, 1.0);
        h.check_accessibility();
        h.snapshot(format!("dutchy-agent-narrow-{label}"));
        snapshots.extend_harness(&mut h);
    }
    snapshots.unwrap();
}
