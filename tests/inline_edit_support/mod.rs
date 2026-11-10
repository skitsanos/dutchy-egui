use dutchy_egui::{ControlSize, InlineEdit, InlineEditEvent, InlineEditState, Input, Theme};
use egui::{Key, Modifiers, Rect, Ui};
use egui_kittest::{Harness, kittest::Queryable};

pub const NAME: &str = "Document name";
pub struct Page {
    pub value: String,
    pub edit: InlineEditState,
    pub events: Vec<InlineEditEvent>,
    pub width: f32,
    pub size: ControlSize,
    pub enabled: bool,
    pub read_only: bool,
    pub parent_disabled: bool,
    pub prefix: bool,
    pub shown: bool,
    pub apply: bool,
    /// Disable the control after a commit, as a host persisting the name would.
    pub hold_on_commit: bool,
    pub discard: bool,
    pub rect: Rect,
    pub below: Rect,
    pub changes: usize,
    pub other: String,
    pub keys: Vec<Key>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            value: "Design notes.md".into(),
            edit: InlineEditState::default(),
            events: vec![],
            width: 400.0,
            size: ControlSize::Small,
            enabled: true,
            read_only: false,
            parent_disabled: false,
            prefix: false,
            shown: true,
            apply: true,
            hold_on_commit: false,
            discard: false,
            rect: Rect::NOTHING,
            below: Rect::NOTHING,
            changes: 0,
            other: String::new(),
            keys: vec![],
        }
    }
}
pub fn draw(ui: &mut Ui, p: &mut Page) {
    let _ = ui.button("Before rename");
    if p.prefix {
        ui.label("Inserted sibling");
    }
    if p.shown {
        ui.add_enabled_ui(!p.parent_disabled, |ui| {
            let result = InlineEdit::new("document", &p.value, &mut p.edit)
                .accessible_name(NAME)
                .width(p.width)
                .size(p.size)
                .char_limit(80)
                .enabled(p.enabled)
                .read_only(p.read_only)
                .show(ui);
            assert_eq!(result.editing, p.edit.is_editing());
            p.rect = result.response.rect;
            p.changes += usize::from(result.response.changed());
            if let Some(event) = result.event {
                if let InlineEditEvent::Commit(value) = &event
                    && p.apply
                {
                    p.value = value.clone();
                }
                if matches!(event, InlineEditEvent::Commit(_)) && p.hold_on_commit {
                    p.enabled = false;
                }
                p.events.push(event);
            }
        });
    }
    p.below = ui.button("After rename").rect;
    ui.add(Input::new("other", &mut p.other).accessible_name("Other field"));
    p.keys.extend(ui.input(|input| {
        input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Key {
                    key, pressed: true, ..
                } => Some(*key),
                _ => None,
            })
            .collect::<Vec<_>>()
    }));
    if p.discard {
        ui.ctx().request_discard("rename second pass");
    }
}
pub fn harness(page: Page) -> Harness<'static, Page> {
    harness_stepped(page, 0.25)
}
/// A harness whose frames are `step_dt` seconds apart; multi-click gestures
/// need display-rate frames to fit egui's double-click delay.
pub fn harness_stepped(page: Page, step_dt: f32) -> Harness<'static, Page> {
    let mut h = Harness::builder()
        .with_size([560.0, 420.0])
        .with_step_dt(step_dt)
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(draw, page);
    h.run_steps(3);
    h
}
pub fn start(h: &mut Harness<'_, Page>) {
    h.get_by_label("Rename Document name").click();
    h.run_steps(3);
}
pub fn key(h: &mut Harness<'_, Page>, key: Key, modifiers: Modifiers) {
    h.state_mut().keys.clear();
    h.event(egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    });
    h.run_steps(3);
    h.event(egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: false,
        repeat: false,
        modifiers,
    });
    h.run_steps(1);
}
