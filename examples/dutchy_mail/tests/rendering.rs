use super::{harness_with, newest, row_label};
use crate::mailbox::INBOX;
use dutchy_egui::{Accent, Mode};
use egui::accesskit::Role;
use egui_kittest::{SnapshotResults, kittest::Queryable};

#[test]
fn the_three_panes_render_in_light_and_dark_at_two_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Blue, Mode::Dark, 2.0, "dark-2x"),
    ] {
        let mut h = harness_with(accent, mode, scale);
        let id = newest(&h, INBOX);
        h.get_by_role_and_label(Role::ListBoxOption, &row_label(&h, id))
            .click();
        h.run();
        h.check_accessibility();
        h.snapshot(format!("dutchy-mail-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
