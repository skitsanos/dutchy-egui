//! Host selection, group semantics and responsive Chip layout.
mod common;

use dutchy_egui::{Accent, FilterChipGroup, FilterChipItem, Mode, Theme, tokens::space};
use egui::{
    Align, Key, Layout, Rect, Ui, Vec2,
    accesskit::{Role, Toggled},
    pos2, vec2,
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::collections::BTreeSet;

struct State {
    items: Vec<FilterChipItem<'static, u32>>,
    selected: BTreeSet<u32>,
    requests: Vec<(u32, bool)>,
    enabled: bool,
    parent_enabled: bool,
    prefix: bool,
    align_right: bool,
    spacing: Option<Vec2>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            items: vec![
                FilterChipItem::new(1, "Architecture"),
                FilterChipItem::new(2, "Research").enabled(false),
                FilterChipItem::new(3, "Photography"),
            ],
            selected: BTreeSet::from([3, 99]),
            requests: Vec::new(),
            enabled: true,
            parent_enabled: true,
            prefix: false,
            align_right: false,
            spacing: None,
        }
    }
}

fn show(ui: &mut Ui, state: &mut State) {
    let _ = ui.button("Before filters");
    if state.prefix {
        ui.label("New sibling");
    }
    if state.align_right {
        ui.with_layout(Layout::top_down(Align::Max), |ui| filters(ui, state));
    } else {
        filters(ui, state);
    }
    let _ = ui.button("After filters");
}

fn filters(ui: &mut Ui, state: &mut State) {
    ui.add_enabled_ui(state.parent_enabled, |ui| {
        if let Some(spacing) = state.spacing {
            ui.spacing_mut().item_spacing = spacing;
        }
        let result = FilterChipGroup::new("formats", &state.items)
            .accessible_name("Formats")
            .enabled(state.enabled)
            .show(ui, |key| state.selected.contains(key));
        state.requests.extend(
            result
                .requests
                .into_iter()
                .map(|request| (request.key, request.selected)),
        );
    });
}

const WIDE: [&str; 5] = ["Drawings", "Photography", "Prints", "Sculpture", "Video"];

fn wide_harness(state: impl FnOnce(&mut State)) -> Harness<'static, State> {
    let mut h = harness(Theme::default(), 1.0, 260.0);
    h.state_mut().items = WIDE
        .into_iter()
        .zip(10..)
        .map(|(label, key)| FilterChipItem::new(key, label))
        .collect();
    state(h.state_mut());
    h.run_steps(3);
    h
}

/// Each chip follows its predecessor in the same row or starts the next row.
fn assert_reading_order(h: &Harness<'_, State>, right_to_left: bool) {
    let rects: Vec<Rect> = WIDE
        .iter()
        .map(|label| h.get_by_label(label).rect())
        .collect();
    let start = |rect: Rect| {
        if right_to_left {
            rect.right()
        } else {
            rect.left()
        }
    };
    let (mut beside, mut wrapped) = (0, 0);
    for pair in rects.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        let gap = if b.top() == a.top() {
            beside += 1;
            if right_to_left {
                a.left() - b.right()
            } else {
                b.left() - a.right()
            }
        } else {
            wrapped += 1;
            assert_eq!(start(b), start(rects[0]), "{b:?} does not start a row");
            b.top() - a.bottom()
        };
        assert!((gap - space::SM).abs() < 0.01, "{a:?} then {b:?}");
    }
    assert!(beside > 0 && wrapped > 0, "{rects:?}");
}

fn harness(theme: Theme, scale: f32, width: f32) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([width, 240.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(show, State::default());
    h.run_steps(3);
    h
}

#[test]
fn named_group_emits_typed_requests_and_the_host_can_decline_them() {
    let mut h = harness(Theme::default(), 1.0, 500.0);
    let group = h.get_by_role_and_label(Role::Group, "Formats");
    let first = group.get_by_role_and_label(Role::Button, "Architecture");
    assert_eq!(first.accesskit_node().toggled(), Some(Toggled::False));
    first.click();
    h.run_steps(2);
    assert_eq!(h.state().requests, [(1, true)]);
    assert_eq!(
        h.get_by_label("Architecture").accesskit_node().toggled(),
        Some(Toggled::False)
    );

    h.get_by_label("Photography").click_accesskit();
    h.run_steps(2);
    assert_eq!(h.state().requests, [(1, true), (3, false)]);
    assert_eq!(h.state().selected, BTreeSet::from([3, 99]));
    h.state_mut().selected.insert(1);
    h.run_steps(2);
    assert_eq!(
        h.get_by_label("Architecture").accesskit_node().toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        h.get_by_label("Photography").accesskit_node().toggled(),
        Some(Toggled::True)
    );
    assert_eq!(h.state().requests.len(), 2);
    h.check_accessibility();
}

#[test]
fn native_tab_order_skips_disabled_choices_and_activates_the_focused_key() {
    let mut h = harness(Theme::default(), 1.0, 500.0);
    h.get_by_label("Before filters").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("Architecture").is_focused());
    h.key_press(Key::Space);
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("Photography").is_focused());
    h.key_press(Key::Enter);
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("After filters").is_focused());
    assert_eq!(h.state().requests, [(1, true), (3, false)]);
}

#[test]
fn disabled_choices_and_groups_block_input_in_the_frame_they_are_disabled() {
    let mut h = harness(Theme::default(), 1.0, 500.0);
    let disabled = h.get_by_label("Research");
    assert!(disabled.accesskit_node().is_disabled());
    disabled.click_accesskit();
    h.run();
    assert!(h.state().requests.is_empty());

    for (enabled, parent_enabled) in [(false, true), (true, false)] {
        let mut h = harness(Theme::default(), 1.0, 500.0);
        h.get_by_label("Architecture").focus();
        h.run();
        assert!(h.get_by_label("Architecture").is_focused());
        h.state_mut().enabled = enabled;
        h.state_mut().parent_enabled = parent_enabled;
        h.key_press(Key::Space);
        h.run();
        assert!(
            h.get_by_role_and_label(Role::Group, "Formats")
                .accesskit_node()
                .is_disabled()
        );
        let first = h.get_by_label("Architecture");
        assert!(first.accesskit_node().is_disabled());
        first.click_accesskit();
        h.run();
        assert!(h.state().requests.is_empty());
        assert_eq!(
            h.get_by_label("Photography").accesskit_node().toggled(),
            Some(Toggled::True)
        );
    }
}

#[test]
fn keyed_focus_survives_reorder_renaming_and_wrapping() {
    let mut h = harness(Theme::default(), 1.0, 500.0);
    let chip = h.get_by_label("Photography");
    let id = chip.accesskit_node().id();
    chip.focus();
    h.run();

    h.state_mut().items[2] = FilterChipItem::new(3, "Photos");
    h.state_mut().items.rotate_right(1);
    h.state_mut().prefix = true;
    h.set_size(vec2(155.0, 240.0));
    h.run_steps(3);
    let renamed = h.get_by_label("Photos");
    assert_eq!(renamed.accesskit_node().id(), id);
    assert!(renamed.is_focused());
    assert!(renamed.rect().bottom() < h.get_by_label("Architecture").rect().top());
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().requests, [(3, false)]);
}

#[test]
fn wrapped_rows_keep_item_order_from_either_edge() {
    for align_right in [false, true] {
        let h = wide_harness(|state| state.align_right = align_right);
        assert_reading_order(&h, align_right);
    }
}

#[test]
fn chip_gaps_come_from_tokens_inside_custom_parent_spacing() {
    for spacing in [Vec2::ZERO, Vec2::splat(space::XXL)] {
        let h = wide_harness(|state| state.spacing = Some(spacing));
        assert_reading_order(&h, false);
    }
}

#[test]
fn wrapped_rows_preserve_selected_and_unselected_fills_across_themes() {
    for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
        let theme = Theme::new(Accent::Orange, mode);
        let mut h = harness(theme, scale, 155.0);
        let first = h.get_by_label("Architecture").rect();
        let last = h.get_by_label("Photography").rect();
        let before = h.get_by_label("Before filters").rect();
        assert_eq!(first.top() - before.bottom(), space::SM);
        assert!(first.bottom() < last.top());
        assert!(last.right() <= 155.0);
        assert!(last.bottom() < h.get_by_label("After filters").rect().top());
        let image = h.render().unwrap();
        for (rect, fill) in [
            (first, theme.palette.background),
            (last, theme.palette.foreground),
        ] {
            let point = rect.min + vec2(5.0, 5.0);
            let pixel = pos2(point.x * scale, point.y * scale);
            assert_eq!(
                image.get_pixel(pixel.x as u32, pixel.y as u32).0,
                fill.to_array()
            );
        }
    }
}
