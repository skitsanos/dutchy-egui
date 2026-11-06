//! Public showcase state: host validation, asynchronous metadata and simulated transfers.
#![cfg(all(feature = "bundled-fonts", not(target_arch = "wasm32")))]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use egui::accesskit::Role;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
fn harness(size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_step_dt(0.1)
        .build_eframe(|cc| {
            common::install(
                &cc.egui_ctx,
                dutchy_egui::Theme::default().with_reduced_motion(true),
            );
            gallery::Showcase {
                page: gallery::Page::FileUploads,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn simulated_failure_retry_and_completion_are_host_actions() {
    let mut h = harness([1200.0, 2400.0]);
    h.run_steps(3);
    h.get_by_label("ADD SAMPLE FILES").click();
    h.run_steps(2);
    h.get_by_label("Interrupt the next demo transfer once")
        .click();
    h.run_steps(2);
    h.get_by_label("START DEMO UPLOAD").click();
    h.run_steps(22);
    assert!(h.query_by_label("Retry upload project-brief.pdf").is_some());
    h.get_by_label("Retry upload project-brief.pdf").focus();
    h.run_steps(1);
    h.key_press(egui::Key::Enter);
    h.run_steps(40);
    for name in [
        "Upload complete project-brief.pdf",
        "Upload complete canal-photo.jpg",
    ] {
        assert_eq!(
            h.get_by_role_and_label(Role::ProgressIndicator, name)
                .accesskit_node()
                .numeric_value(),
            Some(100.0)
        );
    }
    h.check_accessibility();
}
#[test]
fn cancelling_a_demo_transfer_removes_only_that_host_file() {
    let mut h = harness([1200.0, 2400.0]);
    h.run_steps(3);
    h.get_by_label("ADD SAMPLE FILES").click();
    h.run_steps(2);
    h.get_by_label("START DEMO UPLOAD").click();
    h.run_steps(2);
    h.get_by_label("Cancel upload project-brief.pdf").focus();
    h.run_steps(1);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::ListItem, "project-brief.pdf")
            .is_none()
    );
    assert!(h.get_by_label("Cancel upload canal-photo.jpg").is_focused());
    assert!(h.query_by_label("1 / 5 FILES SELECTED").is_some());
}
#[derive(Debug)]
struct File(PathBuf);
impl egui::DroppedFile for File {
    fn path(&self) -> &Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        panic!("the demo only inspects metadata")
    }
}
#[test]
fn dropped_native_paths_share_type_size_and_count_validation_without_reading_bytes() {
    let dir = PathBuf::from(format!(
        "target/visual-proof/drop-fixture-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("allowed.txt"), b"Local metadata fixture").unwrap();
    std::fs::write(dir.join("rejected.exe"), b"Not executable").unwrap();
    std::fs::File::create(dir.join("oversized.pdf"))
        .unwrap()
        .set_len(10_000_001)
        .unwrap();
    let mut h = harness([1200.0, 2400.0]);
    h.run_steps(3);
    let center = h.get_by_label("Drop project files").rect().center();
    h.hover_at(center);
    h.run_steps(2);
    h.input_mut().dropped_files = ["allowed.txt", "rejected.exe", "oversized.pdf"]
        .map(|name| Arc::new(File(dir.join(name))) as egui::DroppedFileHandle)
        .to_vec();
    let start = Instant::now();
    loop {
        h.run_steps(2);
        if h.query_by_role_and_label(Role::ListItem, "allowed.txt")
            .is_some()
        {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(5));
        thread::sleep(Duration::from_millis(2));
    }
    assert!(
        h.query_by_role_and_label(Role::ListItem, "rejected.exe")
            .is_none()
    );
    assert!(
        h.query_by_role_and_label(Role::ListItem, "oversized.pdf")
            .is_none()
    );
    assert!(h.query_by_label("1 / 5 FILES SELECTED").is_some());
    assert!(
        h.query_by_label_contains("rejected.exe: File type is not accepted")
            .is_some()
    );
    h.check_accessibility();
}
#[test]
fn showcase_is_readable_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [([1200.0, 2000.0], "desktop"), ([480.0, 3000.0], "narrow")] {
        let mut h = harness(size);
        h.run_steps(3);
        h.get_by_label("ADD SAMPLE FILES").click();
        h.run_steps(3);
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/file-showcase-{name}.png"))
            .unwrap();
    }
}
