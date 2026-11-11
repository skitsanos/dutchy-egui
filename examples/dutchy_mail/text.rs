//! Single-line row text and dates.

use dutchy_egui::{Text, TextColor, TextSize, Theme, Weight, calendar::Date};
use egui::{Label, Ui};
use std::ops::Range;

/// One elided, non-selectable line that leaves clicks to its row.
pub fn line(ui: &mut Ui, text: &str, size: TextSize, bold: bool, color: TextColor) {
    marked_line(ui, text, &[], size, bold, color);
}

/// A [`line`] that emphasizes `marks`, the byte ranges a search matched.
pub fn marked_line(
    ui: &mut Ui,
    text: &str,
    marks: &[Range<usize>],
    size: TextSize,
    bold: bool,
    color: TextColor,
) {
    let theme = Theme::from_context(ui.ctx());
    let weight = if bold { Weight::Bold } else { Weight::Regular };
    let mut job = Text::new(text)
        .size(size)
        .weight(weight)
        .color(color)
        .highlight(marks)
        .layout_job(&theme, ui.available_width().max(0.0));
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.add(Label::new(galley).selectable(false));
}

/// "Today", a weekday this week, or a short date.
pub fn date(date: Date, today: Date) -> String {
    let days = today.days_since_epoch() - date.days_since_epoch();
    match days {
        0 => "Today".to_owned(),
        1 => "Yesterday".to_owned(),
        2..=6 => date.weekday().name().to_owned(),
        _ if date.year() == today.year() => format!("{} {}", month(date), date.day()),
        _ => format!("{} {}, {}", month(date), date.day(), date.year()),
    }
}

fn month(date: Date) -> &'static str {
    let name = Date::month_name(date.month());
    &name[..3.min(name.len())]
}

#[cfg(test)]
mod tests {
    use super::date;
    use dutchy_egui::calendar::Date;

    #[test]
    fn dates_read_relative_to_today() {
        let today = Date::new(2026, 11, 8).unwrap();
        let ago = |days: i64| date(today.add_days(-days).unwrap(), today);
        assert_eq!(ago(0), "Today");
        assert_eq!(ago(1), "Yesterday");
        assert_eq!(ago(3), today.add_days(-3).unwrap().weekday().name());
        assert_eq!(ago(30), "Oct 9");
        assert_eq!(ago(400), "Oct 4, 2025");
    }
}
