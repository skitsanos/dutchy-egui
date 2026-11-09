//! Application shell pieces: page frames, a header strip, a sidebar panel and
//! a status strip. Layout only; every value shown comes from the host.

use dutchy_egui::{
    ControlSize, Family, Kbd, Text, TextColor, TextSize, Theme, Tracking, Weight,
    tokens::{border, control, space},
};
use egui::{Frame, InnerResponse, Margin, Rect, Ui, accesskit::Role, vec2};

/// Flat page frame with the theme background and a uniform inset.
pub fn frame(ui: &Ui, inset: i8) -> Frame {
    Frame::NONE
        .fill(Theme::from_context(ui.ctx()).palette.background)
        .inner_margin(Margin::same(inset))
}

/// One-point divider across the available width.
pub fn divider(ui: &mut Ui) {
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), border::THIN));
    paint_divider(ui, rect);
}

fn paint_divider(ui: &Ui, rect: Rect) {
    ui.painter()
        .rect_filled(rect, 0, Theme::from_context(ui.ctx()).palette.border);
}

/// Top strip for the brand line and theme controls, closed by a divider.
pub fn header<R>(ui: &mut Ui, id: &str, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let frame = frame(ui, space::LG as i8);
    let result = egui::Panel::top(id).frame(frame).show(ui, contents);
    let rect = result.response.rect;
    paint_divider(
        ui,
        Rect::from_min_size(
            rect.left_bottom() - vec2(0.0, border::THIN),
            vec2(rect.width(), border::THIN),
        ),
    );
    result
}

/// Fixed-width left panel with a divider on its inner edge.
pub fn sidebar<R>(
    ui: &mut Ui,
    id: &str,
    width: f32,
    contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let frame = frame(ui, space::LG as i8);
    let result = egui::Panel::left(id)
        .exact_size(width)
        .resizable(false)
        .frame(frame)
        .show(ui, contents);
    let rect = result.response.rect;
    paint_divider(
        ui,
        Rect::from_min_size(
            rect.right_top() - vec2(border::THIN, 0.0),
            vec2(border::THIN, rect.height()),
        ),
    );
    result
}

/// A keyboard hint for the status strip: one key chip and what it does.
#[derive(Clone, Copy, Debug)]
pub struct Hint<'a> {
    /// Key or chord, such as `N` or `Ctrl+K`.
    pub key: &'a str,
    /// Short verb, such as `New task`.
    pub action: &'a str,
}

/// Bottom strip of labelled values followed by right-aligned key hints.
///
/// Nothing wraps: hints that do not fit are dropped from the end, so the strip
/// keeps one line at every window size. The strip is a named group; values are
/// plain text and update silently.
pub fn status_strip(ui: &mut Ui, id: &str, segments: &[(&str, String)], hints: &[Hint<'_>]) {
    let frame = frame(ui, space::SM as i8);
    let result = egui::Panel::bottom(id).frame(frame).show(ui, |ui| {
        let shown = fitting_hints(ui, segments, hints);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::LG;
            for (label, value) in segments {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::XS;
                    label_text(ui, label, TextColor::Muted);
                    label_text(ui, value, TextColor::Foreground);
                });
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = space::MD;
                for hint in hints[..shown].iter().rev() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::XS;
                        ui.add(Kbd::new(hint.key));
                        label_text(ui, hint.action, TextColor::Muted);
                    });
                }
            });
        });
    });
    let rect = result.response.rect;
    paint_divider(
        ui,
        Rect::from_min_size(rect.left_top(), vec2(rect.width(), border::THIN)),
    );
    result
        .response
        .ctx
        .accesskit_node_builder(result.response.id, |node| {
            node.set_role(Role::Group);
            node.set_label("Status");
        });
}

/// How many leading hints fit beside the segments on one line.
fn fitting_hints(ui: &Ui, segments: &[(&str, String)], hints: &[Hint<'_>]) -> usize {
    let segments_width: f32 = segments
        .iter()
        .map(|(label, value)| {
            label_width(ui, label) + space::XS + label_width(ui, value) + space::LG
        })
        .sum();
    let mut budget = ui.available_width() - segments_width - space::LG;
    hints
        .iter()
        .take_while(|hint| {
            let width =
                kbd_width(ui, hint.key) + space::XS + label_width(ui, hint.action) + space::MD;
            let fits = width <= budget;
            budget -= width;
            fits
        })
        .count()
}

/// Single-line label text; the accessible name keeps the host's casing.
fn label_text(ui: &mut Ui, text: &str, color: TextColor) {
    let theme = Theme::from_context(ui.ctx());
    let job = Text::label(text)
        .color(color)
        .layout_job(&theme, f32::INFINITY);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let response = ui.add(egui::Label::new(galley));
    let name = text.to_owned();
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_label(name.clone());
        node.set_value(name);
    });
}

/// Width of a `Text::label` on one line.
pub fn label_width(ui: &Ui, text: &str) -> f32 {
    let job = Text::label(text).layout_job(&Theme::from_context(ui.ctx()), f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job)).size().x
}

/// Width a `Button` of this size takes for the label, from the tokens it uses.
pub fn button_width(ui: &Ui, label: &str, size: ControlSize) -> f32 {
    let job = Text::new(label)
        .size(size.text_size())
        .weight(Weight::Bold)
        .tracking(Tracking::Wide)
        .uppercase(true)
        .layout_job(&Theme::from_context(ui.ctx()), f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job)).size().x + 2.0 * size.padding()
}

/// The width a Kbd chip takes for this key, from the same type tokens it uses.
pub fn kbd_width(ui: &Ui, key: &str) -> f32 {
    let job = Text::new(key)
        .family(Family::Mono)
        .size(TextSize::Xs)
        .weight(Weight::Bold)
        .tracking(Tracking::Wider)
        .uppercase(true)
        .layout_job(&Theme::from_context(ui.ctx()), f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job)).size().x + 2.0 * control::KBD_PADDING_X
}
