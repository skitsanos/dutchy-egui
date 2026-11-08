//! One task row: a focusable list item around a Checkbox, the title and badges.
//!
//! The row is the keyboard anchor. While it has focus, Up/Down move between
//! rows, Enter edits, Space toggles and Delete removes; the Checkbox inside
//! keeps its own pointer and keyboard activation.

use crate::store::{Priority, Task, TaskId};
use dutchy_egui::{
    Badge, BadgeVariant, Checkbox, Theme,
    calendar::Date,
    interaction::paint_focus,
    tokens::{border, space},
};
use egui::{
    EventFilter, FocusDirection, Id, Key, Modifiers, Rect, Response, Sense, Ui, UiBuilder,
    WidgetInfo, accesskit::Role, vec2,
};

/// Row height in points.
pub const HEIGHT: f32 = 56.0;

/// Keyboard request from the focused row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowAction {
    /// Enter: open the editor.
    Edit(TaskId),
    /// Space: flip completion.
    Toggle(TaskId),
    /// Delete or Backspace: remove the task.
    Delete(TaskId),
}

/// What one row reported this frame.
pub struct Outcome {
    /// The row surface.
    pub response: Response,
    /// Checkbox changed to this value.
    pub done: Option<bool>,
    /// Double-click asked for the editor.
    pub edit: bool,
}

/// Stable egui id of a row, independent of layout.
pub fn id(task: TaskId) -> Id {
    Id::unique(("dutchy-todo-row", task.0))
}

/// Focus a row and keep vertical arrows for the list.
pub fn focus(ctx: &egui::Context, task: TaskId) {
    ctx.memory_mut(|memory| {
        memory.move_focus(FocusDirection::None);
        memory.request_focus(id(task));
        lock(memory, id(task));
    });
}

fn lock(memory: &mut egui::Memory, row: Id) {
    memory.set_focus_lock_filter(
        row,
        EventFilter {
            vertical_arrows: true,
            ..Default::default()
        },
    );
}

/// Handle the keys of the focused row before the rows are drawn. Arrow moves
/// are applied here; other requests are returned for the host.
pub fn keys(ui: &Ui, visible: &[TaskId]) -> Option<RowAction> {
    let focused = ui.memory(|memory| memory.focused())?;
    let index = visible.iter().position(|task| id(*task) == focused)?;
    ui.memory_mut(|memory| lock(memory, focused));
    let last = visible.len() - 1;
    let moved = ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
            Some((index + 1).min(last))
        } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
            Some(index.saturating_sub(1))
        } else if input.consume_key(Modifiers::NONE, Key::Home) {
            Some(0)
        } else if input.consume_key(Modifiers::NONE, Key::End) {
            Some(last)
        } else {
            None
        }
    });
    if let Some(next) = moved {
        focus(ui.ctx(), visible[next]);
        return None;
    }
    let current = visible[index];
    ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::Enter) {
            Some(RowAction::Edit(current))
        } else if input.consume_key(Modifiers::NONE, Key::Space) {
            Some(RowAction::Toggle(current))
        } else if input.consume_key(Modifiers::NONE, Key::Delete)
            || input.consume_key(Modifiers::NONE, Key::Backspace)
        {
            Some(RowAction::Delete(current))
        } else {
            None
        }
    })
}

/// Draw one row. `today` decides the due badge.
pub fn show(ui: &mut Ui, task: &Task, today: Date) -> Outcome {
    let theme = Theme::from_context(ui.ctx());
    let p = theme.palette;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    // The surface is registered before its children, so the Checkbox and
    // badges receive their own pointer events.
    let response = ui.interact(rect, id(task.id), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::ListItem, ui.is_enabled(), &task.title));
    describe(&response, task, today);
    if ui.is_rect_visible(rect) {
        if response.hovered() || response.has_focus() {
            ui.painter().rect_filled(rect, 0, p.muted);
        }
        let divider = Rect::from_min_size(
            rect.left_bottom() - vec2(0.0, border::THIN),
            vec2(rect.width(), border::THIN),
        );
        ui.painter().rect_filled(divider, 0, p.border);
    }
    let inner = rect.shrink2(vec2(space::MD, 0.0));
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(("row", task.id.0))
            .max_rect(inner)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    child.spacing_mut().item_spacing.x = space::SM;
    badges(&mut child, task, today);
    let mut done = task.done;
    let checkbox = child.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        if done {
            ui.visuals_mut().override_text_color = Some(p.muted_foreground);
        }
        ui.add(Checkbox::new(
            ("task", task.id.0),
            &mut done,
            task.title.as_str(),
        ))
    });
    paint_focus(ui, &response, p.primary);
    Outcome {
        edit: response.double_clicked(),
        done: checkbox.inner.changed().then_some(done),
        response,
    }
}

fn describe(response: &Response, task: &Task, today: Date) {
    let mut parts = vec![format!("{} priority", task.priority.name())];
    if let Some(due) = task.due {
        parts.push(format!("due {due}{}", due_suffix(due, today, task.done)));
    }
    parts.push(if task.done {
        "done".into()
    } else {
        "open".into()
    });
    let description = format!(
        "{}. Enter edits, Space toggles, Delete removes.",
        parts.join(", ")
    );
    response
        .ctx
        .accesskit_node_builder(response.id, |node| node.set_description(description));
}

fn due_suffix(due: Date, today: Date, done: bool) -> &'static str {
    if done {
        ""
    } else if due < today {
        ", overdue"
    } else if due == today {
        ", today"
    } else {
        ""
    }
}

fn badges(ui: &mut Ui, task: &Task, today: Date) {
    if let Some(due) = task.due {
        let (label, variant) = due_badge(due, today, task.done);
        ui.add(Badge::new(label).variant(variant));
    }
    let variant = if task.done {
        BadgeVariant::Muted
    } else {
        match task.priority {
            Priority::High => BadgeVariant::Destructive,
            Priority::Medium => BadgeVariant::Warning,
            Priority::Low => BadgeVariant::Muted,
        }
    };
    ui.add(Badge::new(task.priority.name()).variant(variant));
}

fn due_badge(due: Date, today: Date, done: bool) -> (String, BadgeVariant) {
    let short = format!("{} {}", due.day(), &Date::month_name(due.month())[..3]);
    if done {
        (short, BadgeVariant::Muted)
    } else if due < today {
        (format!("Overdue {short}"), BadgeVariant::Destructive)
    } else if due == today {
        ("Today".into(), BadgeVariant::Primary)
    } else {
        (short, BadgeVariant::Outline)
    }
}
