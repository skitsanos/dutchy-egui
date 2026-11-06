//! Tooltip timing, accessibility, dismissal, and native control interaction.

mod common;
use dutchy_egui::{Button, Theme, Tooltip, TooltipPosition};
use egui::{Key, Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    open: bool,
    clicks: usize,
    escapes: usize,
    enabled: bool,
    clip: bool,
    position: TooltipPosition,
    anchor: Rect,
    tooltip: Option<Rect>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            clicks: 0,
            escapes: 0,
            enabled: true,
            clip: false,
            position: TooltipPosition::Top,
            anchor: Rect::NOTHING,
            tooltip: None,
        }
    }
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([600.0, 400.0])
        .with_context_setup(|ctx| {
            common::install(ctx, Theme::default());
            ctx.all_styles_mut(|style| {
                style.interaction.tooltip_delay = 0.0;
                style.interaction.show_tooltips_only_when_still = false;
            });
        })
        .build_ui_state(
            |ui, state: &mut State| {
                ui.add_space(120.0);
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    let before = ui.style().clone();
                    let mut response = ui.add_enabled(state.enabled, Button::new("Save"));
                    state.anchor = response.rect;
                    response.ctx.accesskit_node_builder(response.id, |node| {
                        node.set_description("Existing field guidance")
                    });
                    if state.clip {
                        response.interact_rect = Rect::NOTHING;
                    }
                    let result = Tooltip::new("Save your changes")
                        .position(state.position)
                        .show(&response);
                    state.open = result.is_some();
                    state.tooltip = result.map(|r| r.response.rect);
                    if response.clicked() {
                        state.clicks += 1;
                    }
                    assert_eq!(before.as_ref(), ui.style().as_ref());
                });
                let _ = ui.button("Other");
                if ui.input(|input| input.key_pressed(Key::Escape)) {
                    state.escapes += 1;
                }
            },
            State::default(),
        )
}

#[test]
fn keyboard_help_preserves_names_and_descriptions_without_stealing_focus_or_escape() {
    let mut h = harness();
    assert!(!h.state().open);
    let anchor = h.get_by_label("SAVE");
    assert_eq!(
        anchor.accesskit_node().description().as_deref(),
        Some("Existing field guidance\nSave your changes")
    );
    anchor.focus();
    h.run_steps(3);
    assert!(h.state().open);
    assert!(h.get_by_label("SAVE").is_focused());
    h.check_accessibility();
    assert!(h.query_by_role(Role::Tooltip).is_some());
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.state().open);
    // egui itself clears button focus on Escape. The tooltip preserves that policy.
    assert!(!h.get_by_label("SAVE").is_focused());
    assert_eq!(h.state().escapes, 1);
    h.get_by_label("Other").focus();
    h.run_steps(2);
    h.get_by_label("SAVE").focus();
    h.run_steps(3);
    assert!(h.state().open);
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().clicks, 1);
    h.state_mut().clip = true;
    h.run();
    assert!(!h.state().open);
}

#[test]
fn disabled_hover_help_keeps_control_disabled_and_pointer_activation_stays_native() {
    let mut h = harness();
    h.state_mut().enabled = false;
    h.run();
    h.get_by_label("SAVE").hover();
    h.run_steps(3);
    assert!(h.state().open);
    assert!(h.get_by_label("SAVE").accesskit_node().is_disabled());
    h.get_by_label("SAVE").click();
    h.run();
    assert_eq!(h.state().clicks, 0);
    h.state_mut().enabled = true;
    h.run_steps(2);
    h.get_by_label("SAVE").click();
    h.run();
    assert_eq!(h.state().clicks, 1);
}

#[test]
fn escape_holds_hover_hint_closed_until_the_pointer_leaves() {
    let mut h = harness();
    h.get_by_label("SAVE").hover();
    h.run_steps(3);
    assert!(h.state().open);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().open);
    assert_eq!(h.state().escapes, 1);
    h.get_by_label("Other").hover();
    h.run_steps(2);
    h.get_by_label("SAVE").hover();
    h.run_steps(3);
    assert!(h.state().open);
}

#[test]
fn placement_follows_each_preferred_side_and_flips_inside_viewport_edges() {
    for side in [
        TooltipPosition::Top,
        TooltipPosition::Bottom,
        TooltipPosition::Left,
        TooltipPosition::Right,
    ] {
        let mut h = harness();
        h.state_mut().position = side;
        h.get_by_label("SAVE").focus();
        h.run_steps(3);
        let anchor = h.state().anchor;
        let rect = h.state().tooltip.unwrap();
        let expected = match side {
            TooltipPosition::Top => rect.bottom() <= anchor.top(),
            TooltipPosition::Bottom => rect.top() >= anchor.bottom(),
            TooltipPosition::Left => rect.right() <= anchor.left(),
            TooltipPosition::Right => rect.left() >= anchor.right(),
        };
        assert!(expected, "{side:?}: {rect:?} / {anchor:?}");
    }
    let mut h = Harness::builder()
        .with_size([240.0, 180.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, open: &mut Option<Rect>| {
                let response = ui.button("Edge");
                response.request_focus();
                *open = Tooltip::new(
                    "A longer explanation that must wrap within this narrow viewport.",
                )
                .show(&response)
                .map(|r| r.response.rect);
            },
            None,
        );
    h.run_steps(4);
    let rect = h.state().unwrap();
    assert!(
        rect.left() >= 0.0 && rect.right() <= 240.1 && rect.top() >= 0.0 && rect.bottom() <= 180.1,
        "{rect:?}"
    );
}
