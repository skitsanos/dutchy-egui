//! Document flow owns its column; filled sub-surfaces own their ink.
#![cfg(feature = "markdown")]
mod common;
use dutchy_egui::{Card, CardVariant, Markdown, Theme};
use egui::{Align, Layout, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn document_blocks_stay_vertical_and_ragged_inside_justified_horizontal_hosts() {
    let mut h = Harness::builder().with_size([420.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            ui.with_layout(Layout::left_to_right(Align::Center).with_main_justify(true), |ui| {
                Markdown::new("doc", "# Heading\n\nA paragraph long enough to wrap inside this column and remain ragged.\n\nSecond paragraph.").show(ui);
            });
        });
    h.run();
    let heading = h.get_by_role_and_label(Role::Heading, "Heading").rect();
    let last = h
        .get_by_role_and_label(Role::Paragraph, "Second paragraph.")
        .rect();
    assert!(last.top() > heading.bottom());
    assert!((heading.left() - last.left()).abs() < 0.5);
    assert!(
        common::galleys(&h.output().shapes)
            .iter()
            .all(|g| !g.job.justify)
    );
}

#[test]
fn table_headers_own_muted_surface_ink_while_document_body_inherits_inverse_ink() {
    let mut h = Harness::builder()
        .with_size([500.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            Card::new().variant(CardVariant::Dark).show(ui, |ui| {
                Markdown::new(
                    "doc",
                    "Body on inverse.\n\n| Header |\n| --- |\n| Value |\n",
                )
                .show(ui);
            });
        });
    h.run();
    let galleys = common::galleys(&h.output().shapes);
    let ink = |text: &str| {
        galleys
            .iter()
            .find(|g| g.job.text == text)
            .unwrap()
            .job
            .sections[0]
            .format
            .color
    };
    assert_eq!(ink("Body on inverse."), Theme::default().palette.background);
    assert_eq!(ink("HEADER"), Theme::default().palette.foreground);
}
