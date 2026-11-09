use super::MenuItem;
use crate::widgets::navigation::{self, Axis};
use egui::{EventFilter, Id, Key, Modifiers, Rect, Response, Ui};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum InputMode {
    #[default]
    Keyboard,
    Pointer,
}

#[derive(Clone, Default)]
pub(super) struct Memory {
    pub path: Vec<Id>,
    pub levels: Vec<Vec<Id>>,
    pub rects: Vec<Rect>,
    pub pending: Option<usize>,
    pub direction: i8,
    last_frame: u64,
    mode: InputMode,
    pointer_pos: Option<egui::Pos2>,
}

impl Memory {
    pub fn prepare<T>(&mut self, ui: &Ui, root: Id, items: &[MenuItem<'_, T>], bar: bool) {
        if self.last_frame + 1 < ui.ctx().cumulative_frame_nr() {
            self.path.clear();
            self.pending = None;
            self.mode = InputMode::Keyboard;
        }
        self.last_frame = ui.ctx().cumulative_frame_nr();
        self.direction = 0;
        self.update_mode(ui);
        let old_ids: Vec<_> = self.levels.iter().flatten().copied().collect();
        let mut levels = Vec::new();
        let mut lists = Vec::new();
        let mut list = items;
        let mut scope = root;
        loop {
            let ids: Vec<_> = list
                .iter()
                .filter(|i| i.interactive() && ui.is_enabled())
                .map(|i| scope.with(i.id))
                .collect();
            levels.push(ids);
            lists.push((scope, list));
            let depth = levels.len() - 1;
            let next = self.path.get(depth).and_then(|id| {
                list.iter()
                    .find(|i| scope.with(i.id) == *id && i.interactive())
            });
            let Some((id, children)) = next.and_then(|i| i.children.map(|c| (scope.with(i.id), c)))
            else {
                self.path.truncate(depth);
                break;
            };
            scope = id.with("children");
            list = children;
        }
        recover_escape(ui, &levels);
        let focused = ui.memory(|m| m.focused());
        let depth = levels
            .iter()
            .position(|ids| focused.is_some_and(|id| ids.contains(&id)));
        if let Some(depth) = depth {
            navigation::navigate(ui, &levels[depth], Axis::Vertical);
            let focused = ui.memory(|m| m.focused());
            let (scope, list) = lists[depth];
            if let Some(item) = list.iter().find(|i| Some(scope.with(i.id)) == focused) {
                let id = scope.with(item.id);
                if self.mode == InputMode::Keyboard
                    && self.path.get(depth).is_some_and(|open| *open != id)
                {
                    self.path.truncate(depth);
                    self.pending = None;
                }
                self.horizontal(ui, depth, id, item.children.is_some(), bar);
            }
        } else if focused.is_some_and(|id| old_ids.contains(&id)) {
            if let Some(id) = levels
                .last()
                .and_then(|ids| ids.first())
                .or_else(|| levels[0].first())
            {
                ui.memory_mut(|m| m.request_focus(*id));
            } else if let Some(id) = focused {
                ui.memory_mut(|m| m.surrender_focus(id));
            }
        }
        self.levels = levels;
    }

    fn horizontal(&mut self, ui: &Ui, depth: usize, id: Id, branch: bool, bar: bool) {
        if !ui.is_enabled() || !ui.memory(|m| m.allows_interaction(ui.layer_id())) {
            return;
        }
        let right = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight));
        if right {
            self.mode = InputMode::Keyboard;
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            if branch {
                self.enter(depth, id, true);
            } else if bar {
                self.direction = 1;
            }
        }
        let left = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft));
        let escape = depth > 0 && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if left || escape {
            self.mode = InputMode::Keyboard;
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            if depth > 0 {
                let parent = self.path[depth - 1];
                self.path.truncate(depth - 1);
                self.pending = None;
                ui.memory_mut(|m| {
                    m.move_focus(egui::FocusDirection::None);
                    m.request_focus(parent);
                });
            } else if bar && left {
                self.direction = -1;
            }
        }
    }

    pub fn enter(&mut self, depth: usize, id: Id, keyboard: bool) {
        self.path.truncate(depth);
        self.path.push(id);
        if keyboard {
            self.pending = Some(depth + 1);
        }
    }

    pub fn pointer(&mut self, ui: &Ui, response: &Response, depth: usize, branch: bool) {
        if !response.enabled() {
            return;
        }
        if response.clicked() && branch {
            let keyboard = !response.clicked_by(egui::PointerButton::Primary);
            focus(response);
            self.mode = if keyboard {
                InputMode::Keyboard
            } else {
                InputMode::Pointer
            };
            self.enter(depth, response.id, keyboard);
        } else if self.pointer_mode() && response.hovered() {
            let towards_child = self
                .rects
                .get(depth + 1)
                .is_some_and(|rect| ui.input(|i| i.pointer.is_moving_towards_rect(rect)));
            if !towards_child {
                let owns_focus = ui
                    .memory(|m| m.focused())
                    .is_some_and(|id| self.levels.iter().any(|ids| ids.contains(&id)));
                if owns_focus && !ui.is_sizing_pass() && !response.has_focus() {
                    focus(response);
                }
                if branch {
                    self.enter(depth, response.id, false);
                } else {
                    self.path.truncate(depth);
                }
            }
        }
    }

    pub fn focus(&mut self, ui: &Ui, depth: usize, rows: &[Response]) {
        if ui.is_sizing_pass() || self.pending.is_none_or(|d| d != depth) {
            return;
        }
        self.pending = None;
        let row = rows.iter().find(|r| r.enabled());
        if let Some(row) = row {
            focus(row);
        }
    }

    pub fn pointer_mode(&self) -> bool {
        self.mode == InputMode::Pointer
    }
    pub fn keyboard(&mut self) {
        self.mode = InputMode::Keyboard;
    }

    fn update_mode(&mut self, ui: &Ui) {
        ui.input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::PointerMoved(pos) if self.pointer_pos != Some(*pos) => {
                        self.pointer_pos = Some(*pos);
                        self.mode = InputMode::Pointer;
                    }
                    egui::Event::PointerButton { pos, .. } => {
                        self.pointer_pos = Some(*pos);
                        self.mode = InputMode::Pointer;
                    }
                    egui::Event::MouseWheel { .. } => self.mode = InputMode::Pointer,
                    egui::Event::Key { pressed: true, .. }
                    | egui::Event::AccessKitActionRequest(_) => self.mode = InputMode::Keyboard,
                    _ => {}
                }
            }
        });
    }
}

// egui clears focus on Escape before rendering when a newly focused item has
// not yet had a pass in which its lock filter can be installed. Recover only
// a descendant that owned focus at the end of the previous pass. An external
// editor that took focus after the menu was drawn must never be displaced.
fn recover_escape(ui: &Ui, levels: &[Vec<Id>]) {
    if ui.memory(|m| m.focused().is_some()) || !ui.input(|i| i.key_pressed(Key::Escape)) {
        return;
    }
    if let Some(id) = crate::widgets::popup_focus::previous_focus(ui.ctx())
        .filter(|id| levels.iter().skip(1).any(|ids| ids.contains(id)))
    {
        ui.memory_mut(|m| m.request_focus(id));
    }
}

pub(crate) fn focus(response: &Response) {
    response.ctx.memory_mut(|m| {
        m.move_focus(egui::FocusDirection::None);
        m.request_focus(response.id);
    });
    lock(response, true);
    response.ctx.request_repaint();
}

/// Keep arrow keys for menu navigation. Rows also keep Escape, which closes
/// one level; triggers leave it to egui so it releases focus as elsewhere.
pub(crate) fn lock(response: &Response, escape: bool) {
    if response.has_focus() {
        response.ctx.memory_mut(|m| {
            m.set_focus_lock_filter(
                response.id,
                EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape,
                    ..Default::default()
                },
            )
        });
    }
}
