//! Controlled expansion, native input, stable body scopes, and accessible regions.
mod common;
use dutchy_egui::{Accordion, AccordionItem, AccordionState, AccordionVariant, Input, Text, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    open: AccordionState<u8>,
    input: String,
    actions: usize,
    rendered: Vec<u8>,
    reverse: bool,
    inserted: bool,
    disabled: bool,
    width: f32,
    variant: AccordionVariant,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: AccordionState::Single(Some(1)),
            input: String::new(),
            actions: 0,
            rendered: Vec::new(),
            reverse: false,
            inserted: false,
            disabled: false,
            width: 600.0,
            variant: AccordionVariant::Default,
        }
    }
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([700.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                ui.set_width(state.width);
                let before = ui.style().clone();
                let spacing = ui.spacing().item_spacing;
                if state.inserted {
                    ui.label("New sibling");
                }
                let mut items = vec![
                    AccordionItem::new(1, "Identity"),
                    AccordionItem::new(2, "Restricted").enabled(false),
                    AccordionItem::new(3, "Preferences"),
                ];
                if state.reverse {
                    items.reverse();
                }
                state.rendered.clear();
                ui.add_enabled_ui(!state.disabled, |ui| {
                    Accordion::new("settings", &mut state.open, &items)
                        .variant(state.variant)
                        .accessible_name("Workspace sections")
                        .show(ui, |ui, key| {
                            assert_eq!(ui.spacing().item_spacing, spacing);
                            state.rendered.push(*key);
                            if *key == 1 {
                                ui.add(
                                    Input::new("name", &mut state.input).accessible_name("Name"),
                                );
                                if ui.button("Apply identity").clicked() {
                                    state.actions += 1;
                                }
                            } else {
                                ui.add(Text::new("Preference content"));
                            }
                        });
                });
                assert_eq!(before.as_ref(), ui.style().as_ref());
            },
            State::default(),
        )
}

#[test]
fn single_and_multiple_expansion_are_host_owned_and_child_actions_do_not_toggle() {
    let mut h = harness();
    assert_eq!(h.state().rendered, [1]);
    let trigger = h.get_by_role_and_label(Role::Button, "Identity");
    assert_eq!(trigger.accesskit_node().data().is_expanded(), Some(true));
    let region = h.get_by_role_and_label(Role::Region, "Identity");
    assert_eq!(
        trigger
            .accesskit_node()
            .controls()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        &[region.accesskit_node().id()]
    );
    assert_eq!(
        region
            .accesskit_node()
            .labelled_by()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        &[trigger.accesskit_node().id()]
    );
    h.get_by_label("Apply identity").click();
    h.run_steps(2);
    assert_eq!(h.state().actions, 1);
    assert!(h.state().open.is_open(&1));
    h.get_by_role_and_label(Role::Button, "Preferences").click();
    h.run_steps(2);
    assert_eq!(h.state().open, AccordionState::Single(Some(3)));
    assert_eq!(h.state().rendered, [3]);
    assert!(h.query_by_label("Name").is_none());
    h.get_by_role_and_label(Role::Button, "Preferences").focus();
    h.run();
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().open, AccordionState::Single(None));
    h.state_mut().open = AccordionState::Multiple(vec![1, 99]);
    h.run();
    h.get_by_role_and_label(Role::Button, "Preferences").click();
    h.run_steps(2);
    assert_eq!(h.state().open, AccordionState::Multiple(vec![1, 99, 3]));
    h.check_accessibility();
}

#[test]
fn keyboard_navigation_skips_disabled_headers_without_changing_expansion() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "Identity").focus();
    h.run_steps(2);
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "Preferences")
            .is_focused()
    );
    assert_eq!(h.state().open, AccordionState::Single(Some(1)));
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().open, AccordionState::Single(Some(3)));
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "Identity")
            .is_focused()
    );
    h.key_press(Key::End);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "Preferences")
            .is_focused()
    );
    h.key_press(Key::Home);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "Identity")
            .is_focused()
    );
}

#[test]
fn body_focus_and_host_edits_survive_reordering_width_and_sibling_changes() {
    let mut h = harness();
    h.get_by_label("Name").focus();
    h.run();
    let id = h.get_by_label("Name").accesskit_node().id();
    h.state_mut().reverse = true;
    h.state_mut().inserted = true;
    h.state_mut().width = 260.0;
    h.state_mut().variant = AccordionVariant::Bordered;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Name").accesskit_node().id(), id);
    assert!(h.get_by_label("Name").is_focused());
    h.get_by_label("Name").type_text("Ada");
    h.run();
    assert_eq!(h.state().input, "Ada");
    assert!(
        h.get_by_role_and_label(Role::Region, "Identity")
            .rect()
            .right()
            <= 268.1
    );
}

#[test]
fn disabled_parents_disable_headers_and_panel_controls_and_empty_groups_are_safe() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Button, "Restricted").click();
    h.run();
    assert_eq!(h.state().open, AccordionState::Single(Some(1)));
    h.state_mut().disabled = true;
    h.run();
    assert!(h.get_by_label("Name").accesskit_node().is_disabled());
    for label in ["Identity", "Preferences"] {
        assert!(
            h.get_by_role_and_label(Role::Button, label)
                .accesskit_node()
                .is_disabled()
        );
        h.get_by_role_and_label(Role::Button, label).click();
        h.run();
    }
    assert_eq!(h.state().open, AccordionState::Single(Some(1)));
    let mut empty = Harness::builder()
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let mut state = AccordionState::<u8>::default();
            let result = Accordion::new("empty", &mut state, &[])
                .variant(AccordionVariant::Bordered)
                .show(ui, |_, _| panic!("empty panel"));
            assert!(result.bodies.is_empty() && result.headers.is_empty());
            assert_eq!(result.response.rect.height(), 0.0);
        });
    empty.run();
}

fn transition_delay(reduced: bool, hidden: bool, settle: bool) -> std::time::Duration {
    let ctx = egui::Context::default();
    common::install(&ctx, Theme::default().with_reduced_motion(reduced));
    let mut delay = std::time::Duration::ZERO;
    for frame in 0..if settle { 40 } else { 10 } {
        let time = f64::from(frame) / 60.0;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 200.0),
                )),
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                if hidden {
                    ui.add_space(400.0);
                }
                let mut state = AccordionState::Single((frame >= 4).then_some(1));
                let items = [AccordionItem::new(1, "Section")];
                Accordion::new("motion", &mut state, &items).show(ui, |ui, _| ui.label("Body"));
            },
        );
        delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
        output.textures_delta.clear();
    }
    delay
}

#[test]
fn chevrons_stop_repainting_when_settled_clipped_or_motion_is_reduced() {
    use std::time::Duration;
    assert!(transition_delay(false, false, false) <= Duration::from_millis(16));
    assert!(transition_delay(true, false, false) > Duration::from_millis(100));
    assert!(transition_delay(false, true, false) > Duration::from_millis(100));
    assert!(transition_delay(false, false, true) > Duration::from_millis(100));
}

#[test]
fn bordered_headers_use_wide_tracking() {
    let mut h = harness();
    h.state_mut().variant = AccordionVariant::Bordered;
    h.run_steps(2);
    let spacing = common::galleys(&h.output().shapes)
        .into_iter()
        .find(|galley| galley.job.text == "IDENTITY")
        .map(|galley| galley.job.sections[0].format.extra_letter_spacing)
        .expect("header painted");
    assert!((spacing - 0.025 * 14.0).abs() < 1e-3, "tracking {spacing}");
}
#[test]
fn the_accent_edge_lights_while_the_open_body_is_hovered() {
    let mut h = harness();
    h.run_steps(2);
    let body = h.get_by_role_and_label(Role::Region, "Identity");
    let center = body.rect().center();
    body.hover();
    h.run_steps(2);
    let primary = Theme::default().palette.primary;
    let lit = h.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, egui::Shape::Rect(shape)
            if (shape.rect.width() - 4.0).abs() < 0.5
                && shape.fill == primary
                && shape.rect.y_range().contains(center.y))
    });
    assert!(lit, "hovering the open body lights the item's accent edge");
}
