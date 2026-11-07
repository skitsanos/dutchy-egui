//! Host-owned drawing/history, async preview and real export bytes without a picker.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{ScribbleExportOptions, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use gallery::sketch::{jobs, sample};
fn example(width: f32) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size([width, 2400.0])
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Scribble,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn sample_keyboard_history_and_background_preview_use_only_host_data() {
    let mut h = example(1200.0);
    h.run_steps(4);
    h.get_by_label("LOAD SAMPLE DRAWING").click();
    h.run_steps(3);
    let count = h.state().sketch.drawing.strokes().len();
    assert!(count > 0);
    h.get_by_role_and_label(Role::Canvas, "Project drawing")
        .focus();
    h.run_steps(2);
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().sketch.drawing.strokes().len(), count + 1);
    h.get_by_label("Undo Project drawing").click();
    h.run_steps(2);
    assert_eq!(h.state().sketch.drawing.strokes().len(), count);
    h.get_by_label("Redo Project drawing").click();
    h.run_steps(2);
    h.get_by_label("Preview Project drawing").click();
    h.run_steps(2);
    let started = std::time::Instant::now();
    while h.query_by_label("Exported drawing preview").is_none() {
        assert!(started.elapsed().as_secs() < 10);
        std::thread::yield_now();
        h.run_steps(1);
    }
    h.get_by_label("Clear Project drawing").click();
    h.run_steps(2);
    assert!(h.state().sketch.drawing.is_empty());
    h.get_by_label("Undo Project drawing").click();
    h.run_steps(2);
    assert_eq!(h.state().sketch.drawing.strokes().len(), count + 1);
    h.check_accessibility();
}
#[test]
fn host_export_worker_writes_the_selected_snapshot_and_surfaces_io_errors() {
    let ctx = egui::Context::default();
    let drawing = sample::drawing();
    let root = std::path::Path::new("target/visual-proof/scribble-worker");
    std::fs::create_dir_all(root).unwrap();
    let path = root.join("drawing.svg");
    let task = jobs::save(
        ctx.clone(),
        drawing.snapshot(),
        ScribbleExportOptions::default(),
        jobs::Format::Svg,
        path.clone(),
    );
    assert!(matches!(
        task.recv_timeout(std::time::Duration::from_secs(10))
            .unwrap()
            .unwrap(),
        jobs::Product::Saved { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        drawing
            .snapshot()
            .to_svg(ScribbleExportOptions::default())
            .unwrap()
    );
    let bad = jobs::save(
        ctx,
        drawing.snapshot(),
        ScribbleExportOptions::default(),
        jobs::Format::Svg,
        root.to_owned(),
    );
    assert!(
        bad.recv_timeout(std::time::Duration::from_secs(10))
            .unwrap()
            .is_err()
    );
}
#[test]
fn showcase_renders_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for width in [1200.0, 480.0] {
        let mut h = example(width);
        h.run_steps(3);
        h.get_by_label("LOAD SAMPLE DRAWING").click();
        h.run_steps(3);
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/scribble-showcase-{width}.png"))
            .unwrap();
    }
}

#[test]
fn picker_focus_waits_for_the_export_controls_to_be_enabled_again() {
    let mut h = example(1200.0);
    h.state_mut().sketch.drawing = sample::drawing();
    h.run_steps(3);
    let name = "Export SVG Project drawing";
    h.get_by_label(name).focus();
    h.run_steps(2);
    let ctx = h.ctx.clone();
    let opener = ctx.memory(|memory| memory.focused()).unwrap();
    let (sender, task) = std::sync::mpsc::channel();
    h.state_mut().sketch.finish_picker(&ctx, opener, Some(task));
    h.run_steps(3);
    assert!(h.get_by_label(name).accesskit_node().is_disabled());
    assert!(!h.get_by_label(name).is_focused());
    sender
        .send(Ok(jobs::Product::Saved {
            name: "drawing.svg".into(),
            bytes: 123,
        }))
        .unwrap();
    h.run_steps(3);
    assert!(h.get_by_label(name).is_focused());
    assert!(!h.get_by_label(name).accesskit_node().is_disabled());
    let snapshot = h.state().sketch.drawing.snapshot();
    h.get_by_role_and_label(Role::Canvas, "Project drawing")
        .focus();
    h.run_steps(2);
    h.state_mut().sketch.finish_picker(&ctx, opener, None);
    h.run_steps(3);
    assert!(h.get_by_label(name).is_focused());
    assert_eq!(h.state().sketch.drawing.snapshot(), snapshot);
    let (sender, task) = std::sync::mpsc::channel();
    h.state_mut().sketch.finish_picker(&ctx, opener, Some(task));
    h.run_steps(3);
    h.get_by_label("Use accent ink").focus();
    h.run_steps(2);
    sender
        .send(Ok(jobs::Product::Saved {
            name: "drawing.svg".into(),
            bytes: 123,
        }))
        .unwrap();
    h.run_steps(3);
    assert!(h.get_by_label("Use accent ink").is_focused());
}
