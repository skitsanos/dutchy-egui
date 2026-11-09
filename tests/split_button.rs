//! Native command controls and interaction regressions.
mod common;
use dutchy_egui::{
    MenuItem, SplitButton, Theme,
    egui::{Key, accesskit::Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
#[derive(Default)]
struct State {
    open: bool,
    primary: usize,
    actions: Vec<u8>,
    disabled: bool,
    busy: bool,
    empty: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([500.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, s: &mut State| {
                let _ = ui.button("Before");
                let items = if s.empty {
                    vec![]
                } else {
                    vec![
                        MenuItem::new(1, "PNG image"),
                        MenuItem::new(2, "Disabled format").enabled(false),
                        MenuItem::new(3, "SVG vector"),
                    ]
                };
                let out = SplitButton::new("export", "Export", &mut s.open, &items)
                    .alternatives_name("Export alternatives")
                    .enabled(!s.disabled)
                    .loading(s.busy)
                    .show(ui);
                if out.primary_clicked {
                    s.primary += 1;
                }
                if let Some(action) = out.action {
                    s.actions.push(action);
                }
                assert!(
                    (out.primary.rect.right() - out.dropdown.trigger.rect.left()).abs()
                        <= dutchy_egui::tokens::border::STANDARD,
                    "targets must remain joined"
                );
                let _ = ui.button("After");
            },
            State::default(),
        )
}
#[test]
fn primary_and_alternatives_are_independent_and_restore_arrow_focus() {
    let mut h = harness();
    h.get_by_label("Export").click();
    h.run();
    assert_eq!(h.state().primary, 1);
    assert!(!h.state().open);
    h.get_by_label("Export alternatives").click();
    h.run_steps(4);
    assert!(h.state().open);
    h.get_by_role_and_label(Role::MenuItem, "SVG vector")
        .click();
    h.run_steps(4);
    assert_eq!(h.state().actions, [3]);
    assert_eq!(h.state().primary, 1);
    assert!(!h.state().open);
    assert!(h.get_by_label("Export alternatives").is_focused());
    h.check_accessibility();
}
#[test]
fn keyboard_opens_from_primary_and_tab_leaves_the_composite() {
    let mut h = harness();
    h.get_by_label("Export").focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "PNG image")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().open);
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_label("After").is_focused());
}
#[test]
fn busy_disabled_and_empty_alternatives_are_inert() {
    let mut h = harness();
    h.state_mut().disabled = true;
    h.run();
    assert!(h.get_by_label("Export").accesskit_node().is_disabled());
    assert!(
        h.get_by_label("Export alternatives")
            .accesskit_node()
            .is_disabled()
    );
    h.state_mut().disabled = false;
    h.state_mut().busy = true;
    h.run();
    assert!(h.get_by_label("Export").accesskit_node().is_disabled());
    h.state_mut().busy = false;
    h.state_mut().empty = true;
    h.run();
    assert!(!h.get_by_label("Export").accesskit_node().is_disabled());
    assert!(
        h.get_by_label("Export alternatives")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn right_to_left_hosts_keep_the_alternatives_arrow_on_the_right() {
    let mut h = Harness::builder()
        .with_size([300.0, 200.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut open = false;
                let items = [MenuItem::new(1, "Alternative")];
                let out = SplitButton::new("rtl", "Export", &mut open, &items).show(ui);
                assert!(out.dropdown.trigger.rect.center().x > out.primary.rect.center().x);
                assert!(out.dropdown.trigger.rect.right() <= ui.max_rect().right() + 0.5);
            });
        });
    h.run_steps(3);
    h.check_accessibility();
}
#[test]
fn disabled_and_busy_composites_fade_once_without_a_seam() {
    use dutchy_egui::{ButtonVariant, egui::Rect};
    let p = Theme::default().palette;
    for (variant, disabled, busy) in [
        (ButtonVariant::Primary, true, false),
        (ButtonVariant::Primary, false, true),
        (ButtonVariant::Secondary, true, false),
    ] {
        let mut h = Harness::builder()
            .with_size([400.0, 120.0])
            .with_context_setup(|ctx| common::install(ctx, Theme::default()))
            .build_ui_state(
                move |ui, rects: &mut Option<(Rect, Rect)>| {
                    let items = [MenuItem::new(1, "PNG image")];
                    let mut open = false;
                    let out = SplitButton::new("seam", "Export", &mut open, &items)
                        .variant(variant)
                        .enabled(!disabled)
                        .loading(busy)
                        .show(ui);
                    *rects = Some((out.primary.rect, out.dropdown.trigger.rect));
                },
                None,
            );
        h.run();
        let (primary, arrow) = h.state().expect("rendered");
        let image = h.render().expect("render");
        let pixel = |x: u32, y: f32| image.get_pixel(x, y as u32).0;
        // Above the inset divider, label and chevron, one faded fill spans the joint.
        let row = primary.top() + 2.0;
        let fill = pixel(primary.left() as u32 + 4, row);
        for x in primary.left() as u32 + 1..arrow.right() as u32 - 1 {
            assert_eq!(
                pixel(x, row),
                fill,
                "{variant:?} busy={busy}: seam at x={x}"
            );
        }
        // The divider is present but fades with the rest of the control.
        let divider = pixel(arrow.left() as u32, primary.center().y);
        assert_ne!(divider, fill, "{variant:?}: divider missing");
        assert_ne!(
            divider[..3],
            p.border.to_array()[..3],
            "{variant:?} busy={busy}: divider is not faded"
        );
    }
}
#[test]
fn focused_outline_arrow_keeps_its_chevron_visible() {
    use dutchy_egui::{ButtonVariant, egui::Rect};
    let mut h = Harness::builder()
        .with_size([400.0, 120.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, arrow: &mut Option<Rect>| {
                let items = [MenuItem::new(1, "PNG image")];
                let mut open = false;
                let out = SplitButton::new("ink", "Export", &mut open, &items)
                    .variant(ButtonVariant::Outline)
                    .alternatives_name("Export alternatives")
                    .show(ui);
                *arrow = Some(out.dropdown.trigger.rect);
            },
            None,
        );
    h.run();
    for _ in 0..2 {
        h.key_press(Key::Tab);
        h.run();
    }
    assert!(
        h.get_by_label("Export alternatives")
            .accesskit_node()
            .is_focused()
    );
    let arrow = h.state().expect("rendered");
    let image = h.render().expect("render");
    let c = arrow.center();
    let fill = image.get_pixel(arrow.right() as u32 - 4, c.y as u32).0;
    let ink = (c.x as u32 - 6..c.x as u32 + 6)
        .flat_map(|x| (c.y as u32 - 4..c.y as u32 + 4).map(move |y| (x, y)))
        .filter(|&(x, y)| image.get_pixel(x, y).0 != fill)
        .count();
    assert!(
        ink > 4,
        "chevron blends into the focused fill ({ink} ink pixels)"
    );
}
