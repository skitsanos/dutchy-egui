use super::CommandState;
use dutchy_egui::{
    Text, TextColor, TextSize, Theme,
    tokens::{border, space},
};
use egui::{Stroke, Ui, pos2, vec2};
pub(super) fn show(ui: &mut Ui, state: &CommandState) {
    let p = Theme::from_context(ui.ctx()).palette;
    egui::Frame::NONE
        .fill(p.muted)
        .stroke(Stroke::new(border::STANDARD, p.border))
        .inner_margin(egui::Margin::same(space::XL as i8))
        .show(ui, |ui| {
            ui.set_min_size(vec2(ui.available_width(), space::XXL * 5.0));
            let rect = egui::Rect::from_min_size(
                ui.max_rect().min,
                vec2(ui.available_width(), space::XXL * 5.0),
            );
            if state.grid {
                let painter = ui.painter().with_clip_rect(rect);
                for x in (0..(rect.width() / (space::XL * 2.0)) as usize)
                    .map(|x| rect.left() + x as f32 * space::XL * 2.0)
                {
                    painter.line_segment(
                        [pos2(x, rect.top()), pos2(x, rect.top() + space::XXL * 5.0)],
                        Stroke::new(border::THIN, p.border),
                    );
                }
            }
            if state.rulers {
                ui.painter().line_segment(
                    [rect.min, rect.min + vec2(rect.width(), 0.0)],
                    Stroke::new(border::ACCENT, p.primary),
                );
            }
            ui.add_space(space::LG);
            ui.add(Text::label("Canal Studio").color(TextColor::Primary));
            ui.add(Text::heading("Make something sharp.").size(TextSize::Xl3));
            ui.add(Text::new("Your commands. One connected workspace."));
            ui.add_space(space::LG);
        });
}
