use dutchy_egui::{Button, ControlSize, StatusAlign, StatusBar, StatusItem, Theme};
use egui::{Key, Modifiers, Rect, Ui};
use egui_kittest::Harness;

pub struct Page {
    pub width: f32,
    pub items: Vec<StatusItem<'static, u32>>,
    pub enabled: bool,
    pub clicked: Vec<u32>,
    pub hidden: Vec<u32>,
    pub segments: Vec<(u32, Rect)>,
    pub bar: Rect,
    pub below: Rect,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            width: 960.0,
            items: mail_items(),
            enabled: true,
            clicked: Vec::new(),
            hidden: Vec::new(),
            segments: Vec::new(),
            bar: Rect::NOTHING,
            below: Rect::NOTHING,
        }
    }
}
/// Two counts, a clickable sync segment and two end-aligned hints.
/// Priorities: Unread 3, Selected 1, Sync 2, both hints 0.
pub fn mail_items() -> Vec<StatusItem<'static, u32>> {
    vec![
        StatusItem::labeled(1, "Unread", "42").priority(3),
        StatusItem::labeled(2, "Selected", "3").priority(1),
        StatusItem::labeled(3, "Sync", "Up to date")
            .clickable(true)
            .priority(2),
        StatusItem::hint(4, "Ctrl+K", "Commands").align(StatusAlign::End),
        StatusItem::hint(5, "N", "Compose").align(StatusAlign::End),
    ]
}
pub fn harness(page: Page) -> Harness<'static, Page> {
    Harness::builder()
        .with_size([1000.0, 200.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(draw, page)
}
pub fn draw(ui: &mut Ui, p: &mut Page) {
    ui.set_max_width(p.width);
    ui.add(Button::new("Before status").size(ControlSize::Small));
    let r = StatusBar::new("status", &p.items)
        .enabled(p.enabled)
        .show(ui);
    p.clicked.extend(r.clicked);
    p.hidden = r.hidden;
    p.segments = r.segments.iter().map(|(k, s)| (*k, s.rect)).collect();
    p.bar = r.response.rect;
    p.below = ui
        .add(Button::new("After status").size(ControlSize::Small))
        .rect;
}
/// The rectangle of a visible segment.
pub fn segment(p: &Page, key: u32) -> Rect {
    p.segments
        .iter()
        .find_map(|(k, r)| (*k == key).then_some(*r))
        .unwrap_or_else(|| panic!("segment {key} is not visible"))
}
pub fn key(h: &mut Harness<'_, Page>, key: Key) {
    h.event(egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
}
