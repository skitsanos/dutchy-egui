//! Breadcrumb destination requests, current-page semantics, wrapping and collapse.
mod common;
use dutchy_egui::{BreadcrumbItem, Breadcrumbs, Input, Theme};
use egui::{
    Key, Rect,
    accesskit::{AriaCurrent, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
#[derive(Default)]
struct State {
    open: bool,
    collapsed: bool,
    disabled: bool,
    inserted: bool,
    renamed: bool,
    requests: Vec<u8>,
    rects: Vec<Rect>,
    outside: String,
}
fn harness(width: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size([width, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.inserted {
                    ui.label("Inserted sibling");
                }
                let items = [
                    BreadcrumbItem::new(1, "Home"),
                    BreadcrumbItem::new(
                        2,
                        if state.renamed {
                            "Renamed library"
                        } else {
                            "Library"
                        },
                    ),
                    BreadcrumbItem::new(3, "Unavailable").enabled(false),
                    BreadcrumbItem::new(4, "Components"),
                    BreadcrumbItem::new(5, "A current page with a long descriptive name"),
                ];
                let mut widget = Breadcrumbs::new("path", &items)
                    .accessible_name("Workspace path")
                    .enabled(!state.disabled);
                if state.collapsed {
                    widget = widget.collapse(2, &mut state.open);
                }
                let result = widget.show(ui);
                state.rects = result.items.iter().map(|r| r.rect).collect();
                if let Some(request) = result.navigate {
                    state.requests.push(request);
                }
                ui.add(Input::new("outside", &mut state.outside).accessible_name("Outside"));
            },
            State::default(),
        )
}
#[test]
fn ancestors_activate_but_current_page_is_named_noninteractive_and_not_a_tab_stop() {
    let mut h = harness(760.0);
    assert!(
        h.query_by_role_and_label(Role::Navigation, "Workspace path")
            .is_some()
    );
    let current = h.get_by_label("A current page with a long descriptive name");
    assert_eq!(
        current.accesskit_node().data().aria_current(),
        Some(AriaCurrent::Page)
    );
    current.click();
    h.run();
    assert!(h.state().requests.is_empty());
    h.get_by_label("Library").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().requests, [2]);
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().requests, [2, 2]);
    h.get_by_label("Components").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Outside").is_focused());
    assert!(h.get_by_label("Unavailable").accesskit_node().is_disabled());
    h.get_by_label("Unavailable").click();
    h.run();
    assert_eq!(h.state().requests, [2, 2]);
    h.check_accessibility();
}
#[test]
fn collapsed_ancestors_use_keyboard_menu_dismissal_and_return_typed_destinations() {
    let mut h = harness(500.0);
    h.state_mut().collapsed = true;
    h.run_steps(2);
    assert!(h.query_by_label("Library").is_none());
    h.get_by_label("Show collapsed ancestors").focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    assert!(h.state().open);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Library")
            .is_focused()
    );
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Components")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(!h.state().open && h.get_by_label("Show collapsed ancestors").is_focused());
    h.key_press(Key::Enter);
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Components")
        .click();
    h.run_steps(2);
    assert_eq!(h.state().requests, [4]);
    assert!(!h.state().open);
    h.check_accessibility();
}
#[test]
fn keyed_focus_survives_insertions_renaming_and_narrow_reflow() {
    let mut h = harness(260.0);
    h.get_by_label("Library").focus();
    h.run_steps(2);
    let id = h.get_by_label("Library").accesskit_node().id();
    h.state_mut().inserted = true;
    h.state_mut().renamed = true;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Renamed library").accesskit_node().id(), id);
    assert!(h.get_by_label("Renamed library").is_focused());
    assert!(
        h.state()
            .rects
            .iter()
            .all(|r| r.left() >= 8.0 && r.right() <= 252.5),
        "{:?}",
        h.state().rects
    );
    assert!(h.state().rects.windows(2).any(|r| r[1].top() > r[0].top()));
    h.state_mut().disabled = true;
    h.run();
    h.get_by_label("Home").click();
    h.run();
    assert!(h.state().requests.is_empty());
}
#[test]
fn empty_and_single_paths_are_safe_and_single_current_never_navigates() {
    let mut h = Harness::builder()
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let empty: [BreadcrumbItem<'_, u8>; 0] = [];
            assert!(Breadcrumbs::new("empty", &empty).show(ui).items.is_empty());
            let single = [BreadcrumbItem::new(1, "Only current")];
            assert!(
                Breadcrumbs::new("single", &single)
                    .show(ui)
                    .navigate
                    .is_none()
            );
        });
    h.run();
    h.check_accessibility();
}

#[test]
fn an_icon_keeps_its_named_target_compact_without_laying_out_hidden_label_text() {
    let mut h = Harness::builder()
        .with_size([320.0, 200.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let items = [
                BreadcrumbItem::new(1, "A long accessible name for the home icon")
                    .icon(|_, _, _| {}),
                BreadcrumbItem::new(2, "Current"),
            ];
            Breadcrumbs::new("icons", &items).show(ui);
        });
    h.run_steps(2);
    let icon = h.get_by_label("A long accessible name for the home icon");
    assert_eq!(
        icon.rect().height(),
        dutchy_egui::ControlSize::Small.height()
    );
    assert_eq!(icon.rect().top(), h.get_by_label("Current").rect().top());
}

#[test]
fn collapsed_ancestor_buttons_rest_on_the_border_color() {
    let mut h = harness(900.0);
    h.state_mut().collapsed = true;
    h.run_steps(2);
    let target = h.get_by_label("Show collapsed ancestors").rect();
    let stroke = h
        .output()
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if shape.rect == target && shape.stroke.width == 2.0 => {
                Some(shape.stroke.color)
            }
            _ => None,
        })
        .expect("ellipsis outline painted");
    assert_eq!(stroke, Theme::default().palette.border);
}
