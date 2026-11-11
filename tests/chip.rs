//! Chip request isolation, focus, and rendering across feature profiles.
mod common;

use dutchy_egui::{Accent, Chip, ControlSize, Mode, Theme};
use egui::{
    Key, Rect, Ui,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const LABEL: &str = "Project status awaiting final review";
const REMOVE: &str = "Remove Project status awaiting final review";

struct State {
    selected: Option<bool>,
    enabled: bool,
    parent_enabled: bool,
    prefix: bool,
    toggles: Vec<bool>,
    removals: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected: Some(false),
            enabled: true,
            parent_enabled: true,
            prefix: false,
            toggles: Vec::new(),
            removals: 0,
        }
    }
}

fn show(ui: &mut Ui, state: &mut State) {
    let _ = ui.button("Before chip");
    if state.prefix {
        ui.label("Inserted sibling");
    }
    ui.add_enabled_ui(state.parent_enabled, |ui| {
        ui.horizontal_wrapped(|ui| {
            let mut chip = Chip::new("status", LABEL)
                .removable(true)
                .enabled(state.enabled);
            if let Some(selected) = state.selected {
                chip = chip.selected(selected);
            }
            let result = chip.show(ui);
            state.toggles.extend(result.toggle_requested);
            state.removals += usize::from(result.remove_requested);
        });
    });
}

fn harness(state: State, theme: Theme, scale: f32, width: f32) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([width, 180.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(show, state);
    h.run_steps(3);
    h
}

#[test]
fn independent_pointer_keyboard_and_accesskit_actions_leave_selection_with_the_host() {
    let mut h = harness(State::default(), Theme::default(), 1.0, 420.0);
    h.get_by_role_and_label(Role::Button, LABEL).click();
    h.run_steps(2);
    assert_eq!(h.state().toggles, [true]);
    assert_eq!(h.state().removals, 0);
    assert_eq!(
        h.get_by_label(LABEL).accesskit_node().toggled(),
        Some(Toggled::False)
    );

    h.state_mut().selected = Some(true);
    h.get_by_label("Before chip").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label(LABEL).is_focused());
    assert_eq!(
        h.get_by_label(LABEL).accesskit_node().toggled(),
        Some(Toggled::True)
    );
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().toggles, [true, false]);
    assert_eq!(h.state().removals, 0);
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_role_and_label(Role::Button, REMOVE).is_focused());
    h.key_press(Key::Enter);
    h.run_steps(2);
    h.get_by_label(REMOVE).click_accesskit();
    h.run_steps(2);
    h.get_by_label(REMOVE).click();
    h.run_steps(2);
    assert_eq!(h.state().toggles, [true, false]);
    assert_eq!(h.state().removals, 3);
    h.check_accessibility();
}

#[test]
fn a_removable_label_has_only_one_tab_stop_and_its_body_is_passive() {
    let mut h = harness(
        State {
            selected: None,
            ..Default::default()
        },
        Theme::default(),
        1.0,
        420.0,
    );
    h.get_by_role_and_label(Role::Label, LABEL).click();
    h.run();
    assert!(h.state().toggles.is_empty());
    assert_eq!(h.state().removals, 0);
    h.get_by_label("Before chip").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label(REMOVE).is_focused());
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().removals, 1);
    assert!(h.state().toggles.is_empty());
}

#[test]
fn direct_and_inherited_disable_cancel_a_pending_click_and_block_assistive_actions() {
    for (enabled, parent_enabled) in [(false, true), (true, false)] {
        let mut h = harness(State::default(), Theme::default(), 1.0, 420.0);
        let point = h.get_by_label(REMOVE).rect().center();
        h.hover_at(point);
        h.run();
        h.drag_at(point);
        h.run();
        assert_eq!(h.state().removals, 0);

        h.state_mut().enabled = enabled;
        h.state_mut().parent_enabled = parent_enabled;
        h.drop_at(point);
        h.run();
        for label in [LABEL, REMOVE] {
            let node = h.get_by_role_and_label(Role::Button, label);
            assert!(node.accesskit_node().is_disabled());
            node.click_accesskit();
        }
        h.run_steps(2);
        assert!(h.state().toggles.is_empty());
        assert_eq!(h.state().removals, 0);
    }
}

#[test]
fn narrow_reflow_keeps_both_keys_and_focus() {
    let mut h = harness(State::default(), Theme::default(), 1.0, 420.0);
    let ids = [LABEL, REMOVE].map(|label| h.get_by_label(label).accesskit_node().id());
    h.get_by_label(REMOVE).focus();
    h.run();
    h.state_mut().prefix = true;
    h.set_size(egui::vec2(132.0, 180.0));
    h.run_steps(3);
    assert_eq!(
        ids,
        [LABEL, REMOVE].map(|label| h.get_by_label(label).accesskit_node().id())
    );
    assert!(h.get_by_label(REMOVE).is_focused());
    let body = h.get_by_label(LABEL).rect();
    let remove = h.get_by_label(REMOVE).rect();
    assert_eq!(body.right(), remove.left());
    assert!(remove.right() <= 132.0);
    assert_eq!(
        remove.size(),
        egui::Vec2::splat(ControlSize::Small.height())
    );
}

#[test]
fn hover_paint_is_local_to_each_segment_in_light_and_dark() {
    for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
        let theme = Theme::new(Accent::Orange, mode);
        let mut h = harness(State::default(), theme, scale, 245.0);
        for selected in [false, true] {
            h.state_mut().selected = Some(selected);
            h.hover_at(egui::pos2(230.0, 150.0));
            h.run_steps(2);
            let idle = h.render().unwrap();
            let rect = h.get_by_label(LABEL).rect();
            let at = (rect.min + egui::vec2(5.0, 5.0)) * scale;
            let fill = if selected {
                theme.palette.foreground
            } else {
                theme.palette.background
            };
            assert_eq!(idle.get_pixel(at.x as u32, at.y as u32).0, fill.to_array());
            for label in [LABEL, REMOVE] {
                let node = h.get_by_label(label);
                let bounds: Rect = node.rect().expand(1.0);
                node.hover();
                h.run_steps(2);
                let hover = h.render().unwrap();
                let changes = idle
                    .enumerate_pixels()
                    .filter(|&(x, y, pixel)| {
                        if pixel == hover.get_pixel(x, y) {
                            return false;
                        }
                        let point = egui::pos2(x as f32 / scale, y as f32 / scale);
                        assert!(
                            bounds.contains(point),
                            "{label}: hover changed pixels at {point:?}"
                        );
                        true
                    })
                    .count();
                assert!(
                    changes > 0,
                    "{label} hover must be visible in {mode:?}, selected={selected}"
                );
            }
        }
    }
}
