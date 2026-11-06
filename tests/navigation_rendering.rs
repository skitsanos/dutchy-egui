//! Hierarchy and page navigation surfaces, focus, reflow and host workflows.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{
    Accent, BreadcrumbItem, BreadcrumbSeparator, Breadcrumbs, Mode, Pagination, PaginationVariant,
    Text, Theme,
};
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
#[derive(Default)]
struct State {
    open: bool,
}
fn matrix(ui: &mut egui::Ui, state: &mut State) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Orient. Navigate."));
    let items = [
        BreadcrumbItem::new(1, "Home"),
        BreadcrumbItem::new(2, "Library"),
        BreadcrumbItem::new(3, "Components"),
        BreadcrumbItem::new(4, "Navigation"),
    ];
    Breadcrumbs::new("slash", &items)
        .separator(BreadcrumbSeparator::Slash)
        .accessible_name("Standard path")
        .show(ui);
    ui.add_space(16.0);
    Breadcrumbs::new("bar", &items)
        .bar(true)
        .accessible_name("Bar path")
        .show(ui);
    ui.add_space(16.0);
    Breadcrumbs::new("collapsed", &items)
        .collapse(2, &mut state.open)
        .accessible_name("Collapsed path")
        .show(ui);
    ui.add_space(24.0);
    Pagination::new("small", 1, 5)
        .accessible_name("Short pages")
        .show(ui);
    ui.add_space(16.0);
    Pagination::new("window", 7, 20)
        .accessible_name("Long pages")
        .show(ui);
    ui.add_space(16.0);
    Pagination::new("compact", 3, 12)
        .variant(PaginationVariant::Compact)
        .accessible_name("Compact pages")
        .show(ui);
    ui.add_space(16.0);
    Pagination::new("empty", 0, 0)
        .accessible_name("Empty pages")
        .show(ui);
    ui.add_space(16.0);
    Pagination::new("disabled", 2, 5)
        .enabled(false)
        .accessible_name("Disabled pages")
        .show(ui);
}
#[test]
fn navigation_renders_across_themes_scales_focus_and_collapsed_menus() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, prefix) in [
        (Accent::Orange, Mode::Light, 1.0, "navigation-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "navigation-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "navigation-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([720.0, 740.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(matrix, State::default());
        h.get_by_role_and_label(Role::Navigation, "Long pages")
            .get_by_label("Page 8")
            .focus();
        h.run_steps(2);
        h.check_accessibility();
        h.snapshot(format!("{prefix}-focus"));
        h.get_by_label("Show collapsed ancestors").focus();
        h.run_steps(2);
        h.key_press(Key::ArrowDown);
        h.run_steps(4);
        h.check_accessibility();
        h.snapshot(format!("{prefix}-menu"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
fn example(page: gallery::Page, size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn navigation_showcase_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Breadcrumbs, "breadcrumbs"),
        (gallery::Page::Pagination, "pagination"),
    ] {
        for (size, suffix) in [([1200.0, 2000.0], "desktop"), ([480.0, 2800.0], "narrow")] {
            let mut h = example(page, size);
            h.run_steps(3);
            h.check_accessibility();
            h.render()
                .unwrap()
                .save(format!("target/visual-proof/{prefix}-{suffix}.png"))
                .unwrap();
        }
    }
}
#[test]
fn showcase_hierarchy_and_record_browser_apply_only_requested_navigation() {
    let mut h = example(gallery::Page::Breadcrumbs, [1200.0, 2000.0]);
    h.get_by_label("Show collapsed ancestors").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Projects").click();
    h.run_steps(3);
    assert_eq!(h.state().navigation.depth, 4);
    assert_eq!(h.state().navigation.destination, Some(4));
    assert!(h.get_by_label("RESTORE DEEP PATH").is_focused());
    h.get_by_label("RESTORE DEEP PATH").click();
    h.run_steps(2);
    assert_eq!(h.state().navigation.depth, 6);
    h.state_mut().page = gallery::Page::Pagination;
    h.run_steps(3);
    h.get_by_role_and_label(Role::Navigation, "Record pages")
        .get_by_label("Page 2")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().navigation.page, 2);
    assert_eq!(h.state().navigation.requests, 1);
    assert!(h.query_by_label("Project 06 / sample record").is_some());
    assert!(h.query_by_label("Project 01 / sample record").is_none());
    assert!(
        h.get_by_role_and_label(Role::Navigation, "Record pages")
            .get_by_label("Page 2")
            .is_focused()
    );
}
#[test]
fn matrix_wraps_inside_a_narrow_column() {
    let mut h = Harness::builder()
        .with_size([260.0, 1500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(matrix, State::default());
    h.run_steps(3);
    h.check_accessibility();
    h.render()
        .unwrap()
        .save("target/visual-proof/navigation-narrow.png")
        .unwrap();
    for name in ["Short pages", "Long pages", "Compact pages"] {
        let region = h.get_by_role_and_label(Role::Navigation, name);
        assert!(
            region.rect().right() <= 252.5,
            "{name}: {:?}",
            region.rect()
        );
    }
}
