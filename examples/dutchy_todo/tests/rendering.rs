//! Two image baselines: light at 1x and dark at 2x, with a focused row.

use super::harness_with;
use crate::app::App;
use crate::support::data::FIXED_TODAY;
use dutchy_egui::{Accent, Mode};
use egui::accesskit::Role;
use egui_kittest::{SnapshotResults, kittest::Queryable};

#[test]
fn light_and_dark_baselines_with_a_focused_row() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "dutchy-todo-light-1x"),
        (Accent::Blue, Mode::Dark, 2.0, "dutchy-todo-dark-2x"),
    ] {
        let mut h = harness_with(App::seeded(FIXED_TODAY), accent, mode, scale);
        h.get_by_role_and_label(Role::ListItem, "Plant garlic in the raised bed")
            .focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
