//! Roving focus, controlled requests, overflow and stable toolbar identities.
mod common;
use dutchy_egui::{
    Icon, IconButton, Theme, Toolbar, ToolbarEvent, ToolbarItem, ToolbarOrientation,
};
use egui::{
    Id, Key, Modifiers, Rect,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    order: Vec<u8>,
    on: bool,
    apply: bool,
    events: Vec<ToolbarEvent<u8>>,
    disabled: bool,
    busy: bool,
    parent: bool,
    enabled: bool,
    prefix: bool,
    renamed: bool,
    orientation: ToolbarOrientation,
    width: f32,
    wrap: bool,
    after: Option<Id>,
    viewport: Rect,
}
fn harness() -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([520.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                ui.add(IconButton::new("before", "Before toolbar", Icon::Plus).tooltip(""));
                if s.prefix {
                    ui.label("Inserted sibling");
                }
                ui.add_enabled_ui(s.parent, |ui| {
                    let items: Vec<_> = s
                        .order
                        .iter()
                        .map(|value| {
                            match value {
                                0 => ToolbarItem::action(
                                    0,
                                    if s.renamed { "Save document" } else { "Save" },
                                )
                                .icon(Icon::Save)
                                .enabled(!s.disabled)
                                .loading(s.busy),
                                1 => ToolbarItem::action(1, "Unavailable")
                                    .icon(Icon::Trash)
                                    .enabled(false),
                                2 => ToolbarItem::toggle(2, "Bold", s.on)
                                    .icon(Icon::Bold)
                                    .separator_before(true),
                                _ => ToolbarItem::action(3, "More").icon(Icon::More),
                            }
                            .tooltip("")
                        })
                        .collect();
                    let mut toolbar = Toolbar::new("tools", &items)
                        .accessible_name("Document tools")
                        .orientation(s.orientation)
                        .width(s.width)
                        .wrap(s.wrap)
                        .enabled(s.enabled);
                    if let Some(id) = s.after {
                        toolbar = toolbar.focus_when_empty(id);
                    }
                    let out = toolbar.show(ui);
                    s.viewport = out.viewport;
                    if let Some(event) = out.event {
                        if s.apply
                            && let ToolbarEvent::Toggled { pressed, .. } = event
                        {
                            s.on = pressed;
                        }
                        s.events.push(event);
                    }
                });
                s.after = Some(
                    ui.add(IconButton::new("after", "After toolbar", Icon::Check).tooltip(""))
                        .id,
                );
            },
            State {
                order: vec![0, 1, 2, 3],
                on: false,
                apply: false,
                events: Vec::new(),
                disabled: false,
                busy: false,
                parent: true,
                enabled: true,
                prefix: false,
                renamed: false,
                orientation: ToolbarOrientation::Horizontal,
                width: 240.0,
                wrap: false,
                after: None,
                viewport: Rect::NOTHING,
            },
        );
    h.run();
    h
}

#[test]
fn tab_has_one_remembered_entry_and_arrows_skip_disabled_without_activating() {
    let mut h = harness();
    h.check_accessibility();
    h.get_by_label("Before toolbar").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("Save").is_focused());
    h.key_press(Key::ArrowRight);
    h.run();
    assert!(h.get_by_label("Bold").is_focused());
    assert!(!h.state().on);
    assert!(h.state().events.is_empty());
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("After toolbar").is_focused());
    h.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    h.run();
    assert!(h.get_by_label("Bold").is_focused());
    h.key_press(Key::End);
    h.run();
    assert!(h.get_by_label("More").is_focused());
    h.key_press(Key::ArrowRight);
    h.run();
    assert!(h.get_by_label("Save").is_focused());
    h.key_press(Key::Home);
    h.run();
    assert!(h.get_by_label("Save").is_focused());
}

#[test]
fn toggle_and_action_events_are_requests_and_only_the_host_applies_them() {
    let mut h = harness();
    h.get_by_label("Bold").click();
    h.run();
    assert_eq!(
        h.state().events,
        [ToolbarEvent::Toggled {
            value: 2,
            pressed: true
        }]
    );
    assert!(!h.state().on);
    assert_eq!(
        h.get_by_label("Bold").accesskit_node().data().toggled(),
        Some(Toggled::False)
    );
    h.state_mut().apply = true;
    h.key_press(Key::Enter);
    h.run();
    h.run();
    assert!(h.state().on);
    assert_eq!(
        h.get_by_label("Bold").accesskit_node().data().toggled(),
        Some(Toggled::True)
    );
    h.get_by_label("Save").click();
    h.run();
    assert_eq!(h.state().events.last(), Some(&ToolbarEvent::Activated(0)));
}

#[test]
fn identity_survives_reorder_and_labels_and_focus_repairs_after_removal() {
    let mut h = harness();
    h.get_by_label("Save").focus();
    h.run();
    let id = h.ctx.memory(|m| m.focused());
    h.state_mut().order = vec![3, 2, 0, 1];
    h.state_mut().prefix = true;
    h.state_mut().renamed = true;
    h.run();
    assert_eq!(h.ctx.memory(|m| m.focused()), id);
    assert!(h.get_by_label("Save document").is_focused());
    h.state_mut().order = vec![3, 2, 1];
    h.run();
    assert!(h.get_by_label("Bold").is_focused());
    h.state_mut().order.clear();
    h.run_steps(2);
    assert!(h.get_by_label("After toolbar").is_focused());
}

#[test]
fn busy_items_and_disabled_groups_cannot_dispatch_and_vertical_keys_follow_orientation() {
    let mut h = harness();
    h.get_by_label("Save").focus();
    h.run();
    h.state_mut().busy = true;
    h.run();
    assert!(h.get_by_label("Bold").is_focused());
    h.state_mut().orientation = ToolbarOrientation::Vertical;
    h.state_mut().width = 80.0;
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    assert!(h.get_by_label("More").is_focused());
    h.key_press(Key::ArrowUp);
    h.run();
    assert!(h.get_by_label("Bold").is_focused());
    for parent in [true, false] {
        h.state_mut().enabled = parent;
        h.state_mut().parent = !parent;
        h.run();
        for label in ["Save", "Bold", "More"] {
            assert!(h.get_by_label(label).accesskit_node().is_disabled());
            h.get_by_label(label).click();
            h.run();
        }
    }
    assert!(h.state().events.is_empty());
}

#[test]
fn focus_reveals_overflow_and_wrapping_keeps_all_items_inside_the_bar() {
    let mut h = harness();
    h.state_mut().width = 100.0;
    h.run();
    h.get_by_label("Save").focus();
    h.run();
    h.key_press(Key::End);
    h.run_steps(3);
    let item = h.get_by_label("More").rect();
    assert!(
        h.state().viewport.contains_rect(item),
        "{item:?} / {:?}",
        h.state().viewport
    );
    h.state_mut().wrap = true;
    h.run();
    let bar = h
        .get_by_role_and_label(Role::Toolbar, "Document tools")
        .rect();
    for label in ["Save", "Unavailable", "Bold", "More"] {
        assert!(bar.contains_rect(h.get_by_label(label).rect()));
    }
}

#[test]
fn retained_focus_does_not_override_manual_overflow_scrolling() {
    let mut h = harness();
    h.state_mut().width = 100.0;
    h.run();
    h.get_by_label("Save").focus();
    h.run();
    h.key_press(Key::End);
    h.run_steps(4);
    let viewport = h.state().viewport;
    assert!(viewport.contains_rect(h.get_by_label("More").rect()));
    h.hover_at(viewport.center());
    h.run_steps(2);
    h.input_mut().events.push(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: egui::vec2(3000.0, 0.0),
        modifiers: Modifiers::NONE,
    });
    h.run_steps(80);
    assert!(h.get_by_label("More").is_focused());
    assert!(
        !viewport.contains_rect(h.get_by_label("More").rect()),
        "before {viewport:?} / after {:?}",
        h.get_by_label("More").rect()
    );
    assert!(h.state().events.is_empty());
    h.key_press(Key::Home);
    h.run_steps(4);
    assert!(viewport.contains_rect(h.get_by_label("Save").rect()));
}

#[test]
fn vertical_geometry_is_a_column_even_inside_a_horizontal_parent() {
    let mut h = Harness::builder()
        .with_size([520.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, bounded: &mut bool| {
                ui.horizontal(|ui| {
                    let items = [
                        ToolbarItem::action(0u8, "Save").icon(Icon::Save),
                        ToolbarItem::action(1, "Undo").icon(Icon::Undo),
                        ToolbarItem::action(2, "Download").icon(Icon::Download),
                    ];
                    let mut toolbar = Toolbar::new("vertical", &items)
                        .accessible_name("Vertical tools")
                        .orientation(ToolbarOrientation::Vertical);
                    if *bounded {
                        toolbar = toolbar.max_height(64.0);
                    }
                    toolbar.show(ui);
                    ui.label("Adjacent content");
                });
            },
            false,
        );
    h.run();
    let bar = h
        .get_by_role_and_label(Role::Toolbar, "Vertical tools")
        .rect();
    let rects: Vec<_> = ["Save", "Undo", "Download"]
        .map(|name| h.get_by_label(name).rect())
        .into();
    assert!(
        rects
            .windows(2)
            .all(|pair| pair[0].left() == pair[1].left() && pair[0].bottom() < pair[1].top())
    );
    assert!(
        rects
            .iter()
            .all(|rect| rect.left() >= bar.left() && rect.right() <= bar.right())
    );
    assert!(rects.iter().all(|rect| bar.contains_rect(*rect)));
    assert!(h.get_by_label("Adjacent content").rect().left() >= bar.right());
    // Explicitly bound columns scroll while preserving their vertical geometry.
    *h.state_mut() = true;
    h.run();
    let bar = h
        .get_by_role_and_label(Role::Toolbar, "Vertical tools")
        .rect();
    assert!(!bar.contains_rect(h.get_by_label("Download").rect()));
    h.get_by_label("Save").focus();
    h.run();
    h.key_press(Key::End);
    h.run_steps(4);
    assert!(bar.contains_rect(h.get_by_role_and_label(Role::Button, "Download").rect()));
}
