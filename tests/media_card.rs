//! Independent artwork actions, host ownership and native loading states.
mod common;
#[path = "common/media.rs"]
mod media;
use dutchy_egui::{ImageStatus, MediaCard, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::sync::{Arc, Mutex};

struct State {
    probe: media::Shared,
    saved: bool,
    apply_save: bool,
    requested: Option<bool>,
    opens: usize,
    saves: usize,
    retries: usize,
    enabled: bool,
    parent: bool,
    prefix: bool,
    renamed: bool,
    status: Option<ImageStatus>,
    size: egui::Vec2,
}
fn harness() -> Harness<'static, State> {
    let probe = Arc::new(Mutex::new(media::Probe::default()));
    let setup = probe.clone();
    let mut h = Harness::builder()
        .with_size([520.0, 700.0])
        .with_context_setup(move |ctx| {
            common::install(ctx, Theme::default().with_reduced_motion(true));
            media::install(ctx, setup.clone());
        })
        .build_ui_state(
            |ui, state: &mut State| {
                if state.prefix {
                    ui.label("Inserted sibling");
                }
                ui.add_enabled_ui(state.parent, |ui| {
                    let title = if state.renamed {
                        "Renamed artwork"
                    } else {
                        "Canal houses"
                    };
                    let response = MediaCard::new(
                        "stable-card",
                        "fixture://card",
                        "Original canal illustration",
                        title,
                    )
                    .category("Illustration")
                    .description("An original architectural study.")
                    .action_label("Try this style")
                    .saved(state.saved)
                    .width(300.0)
                    .retryable(true)
                    .enabled(state.enabled)
                    .show(ui);
                    state.opens += usize::from(response.open_requested);
                    if let Some(saved) = response.save_requested {
                        state.saves += 1;
                        state.requested = Some(saved);
                        if state.apply_save {
                            state.saved = saved;
                        }
                    }
                    state.retries += usize::from(response.image.retry_requested);
                    state.size = response.image.image.rect.size();
                    state.status = Some(response.image.status);
                });
            },
            State {
                probe,
                saved: false,
                apply_save: false,
                requested: None,
                opens: 0,
                saves: 0,
                retries: 0,
                enabled: true,
                parent: true,
                prefix: false,
                renamed: false,
                status: None,
                size: egui::Vec2::ZERO,
            },
        );
    h.run();
    h
}

#[test]
fn preview_details_and_saved_toggle_are_independent_host_requests() {
    let mut h = harness();
    h.check_accessibility();
    h.get_by_label("Save Canal houses").click();
    h.run();
    assert_eq!(h.state().requested, Some(true));
    assert!(!h.state().saved);
    assert_eq!(h.state().opens, 0);
    assert_eq!(h.state().saves, 1);
    h.state_mut().apply_save = true;
    h.get_by_label("Save Canal houses").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().saved);
    h.run();
    h.get_by_label("Unsave Canal houses").click();
    h.run();
    assert!(!h.state().saved);
    assert_eq!(h.state().opens, 0);
    h.get_by_label("Try this style: Canal houses").click();
    h.run();
    assert_eq!(h.state().opens, 1);
    h.get_by_label("Open Canal houses").focus();
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().opens, 2);
    assert_eq!(h.state().saves, 3);
}

#[test]
fn keyboard_focus_reveals_the_image_action_without_activating_it() {
    let mut h = harness();
    h.get_by_label("Try this style: Canal houses").focus();
    h.run();
    assert!(
        common::galleys(&h.output().shapes)
            .iter()
            .any(|g| g.job.text == "TRY THIS STYLE")
    );
    assert_eq!(h.state().opens, 0);
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().opens, 1);
    h.get_by_role_and_label(Role::Heading, "Canal houses");
}

#[test]
fn retry_is_independent_and_image_status_preserves_its_reserved_geometry() {
    let mut h = harness();
    let size = h.state().size;
    h.state().probe.lock().unwrap().outcome = media::Outcome::Pending;
    h.run();
    assert_eq!(h.state().status, Some(ImageStatus::Loading));
    assert_eq!(h.state().size, size);
    assert!(
        h.get_by_label("Try this style: Canal houses")
            .accesskit_node()
            .data()
            .is_busy()
    );
    h.state().probe.lock().unwrap().outcome = media::Outcome::Failed;
    h.run();
    assert!(matches!(h.state().status, Some(ImageStatus::Failed(_))));
    h.get_by_label("Retry image Original canal illustration")
        .click();
    h.run();
    assert_eq!(h.state().retries, 1);
    assert_eq!(h.state().opens, 0);
    assert_eq!(h.state().saves, 0);
    assert_eq!(h.state().size, size);
}

#[test]
fn identity_survives_renaming_and_insertions_and_disabled_parents_block_actions() {
    let mut h = harness();
    h.get_by_label("Save Canal houses").focus();
    h.run();
    let id = h.ctx.memory(|m| m.focused());
    h.state_mut().prefix = true;
    h.state_mut().renamed = true;
    h.run();
    assert_eq!(h.ctx.memory(|m| m.focused()), id);
    assert!(h.get_by_label("Save Renamed artwork").is_focused());
    for parent in [true, false] {
        h.state_mut().enabled = parent;
        h.state_mut().parent = !parent;
        h.run();
        for label in [
            "Try this style: Renamed artwork",
            "Open Renamed artwork",
            "Save Renamed artwork",
        ] {
            assert!(h.get_by_label(label).accesskit_node().is_disabled());
            h.get_by_label(label).click();
            h.run();
        }
    }
    assert_eq!(h.state().opens, 0);
    assert_eq!(h.state().saves, 0);
}
