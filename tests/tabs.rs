//! Roving focus, manual activation, close requests, overflow, and stable panels.
mod common;
use dutchy_egui::{Input, Tab, Tabs, TabsVariant, Theme};
use egui::{Key, Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct Item {
    id: u8,
    label: &'static str,
    enabled: bool,
}
struct State {
    selected: Option<u8>,
    items: Vec<Item>,
    values: [String; 4],
    closed: Option<u8>,
    remove: bool,
    rendered: Option<u8>,
    inserted: bool,
    disabled: bool,
    width: f32,
    viewport: Rect,
}
impl Default for State {
    fn default() -> Self {
        Self {
            selected: Some(1),
            items: [
                (1, "Overview", true),
                (2, "Reserved", false),
                (3, "Settings", true),
                (4, "Activity log", true),
            ]
            .into_iter()
            .map(|(id, label, enabled)| Item { id, label, enabled })
            .collect(),
            values: Default::default(),
            closed: None,
            remove: false,
            rendered: None,
            inserted: false,
            disabled: false,
            width: 650.0,
            viewport: Rect::NOTHING,
        }
    }
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([700.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                ui.set_width(state.width);
                let before = ui.button("Before");
                if state.inserted {
                    ui.label("New sibling");
                }
                let items: Vec<_> = state
                    .items
                    .iter()
                    .map(|item| {
                        Tab::new(item.id, item.label)
                            .enabled(item.enabled)
                            .closable(true)
                    })
                    .collect();
                state.rendered = None;
                let style = ui.style().clone();
                let spacing = ui.spacing().item_spacing;
                let result = ui
                    .add_enabled_ui(!state.disabled, |ui| {
                        Tabs::new("workspace", &mut state.selected, &items)
                            .accessible_name("Workspace tabs")
                            .focus_when_empty(before.id)
                            .show(ui, |ui, key| {
                                assert_eq!(ui.spacing().item_spacing, spacing);
                                state.rendered = Some(*key);
                                ui.add(
                                    Input::new("editor", &mut state.values[usize::from(*key) - 1])
                                        .accessible_name(format!("Edit {key}")),
                                );
                            })
                    })
                    .inner;
                state.viewport = result.viewport;
                if let Some(key) = result.close_requested {
                    state.closed = Some(key);
                    if state.remove {
                        state.items.retain(|item| item.id != key);
                    }
                }
                assert_eq!(style.as_ref(), ui.style().as_ref());
                let _ = ui.button("After");
            },
            State::default(),
        )
}

#[test]
fn arrows_skip_disabled_tabs_and_enter_space_activate_without_leaking_to_neighbors() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Tab, "OVERVIEW").focus();
    h.run_steps(2);
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::Tab, "SETTINGS").is_focused());
    assert_eq!(h.state().selected, Some(1));
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(3));
    assert_eq!(h.state().rendered, Some(3));
    assert!(h.query_by_label("Edit 1").is_none());
    h.key_press(Key::End);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Tab, "ACTIVITY LOG")
            .is_focused()
    );
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(4));
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::Tab, "OVERVIEW").is_focused());
    h.key_press(Key::Home);
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(4));
    h.check_accessibility();
}

#[test]
fn tab_order_enters_once_and_panels_have_selected_relationships() {
    let mut h = harness();
    h.get_by_label("Before").focus();
    h.run_steps(2);
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::Tab, "OVERVIEW").is_focused());
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Close Settings").is_focused());
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::TabPanel, "OVERVIEW")
            .is_focused()
    );
    let tab = h.get_by_role_and_label(Role::Tab, "OVERVIEW");
    let panel = h.get_by_role_and_label(Role::TabPanel, "OVERVIEW");
    assert_eq!(tab.accesskit_node().is_selected(), Some(true));
    assert_eq!(
        tab.accesskit_node()
            .controls()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        &[panel.accesskit_node().id()]
    );
    assert_eq!(
        panel
            .accesskit_node()
            .labelled_by()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        &[tab.accesskit_node().id()]
    );
    assert_eq!(
        h.get_by_role_and_label(Role::Tab, "SETTINGS")
            .accesskit_node()
            .is_selected(),
        Some(false)
    );
}

#[test]
fn close_requests_are_independent_and_host_removal_restores_focus() {
    let mut h = harness();
    h.get_by_label("Close Settings").click();
    h.run_steps(2);
    assert_eq!(h.state().closed, Some(3));
    assert_eq!(h.state().selected, Some(1));
    assert_eq!(h.state().items.len(), 4);
    h.get_by_role_and_label(Role::Tab, "SETTINGS").click();
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(3));
    h.state_mut().remove = true;
    h.key_press(Key::Delete);
    h.run_steps(3);
    assert_eq!(h.state().selected, Some(1));
    assert!(h.get_by_role_and_label(Role::Tab, "OVERVIEW").is_focused());
    h.state_mut().items.retain(|item| item.id == 1);
    h.run_steps(2);
    h.key_press(Key::Delete);
    h.run_steps(3);
    assert_eq!(h.state().selected, None);
    assert_eq!(h.state().rendered, None);
    assert!(h.get_by_label("Before").is_focused());
}

#[test]
fn keyed_panels_preserve_edits_and_focus_across_reordering_and_resizing() {
    let mut h = harness();
    h.get_by_label("Edit 1").focus();
    h.run_steps(2);
    let id = h.get_by_label("Edit 1").accesskit_node().id();
    h.state_mut().items.reverse();
    h.state_mut()
        .items
        .iter_mut()
        .find(|item| item.id == 1)
        .unwrap()
        .label = "Renamed overview";
    h.state_mut().inserted = true;
    h.state_mut().width = 260.0;
    h.run_steps(3);
    assert_eq!(h.get_by_label("Edit 1").accesskit_node().id(), id);
    assert!(h.get_by_label("Edit 1").is_focused());
    h.get_by_label("Edit 1").type_text("Host value");
    h.run();
    assert_eq!(h.state().values[0], "Host value");
}

#[test]
fn keyboard_focus_scrolls_overflow_tabs_into_view_without_changing_selection() {
    let mut h = harness();
    h.state_mut().width = 260.0;
    h.run_steps(2);
    h.get_by_role_and_label(Role::Tab, "OVERVIEW").focus();
    h.run_steps(2);
    h.key_press(Key::End);
    h.run_steps(4);
    let rect = h.get_by_role_and_label(Role::Tab, "ACTIVITY LOG").rect();
    assert!(h.state().viewport.height() <= dutchy_egui::tokens::disclosure::TAB_HEIGHT + 1.0);
    assert!(
        h.get_by_role_and_label(Role::Tab, "ACTIVITY LOG")
            .is_focused()
    );
    assert!(
        rect.left() >= h.state().viewport.left() - 1.0
            && rect.right() <= h.state().viewport.right() + 1.0,
        "{rect:?} / {:?}",
        h.state().viewport
    );
    assert_eq!(h.state().selected, Some(1));
}

#[test]
fn disabled_parent_and_disabled_tabs_do_not_select_close_or_edit() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Tab, "RESERVED").click();
    h.run();
    assert_eq!(h.state().selected, Some(1));
    h.state_mut().disabled = true;
    h.run();
    for label in ["OVERVIEW", "SETTINGS"] {
        assert!(
            h.get_by_role_and_label(Role::Tab, label)
                .accesskit_node()
                .is_disabled()
        );
        h.get_by_role_and_label(Role::Tab, label).click();
        h.run();
    }
    h.get_by_label("Close Overview").click();
    h.run();
    assert_eq!(h.state().selected, Some(1));
    assert_eq!(h.state().closed, None);
    assert!(h.get_by_label("Edit 1").accesskit_node().is_disabled());
}

#[test]
fn missing_selection_and_all_disabled_or_empty_lists_have_defined_behavior() {
    let mut h = harness();
    h.state_mut().selected = Some(99);
    h.run();
    assert_eq!(h.state().selected, Some(1));
    for item in &mut h.state_mut().items {
        item.enabled = false;
    }
    h.run_steps(2);
    assert_eq!(h.state().selected, None);
    assert_eq!(h.state().rendered, None);
    h.state_mut().items.clear();
    h.run_steps(2);
    h.check_accessibility();
    // A segmented empty strip also avoids division by zero.
    let mut empty = Harness::builder()
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let mut selected = None::<u8>;
            let result = Tabs::new("empty", &mut selected, &[])
                .variant(TabsVariant::Segmented)
                .show(ui, |_, _| panic!("empty panel"));
            assert!(result.tabs.is_empty() && result.panel.is_none());
        });
    empty.run();
}

#[test]
fn tab_strip_keeps_its_height_below_a_vertical_scroll_areas_initial_bounds() {
    let mut h = Harness::builder()
        .with_size([360.0, 300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, viewport: &mut Rect| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(400.0);
                    let mut selected = Some(1);
                    let items = [Tab::new(1, "First"), Tab::new(2, "Second")];
                    let result = Tabs::new("nested", &mut selected, &items)
                        .show(ui, |ui, _| ui.label("Panel"));
                    *viewport = result.viewport;
                });
            },
            Rect::NOTHING,
        );
    h.run_steps(2);
    assert!((h.state().height() - dutchy_egui::tokens::disclosure::TAB_HEIGHT).abs() <= 1.0);
}
