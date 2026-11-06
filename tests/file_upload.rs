//! Native keyboard activation and spatial, modal-safe lazy file-drop claiming.
#![cfg(not(target_arch = "wasm32"))]
mod common;
use dutchy_egui::{FileUpload, FileUploadVariant, Theme};
use egui::{Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[derive(Debug)]
struct LazyFile {
    path: PathBuf,
    reads: Arc<AtomicUsize>,
}
impl egui::DroppedFile for LazyFile {
    fn path(&self) -> &Path {
        &self.path
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        self.reads.fetch_add(1, Ordering::AcqRel);
        Err("must not read in a widget".into())
    }
}
#[derive(Default)]
struct State {
    disabled: bool,
    parent_disabled: bool,
    clipped: bool,
    modal: bool,
    discard: bool,
    browses: usize,
    drops: Vec<(usize, PathBuf)>,
    rects: Vec<Rect>,
    active: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([640.0, 650.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                state.rects.clear();
                let parent = state.parent_disabled;
                ui.add_enabled_ui(!parent, |ui| {
                    if state.clipped {
                        ui.add_space(1000.0);
                    }
                    for index in 0..2 {
                        let result = FileUpload::new(index)
                            .label(if index == 0 {
                                "Drop first files"
                            } else {
                                "Choose second files"
                            })
                            .variant(if index == 0 {
                                FileUploadVariant::DropZone
                            } else {
                                FileUploadVariant::Compact
                            })
                            .hint("PDF and images")
                            .enabled(!state.disabled)
                            .show(ui);
                        state.rects.push(result.response.rect);
                        state.browses += usize::from(result.browse_requested);
                        state.active = result.drag_hovered;
                        for file in result.dropped {
                            state.drops.push((index, file.path().to_owned()));
                        }
                    }
                });
                if state.discard && ui.ctx().current_pass_index() == 0 {
                    ui.ctx().request_discard("test another layout pass");
                }
                if state.modal {
                    egui::Modal::new(egui::Id::unique("blocking")).show(ui.ctx(), |ui| {
                        ui.label("Blocking dialog");
                    });
                }
            },
            State::default(),
        )
}
fn drop_at(h: &mut Harness<'_, State>, position: egui::Pos2) -> Arc<AtomicUsize> {
    let reads = Arc::new(AtomicUsize::new(0));
    h.hover_at(position);
    h.run_steps(2);
    h.input_mut().dropped_files = vec![Arc::new(LazyFile {
        path: "/tmp/project.pdf".into(),
        reads: reads.clone(),
    })];
    h.run_steps(3);
    reads
}
#[test]
fn clicking_and_enter_or_space_request_the_host_picker_without_retaining_files() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_label("Drop first files").click();
    h.run_steps(2);
    h.get_by_label("Choose second files").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    h.key_press(egui::Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().browses, 3);
    assert!(h.state().drops.is_empty());
    h.check_accessibility();
}
#[test]
fn drops_go_only_to_the_visible_target_once_and_never_read_file_contents() {
    let mut h = harness();
    h.run_steps(3);
    let position = h.state().rects[1].center();
    let reads = drop_at(&mut h, position);
    assert_eq!(h.state().drops, [(1, PathBuf::from("/tmp/project.pdf"))]);
    assert_eq!(reads.load(Ordering::Acquire), 0);
    h.run_steps(3);
    assert_eq!(h.state().drops.len(), 1);
    let outside = egui::pos2(630.0, 620.0);
    drop_at(&mut h, outside);
    assert_eq!(h.state().drops.len(), 1);
}
#[test]
fn disabled_clipped_and_modal_obscured_zones_do_not_claim_files_or_browse() {
    for mode in 0..4 {
        let mut h = harness();
        h.run_steps(3);
        let position = h.state().rects[0].center();
        match mode {
            0 => h.state_mut().disabled = true,
            1 => h.state_mut().parent_disabled = true,
            2 => h.state_mut().clipped = true,
            _ => h.state_mut().modal = true,
        }
        h.run_steps(4);
        let reads = drop_at(&mut h, position);
        assert!(h.state().drops.is_empty());
        assert_eq!(reads.load(Ordering::Acquire), 0);
        if mode < 2 {
            assert!(
                h.get_by_role_and_label(Role::Button, "Drop first files")
                    .accesskit_node()
                    .is_disabled()
            );
        }
        assert_eq!(h.state().browses, 0);
    }
}
#[test]
fn a_discarded_layout_pass_does_not_duplicate_the_drop_event() {
    let mut h = harness();
    h.run_steps(3);
    h.state_mut().discard = true;
    let position = h.state().rects[0].center();
    drop_at(&mut h, position);
    assert_eq!(h.state().drops.len(), 1);
}
#[test]
fn external_drag_highlights_the_target_without_adding_a_file() {
    let mut h = harness();
    h.run_steps(3);
    let position = h.state().rects[1].center();
    h.hover_at(position);
    h.run_steps(2);
    h.input_mut().hovered_files = vec![egui::HoveredFile {
        path: Some("/tmp/project.pdf".into()),
        mime: String::new(),
    }];
    h.run_steps(1);
    assert!(h.state().active);
    assert!(h.state().drops.is_empty());
}
