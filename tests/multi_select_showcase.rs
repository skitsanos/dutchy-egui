//! Showcase applies requested changes to its asset filter.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::Theme;
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn choosing_and_removing_types_updates_the_asset_results() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut gallery::Showcase| state.show(ui),
            gallery::Showcase {
                page: gallery::Page::MultiSelect,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    h.get_by_label("Canal at dusk");
    assert!(h.query_by_label("Studio colour proof").is_none());
    let query = h.get_by_role_and_label(Role::EditableComboBox, "Media types");
    query.focus();
    query.type_text("prn");
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(3);
    h.get_by_label("Studio colour proof");
    h.get_by_label("Canal at dusk");
    h.get_by_label("Remove Photography").click();
    h.run_steps(3);
    assert!(h.query_by_label("Canal at dusk").is_none());
    h.get_by_label("Studio colour proof");
    h.check_accessibility();
}
