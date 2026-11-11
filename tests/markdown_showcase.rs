//! The Markdown showcase page: navigation, accessibility, host-reported links,
//! heading demotion and live source edits through the public example.

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

fn showcase() -> Harness<'static, gallery::Showcase> {
    let mut harness = Harness::builder()
        .with_size([1200.0, 1600.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                reduced_motion: true,
                page: gallery::Page::Markdown,
                ..Default::default()
            }
        });
    harness.run();
    harness
}

#[test]
fn markdown_page_is_reachable_and_accessible() {
    let mut harness = Harness::builder()
        .with_size([1200.0, 940.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                reduced_motion: true,
                ..Default::default()
            }
        });
    harness.run();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Overview")
        .focus();
    harness.run();
    harness.event(egui::Event::Text("markdown".into()));
    harness.run_steps(3);
    assert_eq!(harness.state().page, gallery::Page::Markdown);
    harness.get_by_role_and_label(Role::ListBoxOption, "Markdown");
    harness.check_accessibility();
    harness.get_by_role_and_label(Role::Document, "Release notes");
    std::fs::create_dir_all("target/visual-proof").unwrap();
    harness
        .render()
        .unwrap()
        .save("target/visual-proof/markdown-showcase.png")
        .unwrap();
}

#[test]
fn showcase_reports_links_demotes_headings_and_follows_source_edits() {
    let mut harness = showcase();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::Heading, "Release notes")
            .accesskit_node()
            .level(),
        Some(1)
    );
    harness
        .get_by_role_and_label(Role::Link, "the web design system")
        .click();
    harness.run();
    let links = &harness.state().markdown.links;
    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].href,
        "https://github.com/skitsanos/dutchy-design-system"
    );
    assert!(
        harness
            .output()
            .platform_output
            .commands
            .iter()
            .all(|command| !matches!(command, egui::OutputCommand::OpenUrl(_))),
        "the showcase reports links instead of opening them"
    );
    harness
        .get_by_role_and_label(Role::Switch, "Demote headings one level")
        .click();
    harness.run();
    assert_eq!(
        harness
            .get_by_role_and_label(Role::Heading, "Release notes")
            .accesskit_node()
            .level(),
        Some(2)
    );
    harness.state_mut().markdown.source = "# Edited\n\nFresh text.\n".into();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Edited");
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Release notes")
            .is_none()
    );
    harness.get_by_label("Fresh text.");
}
