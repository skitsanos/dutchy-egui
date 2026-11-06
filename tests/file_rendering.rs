//! File surfaces, long names, transfer states and action feedback across themes/scales.
#![cfg(feature = "bundled-fonts")]
mod common;
use dutchy_egui::{
    Accent, FileItem, FileList, FileStatus, FileUpload, FileUploadVariant, Mode, Theme,
};
use egui::accesskit::Role;
use egui_kittest::{
    Harness, SnapshotResults,
    kittest::{By, Queryable},
};
fn harness(theme: Theme, scale: f32, width: f32) -> Harness<'static, ()> {
    Harness::builder().with_size([width,1700.0]).with_pixels_per_point(scale).with_render_cursor(false)
        .with_theme(if theme.mode == Mode::Dark { egui::Theme::Dark } else { egui::Theme::Light })
        .with_context_setup(move |ctx| common::install(ctx,theme.with_reduced_motion(true)))
        .build_ui(|ui| {
            FileUpload::new("drop").label("Drop project files").hint("PDF, PNG, JPEG or TXT · 10 MB per file · 5 files maximum").show(ui);
            ui.add_space(16.0);
            FileUpload::new("compact").variant(FileUploadVariant::Compact).label("Choose files").show(ui);
            ui.add_space(24.0);
            let files = [
                FileItem::new(0,"quarterly_financial_report_with_a_very_long_unbroken_filename_and_a_stable_host_key.pdf").size(2_400_000),
                FileItem::new(1,"canal-photo.jpg").size(840_000).status(FileStatus::Uploading(Some(37.0))),
                FileItem::new(2,"archive.zip").status(FileStatus::Uploading(None)),
                FileItem::new(3,"completed.pdf").size(1024).status(FileStatus::Complete),
                FileItem::new(4,"retry.pdf").size(600).status(FileStatus::Failed("The connection was interrupted. Retry to continue.".into())),
                FileItem::new(5,"locked.txt").size(0).enabled(false),
            ];
            FileList::new("files",&files).show(ui);
        })
}
#[test]
fn file_surfaces_and_transfer_states_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "purple-2x"),
    ] {
        let mut h = harness(Theme::new(accent, mode), scale, 800.0);
        h.run_steps(4);
        h.get_by_label("Drop project files").focus();
        h.run_steps(2);
        h.check_accessibility();
        h.snapshot(format!("file-uploads-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
#[test]
fn narrow_rows_wrap_long_names_and_keep_controls_inside_their_list() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for width in [280.0, 160.0] {
        let mut h = harness(Theme::default(), 1.0, width);
        h.run_steps(4);
        h.check_accessibility();
        for role in [Role::Button, Role::ListItem, Role::ProgressIndicator] {
            for node in h.query_all(By::new().role(role)) {
                let rect = node.rect();
                assert!(
                    rect.left() >= 7.5 && rect.right() <= width - 7.5,
                    "{width}: {rect:?}"
                );
            }
        }
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/file-uploads-{width}.png"))
            .unwrap();
    }
}
#[test]
fn remove_hover_has_visible_feedback_without_changing_selection_or_geometry() {
    let mut h = harness(Theme::default(), 1.0, 800.0);
    h.run_steps(4);
    let name = "Remove file completed.pdf";
    let rect = h.get_by_label(name).rect();
    let before = h.render().unwrap();
    h.get_by_label(name).hover();
    h.run_steps(2);
    assert_eq!(h.get_by_label(name).rect(), rect);
    assert_ne!(h.render().unwrap(), before);
    h.snapshot("file-remove-hover");
}
