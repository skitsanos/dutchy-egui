use dutchy_egui::{Button, ControlSize, Input, SplitPane, Theme};
use egui::{Id, Key, Modifiers, Pos2, Rect, Ui, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

pub struct Page {
    pub first: f32,
    pub vertical: bool,
    pub width: f32,
    pub height: f32,
    pub minima: [f32; 2],
    pub enabled: bool,
    pub read_only: bool,
    pub parent_disabled: bool,
    pub discard: bool,
    pub inserted: bool,
    pub text: String,
    pub rects: [Rect; 3],
    pub outer: Rect,
    pub below: Rect,
    pub first_id: Id,
    pub divider_id: Id,
    pub changes: usize,
    pub first_calls: usize,
    pub second_calls: usize,
    pub remaining_keys: Vec<Key>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            first: 160.0,
            vertical: false,
            width: 560.0,
            height: 240.0,
            minima: [80.0, 100.0],
            enabled: true,
            read_only: false,
            parent_disabled: false,
            discard: false,
            inserted: false,
            text: String::new(),
            rects: [Rect::NOTHING; 3],
            outer: Rect::NOTHING,
            below: Rect::NOTHING,
            first_id: Id::NULL,
            divider_id: Id::NULL,
            changes: 0,
            first_calls: 0,
            second_calls: 0,
            remaining_keys: Vec::new(),
        }
    }
}
pub fn harness(page: Page) -> Harness<'static, Page> {
    Harness::builder()
        .with_size([700.0, 500.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(draw, page)
}
pub fn draw(ui: &mut Ui, p: &mut Page) {
    ui.set_max_width(p.width);
    ui.add(Button::new("Before panes").size(ControlSize::Small));
    if p.inserted {
        ui.label("Inserted sibling");
    }
    ui.scope(|ui| {
        if p.parent_disabled {
            ui.disable();
        }
        let split = if p.vertical {
            SplitPane::vertical("panes", &mut p.first)
        } else {
            SplitPane::horizontal("panes", &mut p.first)
        };
        let r = split
            .min_sizes(p.minima[0], p.minima[1])
            .height(p.height)
            .enabled(p.enabled)
            .read_only(p.read_only)
            .accessible_name("Folders")
            .show(
                ui,
                |ui| {
                    p.first_calls += 1;
                    ui.add(Button::new("First pane action").size(ControlSize::Small));
                    ui.add(Input::new("draft", &mut p.text).accessible_name("Draft"));
                    10
                },
                |ui| {
                    p.second_calls += 1;
                    ui.add(Button::new("Second pane action").size(ControlSize::Small));
                    20
                },
            );
        assert_eq!((r.first.inner, r.second.inner), (10, 20));
        assert_eq!(r.response.changed(), r.divider.changed());
        p.rects = [
            r.first.response.rect,
            r.divider.rect,
            r.second.response.rect,
        ];
        p.outer = r.response.rect;
        p.first_id = r.first.response.id;
        p.divider_id = r.divider.id;
        p.changes += usize::from(r.response.changed());
    });
    p.below = ui
        .add(Button::new("After panes").size(ControlSize::Small))
        .rect;
    p.remaining_keys = ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::Key {
                    key, pressed: true, ..
                } => Some(*key),
                _ => None,
            })
            .collect()
    });
    if p.discard {
        ui.ctx()
            .request_discard("exercise split pane on a second pass");
    }
}
pub fn focus(h: &mut Harness<'_, Page>) {
    h.get_by_role_and_label(Role::Splitter, "Folders").focus();
    h.run_steps(3);
}
pub fn key(h: &mut Harness<'_, Page>, key: Key, modifiers: Modifiers) {
    h.event(egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    });
    h.run_steps(2);
}
pub fn button(h: &mut Harness<'_, Page>, pos: Pos2, pressed: bool) {
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
}
