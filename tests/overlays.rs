//! Host visibility, native modality, dismissal, and focus lifecycle.
mod common;
use dutchy_egui::{Drawer, DrawerSide, Modal, OverlayDismissal, OverlayOptions, Theme};
use egui::{Id, Key, Modifiers, Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
#[derive(Default)]
struct State {
    open: bool,
    drawer: Option<DrawerSide>,
    options: OverlayOptions,
    value: String,
    outside: usize,
    saved: usize,
    dismissed: Vec<OverlayDismissal>,
    surface: Option<Rect>,
    initial_input: bool,
    inserted: bool,
    hide_opener: bool,
}
fn harness(size: [f32; 2]) -> Harness<'static, State> {
    Harness::builder()
        .with_size(size)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                let opener = if !state.hide_opener {
                    Some(ui.button("Open editor"))
                } else {
                    None
                };
                if let Some(opener) = &opener
                    && opener.clicked()
                {
                    state.open = true;
                }
                if ui.button("Outside action").clicked() {
                    state.outside += 1;
                }
                let fallback = ui.button("Focus fallback");
                let mut options = state.options.clone();
                if state.hide_opener {
                    options.return_focus = Some(fallback.id);
                }
                if state.initial_input {
                    options.initial_focus = Some(Id::unique("initial-field"));
                }
                let value = &mut state.value;
                let inserted = state.inserted;
                let body = |ui: &mut egui::Ui| {
                    if inserted {
                        ui.label("Inserted content");
                    }
                    ui.label("Edit the name; the host retains this value.");
                    ui.add(egui::TextEdit::singleline(value).id(Id::unique("initial-field")))
                        .widget_info(|| {
                            egui::WidgetInfo::labeled(Role::TextInput, true, "Editor name")
                        });
                    ui.menu_button("Editor options", |ui| {
                        let _ = ui.button("Menu action");
                    });
                };
                let footer = |ui: &mut egui::Ui| ui.button("Save editor").clicked();
                let result = if let Some(side) = state.drawer {
                    Drawer::new("editor", &mut state.open, "Edit record")
                        .side(side)
                        .options(options)
                        .show_with_footer(ui.ctx(), body, footer)
                } else {
                    Modal::new("editor", &mut state.open, "Edit record")
                        .options(options)
                        .show_with_footer(ui.ctx(), body, footer)
                };
                if let Some(result) = result {
                    state.surface = Some(result.response.rect);
                    if let Some(reason) = result.dismissed {
                        state.dismissed.push(reason);
                    }
                    if result.inner.1 {
                        state.saved += 1;
                        state.open = false;
                    }
                }
            },
            State::default(),
        )
}
fn open(h: &mut Harness<'_, State>) {
    h.get_by_label("Open editor").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(5);
}
#[test]
fn named_modal_focus_and_tab_confinement_restore_to_opener() {
    let mut h = harness([800.0, 600.0]);
    open(&mut h);
    let dialog = h.get_by_role_and_label(Role::Dialog, "Edit record");
    assert!(dialog.accesskit_node().data().is_modal());
    assert!(h.get_by_label("Close Edit record").is_focused());
    h.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_label("Save editor").is_focused());
    for modifiers in [
        Modifiers::NONE,
        Modifiers::SHIFT,
        Modifiers::NONE,
        Modifiers::NONE,
        Modifiers::NONE,
        Modifiers::NONE,
    ] {
        h.key_press_modifiers(modifiers, Key::Tab);
        h.run_steps(3);
        assert!(!h.get_by_label("Open editor").is_focused());
        assert!(!h.get_by_label("Outside action").is_focused());
    }
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().open);
    assert_eq!(h.state().dismissed, [OverlayDismissal::Escape]);
    assert!(h.get_by_label("Open editor").is_focused());
    h.check_accessibility();
}
#[test]
fn backdrop_blocks_underlying_pointer_actions_and_can_be_disabled() {
    let mut h = harness([800.0, 600.0]);
    h.state_mut().options.dismiss_on_backdrop = false;
    open(&mut h);
    h.get_by_label("Outside action").click();
    h.run_steps(3);
    assert_eq!(h.state().outside, 0);
    assert!(h.state().open);
    h.state_mut().options.dismiss_on_backdrop = true;
    h.run_steps(2);
    h.get_by_label("Outside action").click();
    h.run_steps(4);
    assert!(!h.state().open);
    assert_eq!(h.state().outside, 0);
    assert_eq!(h.state().dismissed, [OverlayDismissal::Backdrop]);
    assert!(h.get_by_label("Open editor").is_focused());
}
#[test]
fn explicit_initial_focus_host_save_and_reopen_preserve_input() {
    let mut h = harness([800.0, 600.0]);
    h.state_mut().initial_input = true;
    open(&mut h);
    assert!(h.get_by_label("Editor name").is_focused());
    h.get_by_label("Editor name").type_text("Retained value");
    h.run_steps(2);
    h.get_by_label("Save editor").click();
    h.run_steps(4);
    assert_eq!(h.state().saved, 1);
    assert_eq!(h.state().value, "Retained value");
    assert!(h.get_by_label("Open editor").is_focused());
    open(&mut h);
    assert_eq!(h.state().value, "Retained value");
    h.get_by_label("Close Edit record").focus();
    h.run();
    h.key_press(Key::Space);
    h.run_steps(4);
    assert_eq!(h.state().dismissed, [OverlayDismissal::CloseButton]);
    assert_eq!(h.state().saved, 1);
}
#[test]
fn popup_escape_is_consumed_before_the_overlay_and_locked_escape_is_respected() {
    let mut h = harness([800.0, 600.0]);
    open(&mut h);
    h.get_by_label("Editor options").click();
    h.run_steps(3);
    assert!(h.query_by_label("Menu action").is_some());
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.state().open);
    assert!(h.query_by_label("Menu action").is_none());
    h.state_mut().options.dismiss_on_escape = false;
    h.run();
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.state().open);
    h.get_by_label("Close Edit record").click();
    h.run_steps(4);
    assert!(!h.state().open);
}
#[test]
fn external_closure_restores_focus_and_stable_input_survives_insertions() {
    let mut h = harness([800.0, 600.0]);
    h.state_mut().initial_input = true;
    open(&mut h);
    let id = h.get_by_label("Editor name").accesskit_node().id();
    h.state_mut().inserted = true;
    h.run_steps(3);
    assert_eq!(id, h.get_by_label("Editor name").accesskit_node().id());
    assert!(h.get_by_label("Editor name").is_focused());
    h.state_mut().open = false;
    h.run_steps(4);
    assert!(h.get_by_label("Open editor").is_focused());
}
#[test]
fn drawer_edges_fill_the_viewport_and_respect_narrow_bounds() {
    for side in [DrawerSide::Left, DrawerSide::Right] {
        let mut h = harness([280.0, 460.0]);
        h.state_mut().drawer = Some(side);
        open(&mut h);
        let rect = h.state().surface.unwrap();
        assert!(rect.left() >= -0.5 && rect.right() <= 280.5, "{rect:?}");
        assert!(
            rect.top().abs() <= 0.5 && (rect.bottom() - 460.0).abs() <= 0.5,
            "{rect:?}"
        );
        h.key_press(Key::Escape);
        h.run_steps(4);
        assert!(!h.state().open);
        assert!(h.get_by_label("Open editor").is_focused());
        h.check_accessibility();
    }
}
#[test]
fn explicit_return_target_handles_removed_opener() {
    let mut h = harness([800.0, 600.0]);
    open(&mut h);
    h.state_mut().hide_opener = true;
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(h.get_by_label("Focus fallback").is_focused());
}

#[test]
fn topmost_overlay_dismisses_first_and_restores_the_underlying_modal_focus() {
    let mut h = Harness::builder()
        .with_size([800.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut (bool, bool)| {
                if ui.button("Open parent").clicked() {
                    state.0 = true;
                }
                let parent = Modal::new("parent", &mut state.0, "Parent dialog")
                    .show(ui.ctx(), |ui| ui.button("Open child").clicked());
                if parent.is_some_and(|r| r.inner) {
                    state.1 = true;
                }
                Modal::new("child", &mut state.1, "Child dialog").show(ui.ctx(), |ui| {
                    ui.label("Secondary task");
                });
            },
            (false, false),
        );
    h.get_by_label("Open parent").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    h.get_by_label("Open child").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(h.state().0 && h.state().1);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(h.state().0 && !h.state().1);
    assert!(h.get_by_label("Open child").is_focused());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().0);
    assert!(h.get_by_label("Open parent").is_focused());
}
#[test]
fn read_only_dialog_without_close_button_can_use_content_dismissal() {
    let mut h = Harness::builder()
        .with_size([600.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, open: &mut bool| {
                Modal::new("read-only", open, "Read-only details")
                    .options(OverlayOptions {
                        show_close_button: false,
                        dismiss_on_escape: false,
                        dismiss_on_backdrop: false,
                        ..Default::default()
                    })
                    .show(ui.ctx(), |ui| {
                        if ui.button("Done reading").clicked() {
                            ui.close();
                        }
                    });
            },
            true,
        );
    h.run_steps(4);
    assert!(h.query_by_label("Close Read-only details").is_none());
    assert!(
        h.get_by_role_and_label(Role::Heading, "Read-only details")
            .is_focused()
    );
    h.key_press(Key::Tab);
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(!h.state());
}

#[test]
fn close_target_stays_at_the_header_edge_with_token_padding() {
    for side in [None, Some(DrawerSide::Left), Some(DrawerSide::Right)] {
        let mut h = harness([800.0, 600.0]);
        h.state_mut().drawer = side;
        open(&mut h);
        let surface = h.state().surface.unwrap();
        let close = h.get_by_label("Close Edit record").rect();
        let stroke = if side.is_none() { 4.0 } else { 0.0 };
        assert!(
            (surface.right() - close.right() - 24.0 - stroke).abs() < 0.5,
            "{side:?}: {surface:?}, {close:?}"
        );
        assert!((close.top() - surface.top() - 16.0 - stroke).abs() < 0.5);
        assert_eq!(close.size(), egui::vec2(32.0, 32.0));
    }
}
