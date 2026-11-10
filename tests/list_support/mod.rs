use dutchy_egui::{List, ListItem, ListSelectionMode, ListState, Theme};
use egui::{Id, Rect};
use egui_kittest::Harness;
use std::ops::Range;

pub struct Page {
    pub state: ListState<u32>,
    pub items: Vec<ListItem<'static, u32>>,
    pub mode: ListSelectionMode,
    pub enabled: bool,
    pub read_only: bool,
    pub changed: bool,
    pub changes: usize,
    pub activated: Vec<u32>,
    pub rendered: usize,
    pub range: Range<usize>,
    pub viewport: Rect,
    pub row_ids: Vec<Id>,
    pub hovered: Option<u32>,
}
impl Page {
    pub fn new(count: u32) -> Self {
        Self {
            state: ListState::default(),
            items: (0..count)
                .map(|i| ListItem::new(i, format!("Row {i:05}")))
                .collect(),
            mode: ListSelectionMode::Single,
            enabled: true,
            read_only: false,
            changed: false,
            changes: 0,
            activated: Vec::new(),
            rendered: 0,
            range: 0..0,
            viewport: Rect::NOTHING,
            row_ids: Vec::new(),
            hovered: None,
        }
    }
}
pub fn show(ui: &mut egui::Ui, page: &mut Page) {
    let _ = ui.button("Before list");
    page.rendered = 0;
    let output = List::new("records", &mut page.state, &page.items)
        .selection_mode(page.mode)
        .accessible_name("Records")
        .max_height(192.0)
        .enabled(page.enabled)
        .read_only(page.read_only)
        .show_with(ui, |ui, item, _| {
            page.rendered += 1;
            ui.label(item.label.as_ref());
        });
    page.changed = output.response.changed();
    page.changes += usize::from(page.changed);
    page.range = output.visible_range;
    page.viewport = output.viewport;
    page.row_ids = output.rows.iter().map(|row| row.response.id).collect();
    page.hovered = output
        .rows
        .iter()
        .find(|row| row.response.hovered())
        .map(|row| row.key);
    page.activated.extend(output.activated);
    let _ = ui.button("After list");
}
pub fn harness(page: Page) -> Harness<'static, Page> {
    let mut h = Harness::builder()
        .with_size([400.0, 330.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(show, page);
    h.run_steps(3);
    h
}
pub fn focus(h: &mut Harness<'_, Page>, key: u32) {
    use egui_kittest::kittest::Queryable;
    h.get_by_role_and_label(
        if h.state().mode == ListSelectionMode::None {
            egui::accesskit::Role::ListItem
        } else {
            egui::accesskit::Role::ListBoxOption
        },
        &format!("Row {key:05}"),
    )
    .focus();
    h.run_steps(3);
}
pub fn key(h: &mut Harness<'_, Page>, key: egui::Key, modifiers: egui::Modifiers) {
    h.key_press_modifiers(modifiers, key);
    h.run_steps(3);
}
/// Type one character as native backends do: Space also arrives as a key press.
pub fn type_char(h: &mut Harness<'_, Page>, c: char) {
    if c == ' ' {
        h.event(egui::Event::Key {
            key: egui::Key::Space,
            physical_key: Some(egui::Key::Space),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.event(egui::Event::Text(c.to_string()));
    h.run_steps(1);
}
pub fn labeled(labels: &[&'static str]) -> Vec<ListItem<'static, u32>> {
    (0..)
        .zip(labels)
        .map(|(key, label)| ListItem::new(key, *label))
        .collect()
}
pub fn selected(h: &Harness<'_, Page>) -> Vec<u32> {
    let mut keys: Vec<_> = h.state().state.selected.iter().copied().collect();
    keys.sort_unstable();
    keys
}
