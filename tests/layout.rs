//! Native geometry and stable control identities while layouts reflow.

mod common;
use dutchy_egui::{
    ColumnWidth, Container, ContainerWidth, CrossAlign, Flex, FlexDirection, Input, ResponsiveGrid,
    Spacing, Theme,
};
use egui::{Rect, Ui};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn flex_wraps_with_token_gaps_and_keeps_native_reverse_order() {
    let mut h = Harness::builder()
        .with_size([300.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.style().clone();
            Flex::row("wrapped")
                .gap(Spacing::Lg)
                .align(CrossAlign::Start)
                .wrap(true)
                .show(ui, |ui| {
                    for name in ["First", "Second", "Third"] {
                        ui.add_sized([120.0, 48.0], egui::Button::new(name));
                    }
                });
            Flex::row("reverse")
                .direction(FlexDirection::RowReverse)
                .gap(Spacing::Xxl)
                .show(ui, |ui| {
                    let _ = ui.button("Right");
                    let _ = ui.button("Left");
                });
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.run();
    let first = h.get_by_label("First").rect();
    let second = h.get_by_label("Second").rect();
    let third = h.get_by_label("Third").rect();
    assert!((second.left() - first.right() - Spacing::Lg.points()).abs() < 0.1);
    assert!(third.top() >= first.bottom() + Spacing::Lg.points() - 0.1);
    assert!(h.get_by_label("Left").rect().right() < h.get_by_label("Right").rect().left());
    h.check_accessibility();
}

#[test]
fn centered_container_respects_outer_width_and_horizontal_gutters() {
    for width in [260.0, 900.0] {
        let mut h = Harness::builder()
            .with_size([width, 300.0])
            .with_context_setup(|ctx| common::install(ctx, Theme::default()))
            .build_ui_state(
                |ui, bounds: &mut (Rect, Rect, Rect)| {
                    bounds.0 = ui.available_rect_before_wrap();
                    let result = Container::new("centered")
                        .max_width(ContainerWidth::Sm)
                        .padding(Spacing::Lg)
                        .show(ui, |ui| {
                            bounds.2 = ui.available_rect_before_wrap();
                            ui.add(
                                Input::new("name", &mut String::from("Host content"))
                                    .accessible_name("Name"),
                            )
                        });
                    bounds.1 = result.response.rect;
                },
                (Rect::NOTHING, Rect::NOTHING, Rect::NOTHING),
            );
        h.run();
        let (available, outer, inner) = *h.state();
        assert!(
            (outer.width() - available.width().min(384.0)).abs() <= 0.1,
            "{outer:?}"
        );
        assert!((outer.center().x - available.center().x).abs() <= 0.1);
        assert!((inner.left() - outer.left() - 16.0).abs() <= 0.1);
        assert!(inner.right() <= outer.right() - 15.9);
    }
}

struct Item {
    id: u8,
    value: String,
}
struct GridState {
    items: Vec<Item>,
    width: f32,
    inserted: bool,
}

fn grid_ui(ui: &mut Ui, state: &mut GridState) {
    ui.set_width(state.width);
    if state.inserted {
        ui.label("Extra sibling above the grid");
    }
    let results = ResponsiveGrid::new("editable")
        .columns(3)
        .min_column_width(ColumnWidth::Compact)
        .gap(Spacing::Lg)
        .show(
            ui,
            &state.items,
            |item| item.id,
            |ui, item| {
                let mut value = item.value.clone();
                ui.add(
                    Input::new("value", &mut value).accessible_name(format!("Item {}", item.id)),
                );
                value
            },
        );
    for (item, value) in state.items.iter_mut().zip(results.inner) {
        item.value = value;
    }
}

#[test]
fn keyed_grid_preserves_focus_and_edits_across_reflow_reorder_and_sibling_insertion() {
    let mut h = Harness::builder()
        .with_size([700.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            grid_ui,
            GridState {
                width: 650.0,
                inserted: false,
                items: (1..=3)
                    .map(|id| Item {
                        id,
                        value: String::new(),
                    })
                    .collect(),
            },
        );
    let first = h.get_by_label("Item 1").rect();
    let second = h.get_by_label("Item 2").rect();
    assert!((first.top() - second.top()).abs() < 0.1);
    h.get_by_label("Item 2").focus();
    h.run();
    let id = h.get_by_label("Item 2").accesskit_node().id();
    h.state_mut().width = 240.0;
    h.state_mut().inserted = true;
    h.state_mut().items.reverse();
    h.run_steps(2);
    assert_eq!(h.get_by_label("Item 2").accesskit_node().id(), id);
    assert!(h.get_by_label("Item 2").is_focused());
    assert!(h.get_by_label("Item 2").rect().top() > h.get_by_label("Item 3").rect().bottom());
    h.get_by_label("Item 2").type_text("Edited by host");
    h.run_steps(2);
    assert_eq!(
        h.state()
            .items
            .iter()
            .find(|item| item.id == 2)
            .unwrap()
            .value,
        "Edited by host"
    );
    h.check_accessibility();
}

#[test]
fn empty_grid_has_no_callbacks_and_zero_columns_is_safe() {
    let mut h = Harness::builder()
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let empty: [u8; 0] = [];
            let result = ResponsiveGrid::new("empty").columns(0).show(
                ui,
                &empty,
                |item| *item,
                |_, _| panic!("empty grid callback"),
            );
            assert_eq!(result.response.rect.height(), 0.0);
            let result = ResponsiveGrid::new("single").columns(0).show(
                ui,
                &[1u8],
                |item| *item,
                |ui, _| ui.button("Single"),
            );
            assert_eq!(result.inner.len(), 1);
        });
    h.run();
    assert!(h.query_by_label("Single").is_some());
}
