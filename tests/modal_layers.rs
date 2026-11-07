//! Palettes and overlays that lose the top modal layer settle instead of spinning.
mod common;
use dutchy_egui::{CommandPalette, MenuItem, Modal, SearchState, Theme};
use egui::{Id, Key, Modifiers, ViewportId};
use egui_kittest::{Harness, kittest::Queryable};
use std::time::Duration;

#[derive(Default)]
struct State {
    search: SearchState,
    dialog: bool,
    host_modal: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([800.0, 600.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                if ui.button("Open commands").clicked() {
                    state.search.open = true;
                }
                let _ = ui.button("Outside action");
                Modal::new("editor", &mut state.dialog, "Edit record").show(ui.ctx(), |ui| {
                    ui.label("Edit the record.");
                });
                let items = [
                    MenuItem::new(1, "Dashboard"),
                    MenuItem::new(2, "Create project"),
                ];
                CommandPalette::new("commands", &mut state.search, &items).show(ui.ctx());
                if state.host_modal {
                    egui::Modal::new(Id::unique("host-modal")).show(ui.ctx(), |ui| {
                        ui.label("Host dialog");
                        if ui.button("Close host dialog").clicked() {
                            state.host_modal = false;
                        }
                    });
                }
            },
            State::default(),
        )
}
fn repaint_delay(h: &Harness<'_, State>) -> Duration {
    h.output().viewport_output[&ViewportId::ROOT].repaint_delay
}
#[test]
fn closed_palette_does_not_spin_or_steal_focus_behind_a_host_modal() {
    let mut h = harness();
    h.get_by_label("Open commands").focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    assert!(h.state().search.open);
    h.state_mut().search.open = false;
    h.state_mut().host_modal = true;
    h.run_steps(8);
    assert!(
        repaint_delay(&h) > Duration::from_secs(1),
        "palette must not request repaints while another modal is on top"
    );
    h.get_by_label("Close host dialog").click();
    h.run_steps(6);
    assert!(
        !h.get_by_label("Open commands").is_focused(),
        "palette must not move focus after an unrelated modal closes"
    );
}
#[test]
fn overlay_opened_beneath_a_host_modal_waits_without_spinning_and_then_enters() {
    let mut h = harness();
    h.run_steps(2);
    // Both open in one pass; the host modal is shown later, so its area is newer and on top.
    h.state_mut().dialog = true;
    h.state_mut().host_modal = true;
    h.run_steps(8);
    assert!(
        repaint_delay(&h) > Duration::from_secs(1),
        "overlay must not request repaints while another modal is on top"
    );
    h.get_by_label("Close host dialog").click();
    h.run_steps(6);
    assert!(
        h.get_by_label("Close Edit record").is_focused(),
        "overlay receives entry focus once it becomes the top modal"
    );
}
