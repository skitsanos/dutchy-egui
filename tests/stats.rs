//! Host formatting, trend semantics, responsive identity, and bounded geometry.

mod common;
use dutchy_egui::{
    Stat, StatSize, StatVariant, StatsGrid, StatsLayout, Theme, Trend, TrendDirection,
    TrendSentiment,
};
use egui::accesskit::Role;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn values_are_host_formatted_and_downward_changes_can_be_positive() {
    let mut h = Harness::builder()
        .with_size([600.0, 500.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            ui.add(
                Stat::new("Error rate", "0.80% / 2,847 requests")
                    .trend(
                        Trend::new("-18%", TrendDirection::Down)
                            .sentiment(TrendSentiment::Positive)
                            .comparison("fewer failed requests"),
                    )
                    .description("Last 30 days"),
            );
            ui.add(Stat::new("Queue", "—").trend(Trend::new("0%", TrendDirection::Flat)));
            let _ = ui.button("Next action");
        });
    let stat = h.get_by_role_and_label(Role::Group, "Error rate");
    assert_eq!(
        stat.accesskit_node().value().as_deref(),
        Some("0.80% / 2,847 requests")
    );
    assert_eq!(
        stat.accesskit_node().description().as_deref(),
        Some("Last 30 days. Down -18%, Positive, fewer failed requests")
    );
    assert!(h.query_by_label("Down -18%").is_some());
    assert_eq!(
        h.get_by_role_and_label(Role::Group, "Queue")
            .accesskit_node()
            .description()
            .as_deref(),
        Some("Unchanged 0%, Neutral")
    );
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(h.get_by_label("Next action").is_focused());
}

struct State {
    width: f32,
    inserted: bool,
}

#[test]
fn metric_identity_survives_grid_reflow_and_inserted_siblings() {
    let mut h = Harness::builder()
        .with_size([1000.0, 800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let _ = ui.button("Keep focus");
                if state.inserted {
                    ui.label("Inserted context");
                }
                ui.scope(|ui| {
                    ui.set_width(state.width);
                    let stats = [
                        Stat::new("Revenue", "$48,290"),
                        Stat::new("Active users", "2,847"),
                        Stat::new("Orders", "1,429"),
                        Stat::new("Conversion", "3.24%"),
                    ];
                    ui.add(
                        StatsGrid::new("performance", &stats)
                            .accessible_name("Monthly performance"),
                    );
                });
            },
            State {
                width: 940.0,
                inserted: false,
            },
        );
    h.get_by_label("Keep focus").focus();
    h.run();
    let id = h
        .get_by_role_and_label(Role::Group, "Active users")
        .accesskit_node()
        .id();
    let before = h.get_by_label("2,847").rect();
    h.state_mut().width = 240.0;
    h.state_mut().inserted = true;
    h.run_steps(2);
    assert_eq!(
        h.get_by_role_and_label(Role::Group, "Active users")
            .accesskit_node()
            .id(),
        id
    );
    let after = h.get_by_label("2,847").rect();
    assert!(after.left() < before.left());
    assert!(after.top() > before.top());
    assert!(h.get_by_label("Keep focus").is_focused());
}

#[test]
fn long_values_and_all_surfaces_fit_narrow_columns_without_style_leaks() {
    let mut h = Harness::builder()
        .with_size([280.0, 1800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let bounds = ui.available_rect_before_wrap();
            let before = ui.style().clone();
            for variant in StatVariant::ALL {
                let response = ui.add(
                    Stat::new(variant.name(), "€12345678901234567890.12")
                        .variant(variant)
                        .size(StatSize::Small)
                        .accented(true),
                );
                assert!(response.rect.right() <= bounds.right() + 0.1);
                assert!(response.rect.is_finite());
            }
            let stats = [
                Stat::new("Long metric name that wraps naturally", "2,847"),
                Stat::new("Orders", "1,429"),
            ];
            for layout in [StatsLayout::Grid, StatsLayout::Compact] {
                let response = ui.add(
                    StatsGrid::new(format!("{layout:?}"), &stats)
                        .layout(layout)
                        .columns(usize::MAX),
                );
                assert!(response.rect.right() <= bounds.right() + 0.1);
            }
            let empty = ui.add(StatsGrid::new("empty", &[]).columns(0));
            assert_eq!(empty.rect.height(), 0.0);
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.run();
}
