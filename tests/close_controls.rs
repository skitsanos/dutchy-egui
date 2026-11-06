//! Shared close affordances and alert corner alignment.
mod common;
use dutchy_egui::{Alert, ControlSize, Theme, tokens::surface};
use egui::Rect;
use egui_kittest::{Harness, kittest::Queryable};

#[derive(Default)]
struct Geometry {
    alert: Option<Rect>,
    dismissals: usize,
}
#[test]
fn alert_dismiss_has_equal_top_and_right_padding_and_a_real_hit_target() {
    for width in [280.0, 760.0] {
        for banner in [false, true] {
            let mut h = Harness::builder()
                .with_size([width, 500.0])
                .with_context_setup(|ctx| common::install(ctx, Theme::default()))
                .build_ui_state(
                    move |ui, state: &mut Geometry| {
                        let result =
                            Alert::new("padding", "Review the connection and retry when ready.")
                                .title("Connection interrupted")
                                .dismissible(true)
                                .banner(banner)
                                .action("Retry")
                                .show(ui);
                        state.alert = Some(result.response.rect);
                        state.dismissals += usize::from(result.dismissed);
                    },
                    Geometry::default(),
                );
            h.run_steps(2);
            let alert = h.state().alert.unwrap();
            let close = h.get_by_label("Dismiss Connection interrupted").rect();
            let top = close.top() - alert.top();
            let right = alert.right() - close.right();
            assert_eq!(close.size(), egui::Vec2::splat(ControlSize::Small.height()));
            assert!(
                (top - surface::ALERT_PADDING).abs() < 0.5,
                "top={top} right={right}"
            );
            assert!((right - top).abs() < 0.5, "top={top} right={right}");
            // A pointer press near the target's corner must activate, not only its glyph.
            let corner = close.min + egui::vec2(2.0, 2.0);
            h.hover_at(corner);
            h.run_steps(2);
            h.drag_at(corner);
            h.run_steps(2);
            assert_eq!(h.state().dismissals, 0);
            h.drop_at(corner);
            h.run_steps(2);
            assert_eq!(h.state().dismissals, 1);
            h.check_accessibility();
        }
    }
}

#[cfg(feature = "bundled-fonts")]
mod rendering {
    use super::*;
    use dutchy_egui::{Accent, AlertVariant, Badge, BadgeVariant, Mode, Tab, Tabs, Text};
    use egui_kittest::{SnapshotResults, kittest::NodeT};

    struct State {
        selected: Option<u8>,
        actions: usize,
        disabled: bool,
    }
    impl Default for State {
        fn default() -> Self {
            Self {
                selected: Some(1),
                actions: 0,
                disabled: false,
            }
        }
    }
    fn matrix(ui: &mut egui::Ui, state: &mut State) {
        let p = Theme::from_context(ui.ctx()).palette;
        ui.painter().rect_filled(ui.max_rect(), 0, p.background);
        ui.add(Text::heading("Small actions. Clear feedback."));
        ui.add_space(16.0);
        ui.add_enabled_ui(!state.disabled, |ui| {
            let result = Alert::new("alert", "Your work stays with the host application.")
                .variant(AlertVariant::Error)
                .title("Connection interrupted")
                .dismissible(true)
                .action("Retry connection")
                .show(ui);
            state.actions += usize::from(result.dismissed || result.action_clicked);
            ui.add_space(16.0);
            let result = Alert::new("banner", "A new component release is available.")
                .title("Library update")
                .banner(true)
                .dismissible(true)
                .show(ui);
            state.actions += usize::from(result.dismissed);
            ui.add_space(24.0);
            let items = [
                Tab::new(1, "Document one").closable(true),
                Tab::new(2, "Document two").closable(true),
                Tab::new(3, "Unavailable").closable(true).enabled(false),
            ];
            let result = Tabs::new("documents", &mut state.selected, &items).show(ui, |ui, _| {
                ui.add(Text::new("Close requests do not switch the selected tab."));
            });
            state.actions += usize::from(result.close_requested.is_some());
            ui.add_space(24.0);
            ui.horizontal_wrapped(|ui| {
                for variant in BadgeVariant::ALL {
                    state.actions += usize::from(
                        ui.add(
                            Badge::new(format!("{} tag", variant.name()))
                                .variant(variant)
                                .closable(true),
                        )
                        .clicked(),
                    );
                }
            });
        });
    }
    fn harness(theme: Theme, scale: f32) -> Harness<'static, State> {
        Harness::builder()
            .with_size([720.0, 720.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if theme.mode == Mode::Dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            })
            .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
            .build_ui_state(matrix, State::default())
    }
    fn targets() -> [(&'static str, &'static str); 4] {
        [
            ("Dismiss Connection interrupted", "alert"),
            ("Dismiss Library update", "banner"),
            ("Close Document two", "tab"),
            ("Dismiss Primary tag", "tag"),
        ]
    }
    #[test]
    fn hover_and_press_are_visible_local_and_do_not_activate_early() {
        let mut snapshots = SnapshotResults::new();
        for (theme, scale, prefix) in [
            (
                Theme::new(Accent::Orange, Mode::Light),
                1.0,
                "close-light-1x",
            ),
            (
                Theme::new(Accent::Orange, Mode::Dark),
                1.5,
                "close-dark-1_5x",
            ),
            (
                Theme::new(Accent::Purple, Mode::Light),
                2.0,
                "close-purple-2x",
            ),
        ] {
            for (name, kind) in targets() {
                let mut h = harness(theme, scale);
                h.run_steps(2);
                let idle = h.render().unwrap();
                let rect = h.get_by_label(name).rect();
                h.get_by_label(name).hover();
                h.run_steps(2);
                let hover = h.render().unwrap();
                let mut changed = 0;
                for (x, y, pixel) in idle.enumerate_pixels() {
                    if pixel != hover.get_pixel(x, y) {
                        changed += 1;
                        let point = egui::pos2(x as f32 / scale, y as f32 / scale);
                        assert!(
                            rect.expand(1.0).contains(point),
                            "{kind}: hover changed outside the close control at {point:?}"
                        );
                    }
                }
                assert!(changed > 0, "{kind}: hover must be visible");
                assert_eq!(h.state().actions, 0);
                assert_eq!(h.state().selected, Some(1));
                assert_eq!(
                    h.output().platform_output.cursor_icon,
                    egui::CursorIcon::PointingHand
                );
                h.snapshot(format!("{prefix}-{kind}-hover"));
                h.drag_at(rect.center());
                h.run_steps(2);
                let pressed = h.render().unwrap();
                assert_ne!(
                    hover, pressed,
                    "{kind}: pressed feedback must differ from hover"
                );
                assert_eq!(h.state().actions, 0);
                h.drop_at(rect.center());
                h.run_steps(2);
                assert_eq!(h.state().actions, 1);
                assert_eq!(h.state().selected, Some(1));
                h.check_accessibility();
                snapshots.extend_harness(&mut h);
            }
        }
        snapshots.unwrap();
    }
    #[test]
    fn disabled_close_controls_do_not_highlight_or_activate() {
        for name in targets()
            .map(|(name, _)| name)
            .into_iter()
            .chain(["Close Unavailable"])
        {
            let mut h = harness(Theme::default(), 1.0);
            h.state_mut().disabled = true;
            h.run_steps(2);
            let before = h.render().unwrap();
            assert!(h.get_by_label(name).accesskit_node().is_disabled());
            h.get_by_label(name).hover();
            h.run_steps(2);
            assert_eq!(before, h.render().unwrap());
            h.get_by_label(name).click();
            h.run_steps(2);
            assert_eq!(h.state().actions, 0);
        }
    }
}
