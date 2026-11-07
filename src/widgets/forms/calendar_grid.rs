//! Month grid shown by the date picker: header, weekday row and 42 day cells.

use crate::{
    Text, TextColor, TextSize, Theme, Weight,
    calendar::{Date, Weekday},
    interaction::{focus_is_visible, paint_focus_inside},
    tokens::{border, control, space},
    widgets::nav_icon,
};
use egui::accesskit::Role;
use egui::{
    EventFilter, Id, Key, Modifiers, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    WidgetInfo, pos2, vec2,
};

/// Inclusive optional bounds; days outside are disabled.
#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub min: Option<Date>,
    pub max: Option<Date>,
}

impl Bounds {
    fn allows(self, date: Date) -> bool {
        self.min.is_none_or(|min| date >= min) && self.max.is_none_or(|max| date <= max)
    }

    fn clamp(self, date: Date) -> Date {
        date.clamp_to(self.min, self.max)
    }

    fn offset_month(self, month: Date, delta: i32) -> Option<Date> {
        month.add_months(delta).filter(|month| {
            self.min.is_none_or(|min| *month >= min.first_of_month())
                && self.max.is_none_or(|max| *month <= max.first_of_month())
        })
    }
}

/// What the grid did this frame.
pub(super) enum Outcome {
    Open,
    Selected(Date),
}

#[derive(Clone, Copy)]
struct State {
    month: Date,
    focused: Date,
    pending_entry: bool,
}

fn state_id(id: Id) -> Id {
    id.with("calendar-state")
}

fn day_id(id: Id, date: Date) -> Id {
    id.with(("day", date.days_since_epoch()))
}

/// Start a fresh grid at `anchor` and move keyboard focus onto it.
pub(super) fn reset(ui: &Ui, id: Id, anchor: Date) {
    let state = State {
        month: anchor.first_of_month(),
        focused: anchor,
        pending_entry: true,
    };
    ui.ctx()
        .data_mut(|data| data.insert_temp(state_id(id), state));
}

pub(super) fn show(
    ui: &mut Ui,
    id: Id,
    value: Option<Date>,
    today: Option<Date>,
    bounds: Bounds,
) -> Outcome {
    let anchor = value.or(today).unwrap_or(super::date::FALLBACK);
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<State>(state_id(id)))
        .unwrap_or(State {
            month: anchor.first_of_month(),
            focused: anchor,
            pending_entry: true,
        });
    let clamped = bounds.clamp(state.focused);
    if clamped != state.focused {
        let owns_focus = ui.memory(|m| m.focused() == Some(day_id(id, state.focused)));
        state.focused = clamped;
        state.month = clamped.first_of_month();
        if owns_focus {
            ui.memory_mut(|m| m.request_focus(day_id(id, clamped)));
        }
    }
    navigate(ui, id, &mut state, bounds);
    let theme = Theme::from_context(ui.ctx());
    ui.spacing_mut().item_spacing = vec2(0.0, space::SM);
    if let Some(month) = header(ui, id, state.month, bounds, theme) {
        state.month = month;
        let length = Date::days_in_month(month.year(), month.month());
        state.focused = bounds.clamp(
            Date::new(month.year(), month.month(), state.focused.day().min(length))
                .unwrap_or(month),
        );
    }
    weekdays(ui, theme);
    let outcome = days(ui, id, &state, value, today, bounds, theme);
    if state.pending_entry && !ui.is_sizing_pass() {
        ui.memory_mut(|m| m.request_focus(day_id(id, state.focused)));
        state.pending_entry = false;
        ui.ctx().request_repaint();
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(state_id(id), state));
    outcome
}

/// Arrow, Home/End and PageUp/PageDown movement for the focused day.
fn navigate(ui: &mut Ui, id: Id, state: &mut State, bounds: Bounds) {
    let focused = day_id(id, state.focused);
    if !ui.is_enabled() || ui.memory(|memory| memory.focused()) != Some(focused) {
        return;
    }
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            focused,
            EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                ..Default::default()
            },
        );
    });
    let weekday = i64::from(state.focused.weekday().index());
    let target = ui.input_mut(|input| {
        let mut consume = |key| input.consume_key(Modifiers::NONE, key);
        if consume(Key::ArrowLeft) {
            state.focused.add_days(-1)
        } else if consume(Key::ArrowRight) {
            state.focused.add_days(1)
        } else if consume(Key::ArrowUp) {
            state.focused.add_days(-7)
        } else if consume(Key::ArrowDown) {
            state.focused.add_days(7)
        } else if consume(Key::Home) {
            state.focused.add_days(-weekday)
        } else if consume(Key::End) {
            state.focused.add_days(6 - weekday)
        } else if consume(Key::PageUp) {
            state.focused.add_months(-1)
        } else if consume(Key::PageDown) {
            state.focused.add_months(1)
        } else {
            None
        }
    });
    if let Some(next) = target {
        let next = bounds.clamp(next);
        state.focused = next;
        state.month = next.first_of_month();
        ui.memory_mut(|memory| {
            memory.move_focus(egui::FocusDirection::None);
            memory.request_focus(day_id(id, next));
        });
    }
}

/// Previous/next month controls around the month label; returns a new month.
fn header(ui: &mut Ui, id: Id, month: Date, bounds: Bounds, theme: Theme) -> Option<Date> {
    let p = theme.palette;
    let cell = control::CALENDAR_CELL;
    let (rect, _) = ui.allocate_exact_size(vec2(7.0 * cell, cell), Sense::hover());
    let mut target = None;
    for (right, verb, delta) in [(false, "Previous month", -1), (true, "Next month", 1)] {
        let button_rect = if right {
            Rect::from_min_size(pos2(rect.right() - cell, rect.top()), vec2(cell, cell))
        } else {
            Rect::from_min_size(rect.min, vec2(cell, cell))
        };
        let candidate = bounds.offset_month(month, delta);
        let mut button_ui = ui.new_child(UiBuilder::new().max_rect(button_rect));
        if candidate.is_none() {
            button_ui.disable();
        }
        let response = button_ui.interact(button_rect, id.with(verb), Sense::click());
        response.widget_info(|| WidgetInfo::labeled(Role::Button, button_ui.is_enabled(), verb));
        if response.hovered() {
            button_ui.painter().rect_filled(button_rect, 0, p.muted);
        }
        nav_icon::chevron(
            button_ui.painter(),
            Rect::from_center_size(
                button_rect.center(),
                vec2(control::INDICATOR, control::INDICATOR),
            ),
            p.foreground,
            right,
        );
        paint_focus_inside(&button_ui, &response, p.primary);
        if response.clicked() {
            target = candidate;
        }
    }
    let label = format!("{} {}", Date::month_name(month.month()), month.year());
    let job = Text::new(label.as_str())
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .color(TextColor::Foreground)
        .layout_job(&theme, 5.0 * cell);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let text_rect = Rect::from_center_size(rect.center(), galley.size());
    let title = ui.interact(text_rect, id.with("month-label"), Sense::hover());
    title.widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), label.as_str()));
    crate::text::name_as_written(&title, &label);
    ui.painter().galley(text_rect.min, galley, p.foreground);
    target
}

fn weekdays(ui: &mut Ui, theme: Theme) {
    let cell = control::CALENDAR_CELL;
    let (rect, _) = ui.allocate_exact_size(vec2(7.0 * cell, control::INDICATOR), Sense::hover());
    for (index, weekday) in Weekday::ALL.iter().enumerate() {
        let job = Text::new(weekday.short())
            .size(TextSize::Xs)
            .weight(Weight::Bold)
            .color(TextColor::Muted)
            .uppercase(true)
            .layout_job(&theme, cell);
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let center = pos2(rect.left() + cell * (index as f32 + 0.5), rect.center().y);
        ui.painter().galley(
            center - galley.size() / 2.0,
            galley,
            theme.palette.muted_foreground,
        );
    }
}

fn days(
    ui: &mut Ui,
    id: Id,
    state: &State,
    value: Option<Date>,
    today: Option<Date>,
    bounds: Bounds,
    theme: Theme,
) -> Outcome {
    let cell = control::CALENDAR_CELL;
    let (rect, _) = ui.allocate_exact_size(vec2(7.0 * cell, 6.0 * cell), Sense::hover());
    let first = state.month;
    let start = first
        .add_days(-i64::from(first.weekday().index()))
        .unwrap_or(first);
    let mut outcome = Outcome::Open;
    for index in 0..42 {
        let Some(date) = start.add_days(index) else {
            continue;
        };
        let column = (index % 7) as f32;
        let row = (index / 7) as f32;
        let cell_rect = Rect::from_min_size(
            pos2(rect.left() + column * cell, rect.top() + row * cell),
            vec2(cell, cell),
        );
        let mut cell_ui = ui.new_child(UiBuilder::new().max_rect(cell_rect));
        if !bounds.allows(date) {
            cell_ui.disable();
        }
        let sense = if date == state.focused {
            Sense::click()
        } else {
            Sense::CLICK
        };
        let response = cell_ui.interact(cell_rect, day_id(id, date), sense);
        let label = format!(
            "{} {} {}",
            date.day(),
            Date::month_name(date.month()),
            date.year()
        );
        let selected = value == Some(date);
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, cell_ui.is_enabled(), selected, label.as_str())
        });
        if response.clicked() {
            outcome = Outcome::Selected(date);
        }
        paint_day(
            &cell_ui,
            &response,
            cell_rect,
            date,
            DayLook {
                selected,
                today: today == Some(date),
                adjacent: date.month() != first.month(),
            },
            theme,
        );
    }
    outcome
}

#[derive(Clone, Copy)]
struct DayLook {
    selected: bool,
    today: bool,
    adjacent: bool,
}

fn paint_day(ui: &Ui, response: &Response, rect: Rect, date: Date, look: DayLook, theme: Theme) {
    let p = theme.palette;
    if look.selected {
        ui.painter().rect_filled(rect, 0, p.primary);
    } else if ui.is_enabled() && (response.hovered() || focus_is_visible(response)) {
        ui.painter().rect_filled(rect, 0, p.muted);
    }
    if look.today && !look.selected {
        ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(border::THIN, p.primary),
            StrokeKind::Inside,
        );
    }
    let color = if look.selected {
        TextColor::Custom(p.on_primary)
    } else if look.adjacent {
        TextColor::Muted
    } else {
        TextColor::Foreground
    };
    let job = Text::new(date.day().to_string())
        .size(TextSize::Sm)
        .weight(if look.selected || look.today {
            Weight::Bold
        } else {
            Weight::Regular
        })
        .color(color)
        .layout_job(&theme, rect.width());
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, p.foreground);
    paint_focus_inside(
        ui,
        response,
        if look.selected {
            p.on_primary
        } else {
            p.primary
        },
    );
}
