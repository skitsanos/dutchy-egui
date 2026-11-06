//! Blocking surfaces, theme/scale baselines, overflow and showcase host flows.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{Accent, Drawer, DrawerSide, Modal, ModalWidth, Mode, Text, Theme};
use egui::{Key, Rect, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
#[derive(Clone, Copy)]
enum Surface {
    Modal,
    Left,
    Right,
}
struct State {
    open: bool,
    rect: Option<Rect>,
    value: String,
    long: bool,
}
fn matrix(
    surface: Surface,
    theme: Theme,
    scale: f32,
    size: [f32; 2],
    long: bool,
) -> Harness<'static, State> {
    Harness::builder().with_size(size).with_pixels_per_point(scale).with_render_cursor(false)
        .with_theme(if theme.mode==Mode::Light {egui::Theme::Light}else{egui::Theme::Dark})
        .with_context_setup(move |ctx|common::install(ctx,theme))
        .build_ui_state(move |ui,state:&mut State| {
            ui.painter().rect_filled(ui.max_rect(),0,Theme::from_context(ui.ctx()).palette.background);
            ui.add(Text::heading("Stay in context."));
            ui.add(Text::new("The host workspace is blocked while this surface is open."));
            let _=ui.button("Background action");
            let body=|ui:&mut egui::Ui| {
                ui.add(Text::new("Edit the sample record. The application owns its values and saving."));
                ui.add_space(16.0);
                dutchy_egui::Field::new("Record name").show(ui,|ui|ui.add(dutchy_egui::Input::new("name",&mut state.value)));
                if state.long {for n in 0..25 {ui.add(Text::new(format!("Supporting detail {n:02}. Content wraps and scrolls within the viewport.")));}}
            };
            let footer=|ui:&mut egui::Ui| {if ui.add(dutchy_egui::Button::new("APPLY RECORD").full_width(true)).clicked(){ui.close();}};
            let result=match surface {
                Surface::Modal=>Modal::new("surface",&mut state.open,"Edit record").width(ModalWidth::Large).show_with_footer(ui.ctx(),body,footer),
                Surface::Left|Surface::Right=>Drawer::new("surface",&mut state.open,"Edit record").side(if matches!(surface,Surface::Left){DrawerSide::Left}else{DrawerSide::Right}).show_with_footer(ui.ctx(),body,footer),
            };
            if let Some(result)=result {state.rect=Some(result.response.rect);}
        },State {open:true,rect:None,value:"Design studio".into(),long})
}
#[test]
fn modal_and_both_drawer_edges_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, prefix) in [
        (Accent::Orange, Mode::Light, 1.0, "overlay-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "overlay-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "overlay-purple-2x"),
    ] {
        for (surface, suffix) in [
            (Surface::Modal, "modal"),
            (Surface::Left, "left"),
            (Surface::Right, "right"),
        ] {
            let mut h = matrix(
                surface,
                Theme::new(accent, mode).with_reduced_motion(true),
                scale,
                [800.0, 640.0],
                false,
            );
            h.run_steps(5);
            h.check_accessibility();
            h.snapshot(format!("{prefix}-{suffix}"));
            results.extend_harness(&mut h);
        }
    }
    results.unwrap();
}
#[test]
fn close_hover_is_visible_and_does_not_dismiss() {
    let mut h = matrix(
        Surface::Modal,
        Theme::default().with_reduced_motion(true),
        1.0,
        [800.0, 640.0],
        false,
    );
    h.run_steps(4);
    let before = h.render().unwrap();
    h.get_by_label("Close Edit record").hover();
    h.run_steps(2);
    assert_ne!(before, h.render().unwrap());
    assert!(h.state().open);
    h.snapshot("overlay-close-hover");
}
#[test]
fn constrained_surfaces_keep_the_header_and_long_content_footer_reachable() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (surface, name) in [
        (Surface::Modal, "modal"),
        (Surface::Left, "left"),
        (Surface::Right, "right"),
    ] {
        let mut h = matrix(
            surface,
            Theme::default().with_reduced_motion(true),
            1.0,
            [280.0, 360.0],
            true,
        );
        h.run_steps(5);
        let rect = h.state().rect.unwrap();
        assert!(
            rect.left() >= -0.5
                && rect.top() >= -0.5
                && rect.right() <= 280.5
                && rect.bottom() <= 360.5,
            "{name}: {rect:?}"
        );
        h.key_press(Key::Tab);
        h.run_steps(3);
        h.key_press(Key::Tab);
        h.run_steps(5);
        assert!(h.get_by_label("APPLY RECORD").is_focused());
        let keyboard_action = h.get_by_label("APPLY RECORD").rect();
        assert!(
            keyboard_action.top() >= 0.0 && keyboard_action.bottom() <= 360.0,
            "{name}: keyboard {keyboard_action:?}"
        );
        h.get_by_label("APPLY RECORD").scroll_to_me();
        h.run_steps(10);
        let action = h.get_by_label("APPLY RECORD").rect();
        assert!(
            action.top() >= 0.0 && action.bottom() <= 360.0,
            "{name}: {action:?}"
        );
        assert!(h.get_by_label("Close Edit record").rect().top() >= 0.0);
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/overlay-{name}-narrow.png"))
            .unwrap();
        h.get_by_label("APPLY RECORD").click();
        h.run_steps(4);
        assert!(!h.state().open);
    }
}
fn example(page: gallery::Page) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size([1200.0, 1100.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(move |cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn showcase_confirmation_only_runs_the_host_action_when_confirmed() {
    let mut h = example(gallery::Page::Modals);
    h.get_by_label("OPEN CONFIRMATION").click();
    h.run_steps(5);
    h.get_by_label("CANCEL ARCHIVE").click();
    h.run_steps(4);
    assert_eq!(h.state().overlays.archives, 0);
    assert!(h.get_by_label("OPEN CONFIRMATION").is_focused());
    h.get_by_label("OPEN CONFIRMATION").click();
    h.run_steps(4);
    h.get_by_label("ARCHIVE PROJECT").click();
    h.run_steps(4);
    assert_eq!(h.state().overlays.archives, 1);
    assert!(!h.state().overlays.confirm_open);
}
#[test]
fn showcase_profile_validation_and_drawer_drafts_remain_host_owned() {
    let mut h = example(gallery::Page::Modals);
    h.get_by_label("EDIT PROFILE").click();
    h.run_steps(4);
    h.state_mut().overlays.draft_name.clear();
    h.run();
    h.get_by_label("SAVE PROFILE").click();
    h.run_steps(3);
    assert!(h.state().overlays.edit_open);
    assert!(h.get_by_label("Profile name").is_focused());
    h.get_by_label("Profile name").type_text("Grace Hopper");
    h.run();
    h.get_by_label("SAVE PROFILE").click();
    h.run_steps(4);
    assert_eq!(h.state().overlays.saved_name, "Grace Hopper");
    assert!(h.get_by_label("EDIT PROFILE").is_focused());
    drawer_workflow(&mut h);
}
fn drawer_workflow(h: &mut Harness<'_, gallery::Showcase>) {
    h.state_mut().page = gallery::Page::Drawers;
    h.run_steps(3);
    h.get_by_label("OPEN LEFT DRAWER").click();
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::Dialog, "Workspace preferences")
            .is_some()
    );
    h.state_mut().overlays.draft_workspace = "Retained draft".into();
    h.run();
    h.get_by_label("Close Workspace preferences").click();
    h.run_steps(4);
    assert!(h.get_by_label("OPEN LEFT DRAWER").is_focused());
    h.get_by_label("OPEN RIGHT DRAWER").click();
    h.run_steps(4);
    assert_eq!(h.state().overlays.draft_workspace, "Retained draft");
    h.get_by_label("APPLY PREFERENCES").scroll_to_me();
    h.run_steps(8);
    h.get_by_label("APPLY PREFERENCES").click();
    h.run_steps(4);
    assert_eq!(h.state().overlays.preference_saves, 1);
    assert!(h.get_by_label("OPEN RIGHT DRAWER").is_focused());
}
