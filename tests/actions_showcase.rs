//! Public controls composed into discoverable application-owned showcase pages.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::Theme;
use egui::{
    Key,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn page(page: gallery::Page, width: f32) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size([width, 1100.0])
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
fn icon_and_toggle_pages_update_only_application_values() {
    let mut h = page(gallery::Page::IconButtons, 1200.0);
    h.run();
    h.get_by_role_and_label(Role::Button, "Use Save tool")
        .click();
    h.run();
    assert_eq!(h.state().actions.actions, 1);
    assert_eq!(h.state().actions.last_icon, "Save");
    h.state_mut().page = gallery::Page::ToggleButtons;
    h.run();
    h.get_by_role_and_label(Role::Button, "Pin workspace")
        .click();
    h.run();
    assert!(h.state().actions.pinned);
    h.key_press(Key::Space);
    h.run();
    assert!(!h.state().actions.pinned);
}

#[test]
fn toolbar_preview_applies_formatting_and_action_requests() {
    let mut h = page(gallery::Page::Toolbars, 1200.0);
    h.run();
    // The first bar is the full-width document formatting example.
    let bold = h
        .get_all_by_role_and_label(Role::Button, "Bold")
        .next()
        .unwrap();
    bold.focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    assert!(!h.state().actions.italic);
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().actions.italic);
    assert_eq!(
        h.get_all_by_role_and_label(Role::Button, "Italic")
            .next()
            .unwrap()
            .accesskit_node()
            .data()
            .toggled(),
        Some(Toggled::True)
    );
    h.key_press(Key::End);
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().actions.saves, 1);
    let bar = h
        .get_by_role_and_label(Role::Toolbar, "Document formatting")
        .rect();
    for (label, align) in [
        ("Align center", egui::Align::Center),
        ("Align right", egui::Align::RIGHT),
    ] {
        h.get_all_by_role_and_label(Role::Button, label)
            .next()
            .unwrap()
            .click();
        h.run();
        let preview = h.get_by_label("Sharp tools. Your state.").rect();
        let expected = align.align_size_within_range(preview.width(), bar.x_range());
        assert!(
            (preview.left() - expected.min).abs() < 1.0,
            "{label}: {preview:?} / {bar:?}"
        );
    }
}

#[test]
fn all_new_pages_render_at_narrow_widths_without_a_horizontal_page_scroll() {
    for page_id in [
        gallery::Page::IconButtons,
        gallery::Page::ToggleButtons,
        gallery::Page::Toolbars,
    ] {
        let mut h = page(page_id, 480.0);
        h.run();
        h.check_accessibility();
        for label in [
            "Small targets. Clear actions.",
            "Press. Keep. Release.",
            "A place for your tools.",
        ] {
            if let Some(node) = h.query_by_label(label) {
                assert!(node.rect().right() <= 480.0);
            }
        }
    }
}
