//! Severity surfaces, viewport constraints, hover/focus and public-API host flows.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{
    Accent, Mode, Text, Theme, Toast, ToastAppearance, ToastPosition, ToastQueue, ToastVariant,
    Toasts,
};
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
struct Matrix {
    solid: ToastQueue<u8>,
    outline: ToastQueue<u8>,
}
impl Default for Matrix {
    fn default() -> Self {
        let mut solid = ToastQueue::new();
        let mut outline = ToastQueue::new();
        for (variant, title, message) in [
            (
                ToastVariant::Info,
                "Heads up",
                "A new version is available.",
            ),
            (ToastVariant::Success, "Saved", "Your changes are saved."),
            (
                ToastVariant::Warning,
                "Session reminder with a longer heading",
                "Save your work before leaving.",
            ),
            (
                ToastVariant::Error,
                "Sync failed",
                "The host decides how to retry.",
            ),
        ] {
            let toast = Toast::new(message)
                .title(title)
                .variant(variant)
                .timeout(None)
                .action("Review details", 1);
            solid.push(toast.clone());
            outline.push(toast.appearance(ToastAppearance::Outline));
        }
        Self { solid, outline }
    }
}
fn matrix(theme: Theme, scale: f32) -> Harness<'static, Matrix> {
    Harness::builder()
        .with_size([900.0, 1150.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(if theme.mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui_state(
            |ui, state: &mut Matrix| {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0,
                    Theme::from_context(ui.ctx()).palette.background,
                );
                ui.add_space(1040.0);
                ui.add(Text::label("Nonblocking feedback / solid and outlined"));
                Toasts::new("solid", &mut state.solid)
                    .accessible_name("Solid notifications")
                    .position(ToastPosition::TopLeft)
                    .max_visible(4)
                    .show(ui.ctx());
                Toasts::new("outline", &mut state.outline)
                    .accessible_name("Outlined notifications")
                    .position(ToastPosition::TopRight)
                    .max_visible(4)
                    .show(ui.ctx());
            },
            Matrix::default(),
        )
}
#[test]
fn severity_surfaces_render_across_themes_scales_and_keyboard_focus() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "toasts-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "toasts-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "toasts-purple-2x"),
    ] {
        let mut h = matrix(Theme::new(accent, mode).with_reduced_motion(true), scale);
        h.run_steps(5);

        h.get_by_role_and_label(Role::Group, "Solid notifications")
            .get_by_role_and_label(Role::Status, "Information: Heads up")
            .get_by_label("Dismiss Heads up")
            .focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
#[test]
fn close_hover_has_flat_feedback_without_triggering_dismissal() {
    let mut h = matrix(Theme::default().with_reduced_motion(true), 1.0);
    h.run_steps(5);
    let before = h.render().unwrap();
    h.get_by_role_and_label(Role::Group, "Solid notifications")
        .get_by_label("Dismiss Heads up")
        .hover();
    h.run_steps(2);
    assert_ne!(before, h.render().unwrap());
    assert_eq!(h.state().solid.len() + h.state().outline.len(), 8);
    h.snapshot("toasts-close-hover");
}
fn example(size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Toasts,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn showcase_undo_and_retry_execute_in_the_host_once_and_keep_data() {
    let mut h = example([1200.0, 1300.0]);
    h.get_by_label("ARCHIVE SAMPLE ITEM").click();
    h.run_steps(5);
    assert_eq!(h.state().notifications.archives, 1);
    h.get_by_label("Undo archive").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert_eq!(h.state().notifications.archives, 0);
    assert!(h.state().notifications.queue.is_empty());
    h.get_by_label("NOTIFY ERROR").click();
    h.run_steps(4);
    h.get_by_label("Retry sync").click();
    h.run_steps(4);
    assert_eq!(h.state().notifications.retries, 1);
    assert!(h.state().notifications.queue.is_empty());
    h.run_steps(4);
    assert_eq!(h.state().notifications.retries, 1);
}
#[test]
fn showcase_and_long_notifications_render_with_narrow_scrolling() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [([1200.0, 1500.0], "desktop"), ([480.0, 1800.0], "narrow")] {
        let mut h = example(size);
        h.state_mut().notifications.persistent = true;
        h.run_steps(4);
        h.get_by_label("NOTIFY WARNING").click();
        h.run_steps(4);
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/toasts-{name}.png"))
            .unwrap();
        h.check_accessibility();
    }
}
