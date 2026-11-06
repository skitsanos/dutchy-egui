//! Keyed responsive thumbnails, native expansion and lazy keyboard reachability.
mod common;
#[path = "common/media.rs"]
mod media;
use dutchy_egui::{Gallery, GalleryItem, ImageStatus, Theme};
use egui::{Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::sync::{Arc, Mutex};
struct State {
    keys: Vec<u8>,
    requests: Vec<u8>,
    disabled: bool,
    renamed: bool,
    rects: Vec<(u8, Rect, ImageStatus)>,
    probe: media::Shared,
}
impl Default for State {
    fn default() -> Self {
        Self {
            keys: (1..=6).collect(),
            requests: Vec::new(),
            disabled: false,
            renamed: false,
            rects: Vec::new(),
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
                let items: Vec<_> = state
                    .keys
                    .iter()
                    .map(|key| {
                        GalleryItem::new(
                            *key,
                            format!("fixture://item-{key}"),
                            format!("{} {key}", if state.renamed { "Renamed" } else { "Photo" }),
                        )
                        .enabled(*key != 3)
                    })
                    .collect();
                egui::ScrollArea::vertical()
                    .max_height(500.0)
                    .show(ui, |ui| {
                        let result = Gallery::new("pictures", &items)
                            .accessible_name("Portfolio")
                            .enabled(!state.disabled)
                            .show(ui);
                        if let Some(key) = result.open_requested {
                            state.requests.push(key);
                        }
                        state.rects = result
                            .images
                            .into_iter()
                            .map(|(key, image)| (key, image.image.rect, image.status))
                            .collect();
                    });
            },
            state,
        )
}
#[test]
fn thumbnails_return_keys_without_owning_selection_and_disabled_controls_cannot_open() {
    let mut h = harness([600.0, 700.0]);
    h.run_steps(4);
    h.get_by_role_and_label(Role::Image, "Photo 1").click();
    h.run_steps(2);
    assert!(h.state().requests.is_empty());
    h.get_by_label("Expand Photo 1").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Space);
    h.run_steps(3);
    assert_eq!(h.state().requests, [1]);
    assert!(
        h.get_by_label("Expand Photo 3")
            .accesskit_node()
            .data()
            .is_disabled()
    );
    h.get_by_label("Expand Photo 3").click();
    h.run_steps(2);
    assert_eq!(h.state().requests, [1]);
    h.state_mut().disabled = true;
    h.run_steps(3);
    h.get_by_label("Expand Photo 2").click();
    h.run_steps(2);
    assert_eq!(h.state().requests, [1]);
    h.check_accessibility();
}
#[test]
fn item_controls_keep_identity_and_focus_through_reordering_renaming_and_responsive_reflow() {
    let mut h = harness([600.0, 700.0]);
    h.run_steps(3);
    h.get_by_label("Expand Photo 2").focus();
    h.run_steps(2);
    let id = h.get_by_label("Expand Photo 2").accesskit_node().id();
    h.state_mut().keys.reverse();
    h.state_mut().renamed = true;
    h.set_size(egui::vec2(300.0, 700.0));
    h.run_steps(6);
    let button = h.get_by_label("Expand Renamed 2");
    assert_eq!(button.accesskit_node().id(), id);
    assert!(button.is_focused());
    assert!(button.rect().bottom() <= 508.5);
    let rows = &h.state().rects;
    assert!(
        rows.windows(2)
            .all(|pair| pair[0].1.top() < pair[1].1.top())
    );
}
#[test]
fn keyboard_reveals_a_lazy_offscreen_uri_item_and_loads_its_preview() {
    let mut h = harness([300.0, 700.0]);
    h.run_steps(4);
    assert_eq!(h.state().rects.last().unwrap().2, ImageStatus::Loading);
    h.get_by_label("Expand Photo 6").focus();
    h.run_steps(6);
    assert!(h.get_by_label("Expand Photo 6").rect().bottom() <= 508.5);
    assert_eq!(h.state().rects.last().unwrap().2, ImageStatus::Ready);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().requests, [6]);
}
#[test]
fn empty_collections_show_guidance_and_pending_or_failed_images_remain_keyboard_reachable() {
    let mut h = harness([600.0, 700.0]);
    h.state().probe.lock().unwrap().outcome = media::Outcome::Pending;
    h.run_steps(3);
    h.get_by_label("Expand Photo 1").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().requests, [1]);
    h.state().probe.lock().unwrap().outcome = media::Outcome::Failed;
    h.run_steps(3);
    h.get_by_label("Expand Photo 2").click();
    h.run_steps(2);
    assert_eq!(h.state().requests, [1, 2]);
    h.state_mut().keys.clear();
    h.run_steps(3);
    assert!(h.query_by_label("NO IMAGES").is_some());
    assert!(h.query_by_role(Role::Image).is_none());
    assert!(
        h.query_by_role_and_label(Role::Group, "Portfolio")
            .is_some()
    );
    h.check_accessibility();
}
