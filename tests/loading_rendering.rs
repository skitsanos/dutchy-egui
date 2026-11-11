//! Loading-state rendering and the example's controlled transition to content.
#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{
    Accent, Card, CardPadding, CardVariant, Mode, Progress, ProgressSize, ProgressVariant,
    Skeleton, SkeletonGroup, SkeletonVariant, Text, TextSize, Theme,
};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

fn matrix(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Loading with clarity.").size(TextSize::Xl3));
    for &variant in ProgressVariant::ALL {
        ui.add(
            Progress::new(64.0)
                .label(variant.name())
                .variant(variant)
                .show_value(true),
        );
    }
    ui.add(
        Progress::new(0.0)
            .label("Not started")
            .show_value(true)
            .size(ProgressSize::Small),
    );
    ui.add(
        Progress::new(1200.0)
            .max(1200.0)
            .label("Complete / 1,200 records")
            .show_value(true)
            .size(ProgressSize::Large),
    );
    ui.add(
        Progress::indeterminate()
            .label("Discovering files")
            .show_value(true),
    );
    ui.add_space(16.0);
    Card::new()
        .variant(CardVariant::Dark)
        .padding(CardPadding::Small)
        .show(ui, |ui| {
            ui.add(
                Progress::new(50.0)
                    .label("On inverted surface")
                    .show_value(true),
            );
        });
    ui.add_space(16.0);
    SkeletonGroup::new("shapes", "Loading shape preview").show(ui, |ui| {
        ui.add(Skeleton::new().variant(SkeletonVariant::Title));
        ui.add(Skeleton::new().count(2));
        ui.add(Skeleton::new().variant(SkeletonVariant::Avatar));
        ui.add(Skeleton::new().variant(SkeletonVariant::TableRow));
    });
    ui.add_space(16.0);
    ui.columns(2, |columns| {
        SkeletonGroup::new("card", "Loading card").show_card(&mut columns[0]);
        SkeletonGroup::new("list", "Loading list").show_list(&mut columns[1], 2);
    });
    ui.add_space(16.0);
    SkeletonGroup::new("table", "Loading table").show_table(ui, 2);
    ui.add_space(16.0);
    SkeletonGroup::new("content", "Loading article").show_content(ui);
}

#[test]
fn loading_states_render_at_multiple_scales_and_themes() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "loading-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "loading-dark-1_5x"),
        (Accent::Crimson, Mode::Light, 2.0, "loading-crimson-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([760.0, 1600.0])
            .with_render_cursor(false)
            .with_pixels_per_point(scale)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(matrix);
        h.run();
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn loading_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Progress, "progress"),
        (gallery::Page::Skeletons, "skeletons"),
    ] {
        for (size, suffix) in [([1200.0, 1800.0], "desktop"), ([480.0, 3000.0], "narrow")] {
            let mut h = Harness::builder()
                .with_size(size)
                .with_theme(egui::Theme::Light)
                .build_eframe(|cc| {
                    common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                    gallery::Showcase {
                        page,
                        reduced_motion: true,
                        ..Default::default()
                    }
                });
            h.run();
            h.check_accessibility();
            h.render()
                .unwrap()
                .save(format!("target/visual-proof/{prefix}-{suffix}.png"))
                .unwrap();
        }
    }
}

#[test]
fn example_progress_and_content_transitions_remain_application_controlled() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1800.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Progress,
                reduced_motion: true,
                ..Default::default()
            }
        });
    h.get_by_label("Add 10").click();
    h.run();
    assert_eq!(h.state().loading.value, 74.0);
    for _ in 0..3 {
        h.get_by_label("Add 10").click();
        h.run();
    }
    assert_eq!(h.state().loading.value, 100.0);
    h.get_by_label("Reset progress").click();
    h.run();
    assert_eq!(h.state().loading.value, 0.0);
    h.get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Progress")
        .focus();
    h.run();
    h.event(egui::Event::Text("skeletons".into()));
    h.run_steps(3);
    assert_eq!(h.state().page, gallery::Page::Skeletons);
    assert!(h.query_by_label("Loading workspace").is_some());
    let before = h.get_by_label("02 / Shape primitives").rect();
    h.get_by_label("Show loaded content").click();
    h.run();
    assert!(h.state().loading.ready);
    assert!(h.query_by_label("Loading workspace").is_none());
    assert_eq!(h.get_by_label("02 / Shape primitives").rect(), before);
    h.get_by_label("Open workspace").click();
    h.run();
    assert_eq!(h.state().loading.opens, 1);
    h.get_by_label("Show placeholders").click();
    h.run();
    assert!(!h.state().loading.ready);
    assert_eq!(h.get_by_label("02 / Shape primitives").rect(), before);
}
