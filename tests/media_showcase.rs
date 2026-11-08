//! Public-API host loading/retry and gallery-to-lightbox workflows.
#![cfg(all(feature = "bundled-fonts", feature = "markdown"))]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::Theme;
use egui::accesskit::Role;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
fn example(page: gallery::Page, size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_theme(egui::Theme::Light)
        .build_eframe(move |cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn gallery_opens_the_bound_key_navigation_skips_disabled_art_and_close_restores_the_opener() {
    let mut h = example(gallery::Page::Galleries, [1200.0, 2400.0]);
    h.run_steps(4);
    h.get_by_role_and_label(Role::Group, "Joined artwork")
        .get_by_label("Expand Canal houses")
        .focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(5);
    assert_eq!(h.state().media.selected, Some(0));
    h.key_press(egui::Key::End);
    h.run_steps(3);
    assert_eq!(h.state().media.selected, Some(5));
    h.key_press(egui::Key::ArrowLeft);
    h.run_steps(3);
    assert_eq!(h.state().media.selected, Some(4));
    h.key_press(egui::Key::ArrowLeft);
    h.run_steps(3);
    assert_eq!(h.state().media.selected, Some(2));
    h.key_press(egui::Key::Escape);
    h.run_steps(5);
    assert_eq!(h.state().media.selected, None);
    assert!(
        h.get_by_role_and_label(Role::Group, "Joined artwork")
            .get_by_label("Expand Canal houses")
            .is_focused()
    );
}
#[test]
fn host_source_replacement_clears_busy_state_and_retry_is_an_explicit_host_event() {
    let mut h = example(gallery::Page::Images, [1200.0, 2600.0]);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::Image, "Host-controlled loading artwork")
            .accesskit_node()
            .data()
            .is_busy()
    );
    h.get_by_label("Show loaded image").click();
    h.run_steps(4);
    assert!(h.state().media.loaded);
    assert!(
        !h.get_by_role_and_label(Role::Image, "Host-controlled loading artwork")
            .accesskit_node()
            .data()
            .is_busy()
    );
    h.get_by_label("Retry image Unavailable artwork").click();
    h.run_steps(3);
    assert_eq!(h.state().media.retries, 1);
    assert!(h.state().media.selected.is_none());
}
#[test]
fn media_showcase_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, name) in [
        (gallery::Page::Images, "images"),
        (gallery::Page::Galleries, "galleries"),
        (gallery::Page::Lightboxes, "lightboxes"),
    ] {
        for (size, width) in [([1200.0, 2600.0], "desktop"), ([480.0, 3600.0], "narrow")] {
            let mut h = example(page, size);
            h.run_steps(4);
            h.check_accessibility();
            h.render()
                .unwrap()
                .save(format!("target/visual-proof/{name}-{width}.png"))
                .unwrap();
        }
    }
}
