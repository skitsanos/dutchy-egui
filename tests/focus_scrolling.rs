//! Retained focus never pulls a scroll area back; focus reveals a control once.
mod common;
#[path = "common/media.rs"]
#[allow(dead_code)]
mod media;
use dutchy_egui::{
    FileItem, FileList, FileStatus, Image, Theme, Toast, ToastQueue, ToastVariant, Toasts,
};
use egui::{Event, Modifiers, MouseWheelUnit, Pos2, Rect, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use std::sync::{Arc, Mutex};

struct Page {
    viewport: Rect,
    tail: Rect,
    offset: f32,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            viewport: Rect::NOTHING,
            tail: Rect::NOTHING,
            offset: 0.0,
        }
    }
}
fn page() -> Harness<'static, Page> {
    let probe = Arc::new(Mutex::new(media::Probe::default()));
    Harness::builder()
        .with_size([640.0, 420.0])
        .with_context_setup(move |ctx| {
            common::install(ctx, Theme::default().with_reduced_motion(true));
            media::install(ctx, probe.clone());
        })
        .build_ui_state(
            |ui, state: &mut Page| {
                let output = egui::ScrollArea::vertical()
                    .id_salt("page")
                    .animated(false)
                    .show(ui, |ui| {
                        let files = [FileItem::new(1u8, "alpha.pdf")
                            .size(1200)
                            .status(FileStatus::Ready)];
                        FileList::new("files", &files).show(ui);
                        Image::new(
                            "picture",
                            egui::ImageSource::Uri("fixture://photo".into()),
                            "Test artwork",
                        )
                        .width(300.0)
                        .expandable(true)
                        .show(ui);
                        ui.add_space(600.0);
                        state.tail = ui.label("Tail marker").rect;
                    });
                state.offset = output.state.clamped_offset().y;
                state.viewport = output.inner_rect;
            },
            Page::default(),
        )
}
fn wheel<S>(h: &mut Harness<'_, S>, at: Pos2, delta: f32) {
    h.hover_at(at);
    h.run_steps(2);
    h.input_mut().events.push(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: vec2(0.0, delta),
        modifiers: Modifiers::NONE,
    });
    h.run_steps(80);
}
#[test]
fn focused_file_row_action_does_not_pull_the_page_back() {
    let mut h = page();
    h.run_steps(3);
    h.get_by_label("Remove file alpha.pdf").focus();
    h.run_steps(3);
    let center = h.state().viewport.center();
    wheel(&mut h, center, -1500.0);
    assert!(
        h.state().viewport.contains_rect(h.state().tail),
        "offset={}",
        h.state().offset
    );
    assert!(h.get_by_label("Remove file alpha.pdf").is_focused());
}
#[test]
fn focused_image_control_does_not_pull_the_page_back() {
    let mut h = page();
    h.run_steps(3);
    h.get_by_label("Expand Test artwork").focus();
    h.run_steps(3);
    let center = h.state().viewport.center();
    wheel(&mut h, center, -1500.0);
    assert!(
        h.state().viewport.contains_rect(h.state().tail),
        "offset={}",
        h.state().offset
    );
    assert!(h.get_by_label("Expand Test artwork").is_focused());
}
struct Stack {
    queue: ToastQueue<u8>,
    rects: Vec<Rect>,
    stack: Rect,
}
fn stack() -> Harness<'static, Stack> {
    let mut queue = ToastQueue::new();
    for title in ["One", "Two", "Three", "Four"] {
        queue.push(
            Toast::new("A notification body that needs some room.")
                .title(title)
                .variant(ToastVariant::Info)
                .timeout(None)
                .action("Undo", 0u8),
        );
    }
    Harness::builder()
        .with_size([420.0, 320.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut Stack| {
                let result = Toasts::new("messages", &mut state.queue)
                    .max_visible(4)
                    .show(ui.ctx());
                state.rects = result.visible.iter().map(|row| row.response.rect).collect();
                state.stack = result.response.map_or(Rect::NOTHING, |r| r.rect);
            },
            Stack {
                queue,
                rects: Vec::new(),
                stack: Rect::NOTHING,
            },
        )
}
#[test]
fn focused_toast_control_does_not_pull_the_stack_back() {
    let mut h = stack();
    h.run_steps(3);
    assert_eq!(h.state().rects.len(), 4);
    h.get_by_label("Dismiss Four").focus();
    h.run_steps(4);
    assert!(
        h.state().rects[0].top() < 0.0,
        "focus should reveal the last entry first"
    );
    let center = h.state().stack.center();
    wheel(&mut h, center, 1500.0);
    assert!(
        h.state().rects[0].top() >= 20.0,
        "first entry top={} after scrolling up",
        h.state().rects[0].top()
    );
    assert!(h.get_by_label("Dismiss Four").is_focused());
}
