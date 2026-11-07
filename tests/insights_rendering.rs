//! Stats surfaces/trends and empty-state wrapping at multiple display scales.
#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{
    Accent, EmptyState, EmptyStateVariant, Mode, Stat, StatVariant, StatsGrid, StatsLayout, Text,
    TextSize, Theme, Trend, TrendDirection, TrendSentiment,
};
use egui::accesskit::Role;
use egui_kittest::{
    Harness, SnapshotResults,
    kittest::{NodeT, Queryable},
};

fn matrix(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Meaning at a glance.").size(TextSize::Xl3));
    let stats = [
        Stat::new("Revenue", "$48,290")
            .trend(Trend::new("+12.5%", TrendDirection::Up).sentiment(TrendSentiment::Positive)),
        Stat::new("Error rate", "0.8%")
            .variant(StatVariant::Muted)
            .trend(Trend::new("-18%", TrendDirection::Down).sentiment(TrendSentiment::Positive)),
        Stat::new("Latency", "240ms")
            .variant(StatVariant::Dark)
            .trend(Trend::new("+8.1%", TrendDirection::Up).sentiment(TrendSentiment::Negative)),
        Stat::new("Active users", "2,847")
            .variant(StatVariant::Primary)
            .trend(Trend::new("0%", TrendDirection::Flat)),
    ];
    ui.add(
        StatsGrid::new("surfaces", &stats)
            .columns(2)
            .accessible_name("Sample performance"),
    );
    ui.add_space(16.0);
    let compact = [
        Stat::new("Users", "2,847"),
        Stat::new("Views", "14,209"),
        Stat::new("Uptime", "99.9%"),
        Stat::new("Load time", "3.2s"),
    ];
    ui.add(StatsGrid::new("compact", &compact).layout(StatsLayout::Compact));
    ui.add_space(16.0);
    ui.add(
        Stat::new("Monthly revenue", "$48,290")
            .accented(true)
            .description("From $42,924 last month")
            .trend(Trend::new("+12.5%", TrendDirection::Up).sentiment(TrendSentiment::Positive)),
    );
    ui.add_space(16.0);
    EmptyState::new("search", "No results found")
        .variant(EmptyStateVariant::Search)
        .description("We could not find a match. Try another term or clear the current filters.")
        .action("Clear search")
        .secondary_action("Reset filters")
        .show(ui);
    EmptyState::new("dashboard", "Set up your dashboard")
        .variant(EmptyStateVariant::Dashboard)
        .description("Add a widget to monitor a key metric, or start with a template.")
        .action("Add widget")
        .secondary_action("Browse templates")
        .show(ui);
}

#[test]
fn insights_render_across_surfaces_modes_and_display_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "insights-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "insights-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "insights-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([800.0, 1600.0])
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
fn insight_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Stats, "stats"),
        (gallery::Page::EmptyStates, "empty-states"),
    ] {
        for (size, suffix) in [([1200.0, 2200.0], "desktop"), ([480.0, 3000.0], "narrow")] {
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
fn example_refreshes_metrics_and_host_actions_replace_empty_content() {
    let mut h = Harness::builder()
        .with_size([1200.0, 2800.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Stats,
                reduced_motion: true,
                ..Default::default()
            }
        });
    h.get_by_label("Refresh sample metrics").click();
    h.run();
    assert_eq!(h.state().insights.refreshes, 1);
    assert_eq!(
        h.get_by_role_and_label(Role::Group, "Revenue")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("$52,460")
    );
    h.get_by_label("Empty states").click();
    h.run();
    h.get_by_label("Browse project templates").click();
    h.run();
    assert_eq!(
        (h.state().insights.templates, h.state().insights.projects),
        (1, 0)
    );
    h.get_by_label("Create project").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().insights.projects, 1);
    assert!(
        h.query_by_role_and_label(Role::Status, "No projects yet")
            .is_none()
    );
    assert!(h.get_by_label("Reset projects").is_focused());
    h.get_by_label("Clear search").click();
    h.run_steps(2);
    assert!(!h.state().insights.filters_active);
    assert!(h.get_by_label("Apply sample filters").is_focused());
    h.get_by_label("Add entry").click();
    h.run_steps(2);
    assert_eq!(h.state().insights.entries, 1);
    h.get_by_label("Add widget").click();
    h.run_steps(2);
    assert_eq!(h.state().insights.widgets, 1);
    assert!(h.get_by_label("Reset dashboard").is_focused());
    h.get_by_label("Reset projects").click();
    h.run();
    assert_eq!(h.state().insights.projects, 0);
    assert!(
        h.query_by_role_and_label(Role::Status, "No projects yet")
            .is_some()
    );
}
