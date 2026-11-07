//! Native modality, keyed navigation, dismissal, focus and collection changes.
mod common;
#[path = "common/media.rs"]
mod media;
use dutchy_egui::{GalleryItem, Lightbox, OverlayDismissal, OverlayOptions, Theme};
use egui::{Key, Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::sync::{Arc, Mutex};
struct State {
    keys: Vec<u8>,
    selected: Option<u8>,
    wrap: bool,
    options: OverlayOptions,
    hide_opener: bool,
    surface: Option<Rect>,
    outside: usize,
    dismissed: Vec<OverlayDismissal>,
    probe: media::Shared,
}
impl Default for State {
    fn default() -> Self {
        Self {
            keys: (1..=4).collect(),
            selected: None,
            wrap: false,
            options: OverlayOptions::default(),
            hide_opener: false,
            surface: None,
            outside: 0,
            dismissed: Vec::new(),
            probe: Arc::new(Mutex::new(media::Probe::default())),
        }
    }
}
fn harness(size: [f32; 2]) -> Harness<'static, State> {
    let state = State::default();
    let probe = state.probe.clone();
    Harness::builder()
        .with_size(size)
        .with_context_setup(move |ctx| {
            common::install(ctx, Theme::default().with_reduced_motion(true));
            media::install(ctx, probe.clone());
        })
        .build_ui_state(
            |ui, state: &mut State| {
                let opener = if state.hide_opener {
                    None
                } else {
                    Some(ui.button("Open viewer"))
                };
                if opener.as_ref().is_some_and(egui::Response::clicked) {
                    state.selected = Some(1);
                }
                if ui.button("Outside action").clicked() {
                    state.outside += 1;
                }
                let fallback = ui.button("Focus fallback");
                let items: Vec<_> = state
                    .keys
                    .iter()
                    .map(|key| {
                        GalleryItem::new(
                            *key,
                            format!("fixture://image-{key}"),
                            format!("Photo {key}"),
                        )
                        .caption(format!("Caption for image {key}"))
                        .caption_source("Fixture")
                        .enabled(*key != 3)
                    })
                    .collect();
                let mut options = state.options.clone();
                if state.hide_opener {
                    options.return_focus = Some(fallback.id);
                }
                if let Some(result) = Lightbox::new("viewer", &items, &mut state.selected)
                    .options(options)
                    .wrap_navigation(state.wrap)
                    .show(ui.ctx())
                {
                    state.surface = Some(result.response.rect);
                    if let Some(dismissed) = result.dismissed {
                        state.dismissed.push(dismissed);
                    }
                }
            },
            state,
        )
}
fn open(h: &mut Harness<'_, State>) {
    h.get_by_label("Open viewer").focus();
    h.run_steps(2);
    h.key_press(Key::Enter);
    h.run_steps(5);
}
#[test]
fn named_dialog_confines_tab_and_escape_restores_the_native_opener() {
    let mut h = harness([900.0, 700.0]);
    open(&mut h);
    assert!(
        h.get_by_role_and_label(Role::Dialog, "Image viewer")
            .accesskit_node()
            .data()
            .is_modal()
    );
    assert!(h.get_by_label("Close Image viewer").is_focused());
    for _ in 0..5 {
        h.key_press(Key::Tab);
        h.run_steps(2);
        assert!(!h.get_by_label("Outside action").is_focused());
    }
    h.key_press(Key::Escape);
    h.run_steps(5);
    assert!(h.state().selected.is_none());
    assert_eq!(h.state().dismissed, [OverlayDismissal::Escape]);
    assert!(h.get_by_label("Open viewer").is_focused());
    h.check_accessibility();
}
#[test]
fn native_buttons_and_arrow_home_end_navigation_skip_disabled_items_and_keep_control_focus() {
    let mut h = harness([900.0, 700.0]);
    open(&mut h);
    assert!(
        h.get_by_label("Previous image")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    h.get_by_label("Next image").focus();
    h.run_steps(2);
    let id = h.get_by_label("Next image").accesskit_node().id();
    h.key_press(Key::Space);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(2));
    assert_eq!(h.get_by_label("Next image").accesskit_node().id(), id);
    assert!(h.get_by_label("Next image").is_focused());
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(4));
    assert!(
        h.get_by_label("Next image")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(4));
    h.key_press(Key::Home);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(1));
    h.key_press(Key::End);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(4));
    h.state_mut().wrap = true;
    h.run_steps(3);
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(1));
    h.key_press(Key::ArrowLeft);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(4));
}
#[test]
fn backdrop_blocks_underlying_actions_and_image_surface_does_not_dismiss() {
    let mut h = harness([900.0, 700.0]);
    h.state_mut().options.dismiss_on_backdrop = false;
    open(&mut h);
    h.get_by_label("Outside action").click();
    h.run_steps(3);
    assert_eq!(h.state().outside, 0);
    assert!(h.state().selected.is_some());
    h.get_by_role_and_label(Role::Image, "Photo 1").click();
    h.run_steps(2);
    assert!(h.state().selected.is_some());
    h.state_mut().options.dismiss_on_backdrop = true;
    h.run_steps(2);
    // A press in the viewport gutter lands on the backdrop, not on the surface.
    h.drag_at(egui::pos2(12.0, 12.0));
    h.run_steps(1);
    h.drop_at(egui::pos2(12.0, 12.0));
    h.run_steps(4);
    assert!(h.state().selected.is_none());
    assert_eq!(h.state().outside, 0);
    assert_eq!(h.state().dismissed, [OverlayDismissal::Backdrop]);
}
#[test]
fn closing_or_host_removal_restores_a_surviving_replacement_target() {
    let mut h = harness([900.0, 700.0]);
    open(&mut h);
    h.state_mut().hide_opener = true;
    h.run_steps(3);
    h.get_by_label("Close Image viewer").focus();
    h.run_steps(2);
    h.key_press(Key::Space);
    h.run_steps(5);
    assert_eq!(h.state().dismissed, [OverlayDismissal::CloseButton]);
    assert!(h.get_by_label("Focus fallback").is_focused());
    h.state_mut().selected = Some(2);
    h.run_steps(5);
    h.state_mut().selected = None;
    h.run_steps(5);
    assert!(h.get_by_label("Focus fallback").is_focused());
}
#[test]
fn reordering_preserves_the_key_and_removed_or_disabled_selection_stays_explicit() {
    let mut h = harness([900.0, 700.0]);
    open(&mut h);
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    h.state_mut().keys.reverse();
    h.run_steps(4);
    assert_eq!(h.state().selected, Some(2));
    assert!(h.query_by_role_and_label(Role::Image, "Photo 2").is_some());
    h.state_mut().keys.retain(|key| *key != 2);
    h.run_steps(4);
    assert_eq!(h.state().selected, Some(2));
    assert!(h.query_by_role(Role::Image).is_none());
    assert!(h.query_by_label("Close Image viewer").is_some());
    h.state_mut().selected = Some(3);
    h.run_steps(3);
    assert!(h.query_by_role(Role::Image).is_none());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(h.state().selected.is_none());
}
#[test]
fn loading_failure_and_ready_replace_only_the_image_with_close_and_navigation_retained() {
    let mut h = harness([900.0, 700.0]);
    h.state().probe.lock().unwrap().outcome = media::Outcome::Pending;
    open(&mut h);
    assert!(
        h.get_by_role_and_label(Role::Image, "Photo 1")
            .accesskit_node()
            .data()
            .is_busy()
    );
    let close = h.get_by_label("Close Image viewer").accesskit_node().id();
    h.state().probe.lock().unwrap().outcome = media::Outcome::Failed;
    h.run_steps(3);
    assert!(h.query_by_label("Retry image Photo 1").is_some());
    assert_eq!(
        h.get_by_label("Close Image viewer").accesskit_node().id(),
        close
    );
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(2));
    h.state().probe.lock().unwrap().outcome = media::Outcome::Ready;
    h.run_steps(3);
    assert!(h.query_by_label("Retry image Photo 2").is_none());
    assert!(h.query_by_role_and_label(Role::Image, "Photo 2").is_some());
}
#[test]
fn narrow_viewer_bounds_and_controls_stay_in_the_viewport() {
    let mut h = harness([280.0, 480.0]);
    open(&mut h);
    let r = h.state().surface.unwrap();
    assert!(
        r.left() >= 23.5 && r.right() <= 256.5 && r.top() >= 23.5 && r.bottom() <= 456.5,
        "{r:?}"
    );
    for name in ["Close Image viewer", "Previous image", "Next image"] {
        let r = h.get_by_label(name).rect();
        assert!(
            r.left() >= 23.5 && r.right() <= 256.5 && r.top() >= 23.5 && r.bottom() <= 456.5,
            "{name}: {r:?}"
        );
    }
    h.check_accessibility();
}

#[test]
fn hidden_close_uses_a_focusable_heading_and_respects_escape_policy() {
    let mut h = harness([900.0, 700.0]);
    h.state_mut().options.show_close_button = false;
    h.state_mut().options.dismiss_on_escape = false;
    open(&mut h);
    assert!(h.query_by_label("Close Image viewer").is_none());
    assert!(
        h.get_by_role_and_label(Role::Heading, "Image viewer")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(1));
    h.state_mut().options.dismiss_on_escape = true;
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(h.state().selected.is_none());
    assert!(h.get_by_label("Open viewer").is_focused());
}

#[test]
fn singleton_has_no_navigation_and_an_empty_collection_keeps_a_closable_missing_selection() {
    let mut h = harness([900.0, 700.0]);
    h.state_mut().keys = vec![1];
    open(&mut h);
    assert!(h.query_by_label("Previous image").is_none());
    assert!(h.query_by_label("Next image").is_none());
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(1));
    h.state_mut().keys.clear();
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(1));
    assert!(h.query_by_role(Role::Image).is_none());
    assert!(h.query_by_label("Close Image viewer").is_some());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(h.state().selected.is_none());
}
