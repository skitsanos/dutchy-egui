//! Cascade geometry, pointer/keyboard ownership and inline dismissal.
mod common;
use dutchy_egui::{
    Accent, Button, Dropdown, Input, Menu, MenuEvent, MenuItem, Mode, Theme,
    tokens::{border, menu as geometry},
};
use egui::{Key, Rect, Shape, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[derive(Default)]
struct State {
    open: bool,
    events: Vec<MenuEvent<u8>>,
    editor: String,
}
fn harness(inline: bool, theme: Theme, scale: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size([1100.0, 650.0])
        .with_pixels_per_point(scale)
        .with_theme(if theme.mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(
            move |ui, state: &mut State| {
                let formats = [MenuItem::new(5, "PNG"), MenuItem::new(6, "SVG")];
                let export = [
                    MenuItem::new(3, "Image format").submenu(&formats),
                    MenuItem::new(4, "Copy SVG"),
                ];
                let items = [
                    MenuItem::new(1, "Save document"),
                    MenuItem::new(2, "Export artwork").submenu(&export),
                ];
                let event = if inline {
                    Menu::new("inline", &items).show(ui).event
                } else {
                    Dropdown::new("commands", &mut state.open, &items)
                        .width(dutchy_egui::MenuWidth::Small)
                        .show(ui, |ui| ui.add(Button::new("Commands")))
                        .menu
                        .and_then(|m| m.event)
                };
                if let Some(event) = event {
                    state.events.push(event);
                }
                ui.add(Input::new("editor", &mut state.editor).accessible_name("Editor"));
            },
            State {
                open: true,
                ..Default::default()
            },
        )
}
fn row(h: &Harness<'_, State>, label: &str) -> Rect {
    h.get_by_role_and_label(Role::MenuItem, label).rect()
}
fn fill(h: &Harness<'_, State>, label: &str) -> egui::Color32 {
    let rect = row(h, label);
    h.output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Rect(shape) if shape.rect == rect => Some(shape.fill),
            _ => None,
        })
        .expect("menu row fill")
}

#[test]
fn pointer_and_keyboard_have_one_active_row_and_quiet_parent_context() {
    for (accent, mode, scale) in [
        (Accent::Orange, Mode::Light, 1.0),
        (Accent::Orange, Mode::Dark, 1.5),
        (Accent::Blue, Mode::Light, 2.0),
    ] {
        let theme = Theme::new(accent, mode);
        let p = theme.palette;
        let mut h = harness(false, theme, scale);
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Save document")
                .is_focused()
        );
        h.hover_at(row(&h, "Export artwork").center());
        h.run_steps(4);
        assert_eq!(fill(&h, "Save document"), p.background);
        assert_eq!(fill(&h, "Export artwork"), p.primary);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Export artwork")
                .is_focused()
        );
        h.hover_at(row(&h, "Image format").center());
        h.run_steps(4);
        assert_eq!(fill(&h, "Export artwork"), p.muted);
        assert_eq!(fill(&h, "Image format"), p.primary);
        assert_ne!(fill(&h, "PNG"), p.primary);
        h.key_press(Key::ArrowDown);
        h.run_steps(3);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Copy SVG")
                .is_focused()
        );
        assert_eq!(fill(&h, "Copy SVG"), p.primary);
        assert_ne!(fill(&h, "Image format"), p.primary);
        assert!(
            h.query_by_role_and_label(Role::MenuItem, "PNG").is_none(),
            "moving to a sibling closes the old child"
        );
        h.check_accessibility();
    }
}

#[test]
fn short_children_fit_content_and_align_with_the_parent_row() {
    let mut h = harness(false, Theme::default(), 1.0);
    h.run_steps(4);
    h.hover_at(row(&h, "Export artwork").center());
    h.run_steps(4);
    let parent = row(&h, "Export artwork");
    let child = row(&h, "Image format");
    assert!(
        (parent.top() - child.top()).abs() <= 0.5,
        "parent {parent:?}, child {child:?}"
    );
    assert!(
        (child.left() - parent.right() - border::STANDARD).abs() <= 0.5,
        "frames should share their bordering stroke"
    );
    assert!((child.width() + 2.0 * border::STANDARD - geometry::SUBMENU_MIN_WIDTH).abs() <= 0.5);
    assert!(child.width() < parent.width());
    h.hover_at(child.center());
    h.run_steps(4);
    assert!((row(&h, "PNG").top() - child.top()).abs() <= 0.5);
}

#[test]
fn wrapped_parent_geometry_does_not_change_when_its_child_opens() {
    const LABEL: &str = "Export the current artwork with all of its editable layers";
    let mut h = Harness::builder()
        .with_size([1000.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let children = [MenuItem::new(2, "PNG")];
            let items = [MenuItem::new(1, LABEL).submenu(&children)];
            let mut open = true;
            Dropdown::new("wrapped", &mut open, &items)
                .width(dutchy_egui::MenuWidth::Compact)
                .show(ui, |ui| ui.add(Button::new("Formats")));
        });
    h.run_steps(4);
    let before = h.get_by_role_and_label(Role::MenuItem, LABEL).rect();
    assert!(before.height() > geometry::ROW_HEIGHT);
    h.hover_at(before.center());
    h.run_steps(4);
    let child = h.get_by_role_and_label(Role::MenuItem, "PNG").rect();
    h.hover_at(child.center());
    h.run_steps(4);
    assert_eq!(
        before,
        h.get_by_role_and_label(Role::MenuItem, LABEL).rect()
    );
    h.check_accessibility();
}

#[test]
fn left_flipped_child_shares_the_edge_and_preserves_row_alignment() {
    let mut h = Harness::builder()
        .with_size([760.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                let children = [MenuItem::new(2, "PNG")];
                let items = [MenuItem::new(1, "Export").submenu(&children)];
                let mut open = true;
                Dropdown::new("flipped", &mut open, &items)
                    .width(dutchy_egui::MenuWidth::Small)
                    .show(ui, |ui| ui.add(Button::new("Formats")));
            });
        });
    h.run_steps(4);
    let parent = h.get_by_role_and_label(Role::MenuItem, "Export").rect();
    h.hover_at(parent.center());
    h.run_steps(4);
    let child = h.get_by_role_and_label(Role::MenuItem, "PNG").rect();
    assert!((parent.top() - child.top()).abs() <= 0.5);
    assert!((parent.left() - child.right() - border::STANDARD).abs() <= 0.5);
    assert!(child.left() >= 0.0);
    h.check_accessibility();
}

#[test]
fn inline_child_activation_closes_descendants_and_returns_to_its_branch() {
    let mut h = harness(true, Theme::default(), 1.0);
    h.run();
    h.get_by_role_and_label(Role::MenuItem, "Export artwork")
        .focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Copy SVG").click();
    h.run_steps(3);
    assert_eq!(h.state().events, [MenuEvent::Activated(4)]);
    assert!(
        h.query_by_role_and_label(Role::MenuItem, "Copy SVG")
            .is_none()
    );
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Export artwork")
            .is_focused()
    );
}

#[test]
fn hover_does_not_steal_an_external_editor_and_new_focus_locks_escape_immediately() {
    let mut h = harness(true, Theme::default(), 1.0);
    h.run();
    h.get_by_label("Editor").click();
    h.run();
    h.hover_at(row(&h, "Export artwork").center());
    h.run_steps(4);
    assert!(h.get_by_label("Editor").is_focused());
    h.get_by_label("Editor").type_text("Keep typing");
    h.run();
    assert_eq!(h.state().editor, "Keep typing");
    let mut h = harness(false, Theme::default(), 1.0);
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Export artwork")
        .focus();
    h.run();
    h.key_press(Key::ArrowRight);
    for _ in 0..4 {
        h.step();
        if h.query_by_role_and_label(Role::MenuItem, "Image format")
            .is_some_and(|n| n.is_focused())
        {
            break;
        }
    }
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Image format")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        h.state().open
            && h.get_by_role_and_label(Role::MenuItem, "Export artwork")
                .is_focused()
    );
}

#[test]
fn escape_does_not_recover_a_stale_menu_focus_after_an_editor_takes_over() {
    let mut h = harness(true, Theme::default(), 1.0);
    h.run_steps(3);
    h.get_by_role_and_label(Role::MenuItem, "Export artwork")
        .focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Image format")
            .is_focused()
    );
    h.get_by_label("Editor").focus();
    h.step();
    assert!(h.get_by_label("Editor").is_focused());
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        !h.get_by_role_and_label(Role::MenuItem, "Export artwork")
            .is_focused()
    );
    assert!(
        !h.get_by_role_and_label(Role::MenuItem, "Image format")
            .is_focused()
    );
}

#[test]
fn focused_row_stays_visible_while_the_pointer_is_outside_the_menu() {
    for (accent, mode, scale) in [
        (Accent::Orange, Mode::Light, 1.0),
        (Accent::Orange, Mode::Dark, 1.5),
        (Accent::Blue, Mode::Light, 2.0),
    ] {
        let theme = Theme::new(accent, mode);
        let p = theme.palette;
        let mut h = harness(false, theme, scale);
        h.run_steps(4);
        let save = h.get_by_role_and_label(Role::MenuItem, "Save document");
        assert!(save.is_focused());
        assert_eq!(fill(&h, "Save document"), p.primary);
        h.hover_at(egui::pos2(1000.0, 600.0));
        h.run_steps(4);
        assert!(
            h.get_by_role_and_label(Role::MenuItem, "Save document")
                .is_focused(),
            "moving the pointer away keeps keyboard focus"
        );
        assert_eq!(
            fill(&h, "Save document"),
            p.muted,
            "the row Enter would activate keeps a visible fill"
        );
        assert_ne!(fill(&h, "Export artwork"), p.primary);
        h.key_press(Key::ArrowDown);
        h.run_steps(3);
        assert_eq!(fill(&h, "Export artwork"), p.primary);
        assert_eq!(fill(&h, "Save document"), p.background);
    }
}

#[test]
fn a_diagonal_pointer_path_across_a_sibling_keeps_the_open_child() {
    let theme = Theme::default();
    let mut h = Harness::builder()
        .with_size([900.0, 600.0])
        // Pointer direction needs samples inside egui's 0.1 s history.
        .with_step_dt(1.0 / 60.0)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(
            |ui, open: &mut bool| {
                let formats = [
                    MenuItem::new(10, "PNG"),
                    MenuItem::new(11, "SVG"),
                    MenuItem::new(12, "PDF"),
                    MenuItem::new(13, "WebP"),
                ];
                let items = [
                    MenuItem::new(1, "Export artwork").submenu(&formats),
                    MenuItem::new(2, "Save document"),
                    MenuItem::new(3, "Close document"),
                ];
                Dropdown::new("diagonal", open, &items)
                    .show(ui, |ui| ui.add(Button::new("Commands")));
            },
            true,
        );
    h.run_steps(4);
    let branch = row_of(&h, "Export artwork");
    h.hover_at(branch.center());
    h.run_steps(4);
    let target = row_of(&h, "WebP");
    let sibling = row_of(&h, "Save document");
    assert!(
        sibling.top() >= branch.bottom() - 0.5,
        "the sibling sits below the branch"
    );
    // Head from the branch's right end down towards the last child and cross
    // the sibling row on the way, as a user cutting the corner would.
    let start = egui::pos2(branch.right() - 24.0, branch.center().y);
    let crossing = egui::pos2(sibling.right() - 12.0, sibling.center().y);
    assert!(
        target.center().y > crossing.y,
        "the child extends below the crossing"
    );
    // egui derives direction from at least three recent samples per frame.
    for step in 0..=4 {
        h.hover_at(start + (crossing - start) * (step as f32 / 4.0));
        h.run_steps(1);
    }
    assert!(
        sibling.contains(crossing),
        "the path ends on the sibling row"
    );
    assert!(
        h.query_by_role_and_label(Role::MenuItem, "WebP").is_some(),
        "moving towards the child keeps it open"
    );
    h.hover_at(target.center());
    h.run_steps(4);
    assert!(h.get_by_role_and_label(Role::MenuItem, "WebP").is_focused());
}
fn row_of<S>(h: &Harness<'_, S>, label: &str) -> Rect {
    h.get_by_role_and_label(Role::MenuItem, label).rect()
}
