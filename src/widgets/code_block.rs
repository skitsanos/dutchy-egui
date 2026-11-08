//! Code block: muted surface, mono text, optional language label, line
//! numbers, host-provided highlighting and a native copy control.

use crate::{
    Button, ButtonVariant, ControlSize, Family, Text, TextColor, TextSize, Theme, Weight, fonts,
    tokens::{border, code as tokens},
};
use egui::{
    Color32, Frame, Id, IdSalt, Label, Margin, Response, Stroke, Ui, Widget, WidgetInfo,
    accesskit::Role,
    pos2,
    text::{LayoutJob, TextFormat},
    text_selection::LabelSelectionState,
    vec2,
};
use std::{borrow::Cow, ops::Range, time::Duration};

/// Host-supplied color for a byte range of the code.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeSpan {
    /// Byte range into the code, on character boundaries.
    pub range: Range<usize>,
    /// Text color of the range.
    pub color: Color32,
}

/// Host syntax highlighter: `(language, code)` to spans in byte order.
/// Return `None` to keep plain foreground text. Overlapping, reversed or
/// out-of-bounds spans are ignored.
pub type CodeHighlighter<'a> = &'a dyn Fn(&str, &str) -> Option<Vec<CodeSpan>>;

/// Preformatted code on the muted surface, with selectable text.
#[must_use = "Add this code block to a Ui"]
pub struct CodeBlock<'a> {
    id: IdSalt,
    code: Cow<'a, str>,
    language: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    line_numbers: bool,
    copy: bool,
    wrap: bool,
    max_height: f32,
    highlighter: Option<CodeHighlighter<'a>>,
}

impl<'a> CodeBlock<'a> {
    /// Bind a stable ID and the code text.
    pub fn new(id: impl egui::AsIdSalt, code: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            code: code.into(),
            language: None,
            name: None,
            line_numbers: false,
            copy: true,
            wrap: false,
            max_height: tokens::MAX_HEIGHT,
            highlighter: None,
        }
    }
    /// Language label shown above the code and passed to the highlighter.
    pub fn language(mut self, language: impl Into<Cow<'a, str>>) -> Self {
        self.language = Some(language.into());
        self
    }
    /// Accessible name; defaults to `Code` or `<language> code`.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Number logical lines in a muted gutter.
    pub fn line_numbers(mut self, numbers: bool) -> Self {
        self.line_numbers = numbers;
        self
    }
    /// Offer a native copy control; the text goes to the platform clipboard.
    pub fn copy(mut self, copy: bool) -> Self {
        self.copy = copy;
        self
    }
    /// Wrap long lines instead of scrolling horizontally.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    /// Body height cap before vertical scrolling, clamped to 96–4096 points.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = if height.is_finite() {
            height.clamp(96.0, 4096.0)
        } else {
            tokens::MAX_HEIGHT
        };
        self
    }
    /// Host syntax highlighting.
    pub fn highlighter(mut self, highlighter: CodeHighlighter<'a>) -> Self {
        self.highlighter = Some(highlighter);
        self
    }
}

impl Widget for CodeBlock<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::from_context(ui.ctx());
        let root = ui.make_persistent_id(("dutchy-code", self.id));
        let p = theme.palette;
        Frame::NONE
            .fill(p.muted)
            .stroke(Stroke::new(border::THIN, p.border))
            .inner_margin(Margin::same(tokens::PADDING as i8))
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = None;
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.0;
                if self.language.is_some() || self.copy {
                    self.header(ui, root);
                    ui.add_space(tokens::HEADER_GAP);
                }
                self.body(ui, root, &theme)
            })
            .inner
    }
}

impl CodeBlock<'_> {
    fn header(&self, ui: &mut Ui, root: Id) {
        ui.spacing_mut().interact_size.y = ControlSize::Small.height();
        ui.horizontal(|ui| {
            if let Some(language) = &self.language {
                ui.add(Text::label(language.as_ref()).color(TextColor::Muted));
            }
            if self.copy {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.copy_control(ui, root);
                });
            }
        });
    }

    fn copy_control(&self, ui: &mut Ui, root: Id) {
        let key = root.with("copied");
        let now = ui.input(|input| input.time);
        let copied_at = ui.ctx().data(|data| data.get_temp::<f64>(key));
        let remaining = copied_at.map_or(0.0, |at| at + tokens::COPY_FEEDBACK - now);
        if remaining > 0.0 {
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(remaining));
        }
        let label = if remaining > 0.0 { "Copied" } else { "Copy" };
        // Outline keeps a visible hover on the muted surface; a ghost fill would vanish.
        let button = Button::new(label)
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small);
        if ui.add(button).clicked() {
            ui.ctx().copy_text(self.code.to_string());
            ui.ctx().data_mut(|data| data.insert_temp(key, now));
        }
    }

    fn body(&self, ui: &mut Ui, root: Id, theme: &Theme) -> Response {
        let p = theme.palette;
        let lines = self.code.lines().count().max(1);
        let gutter = self.gutter_width(ui, lines);
        egui::ScrollArea::new([!self.wrap, true])
            .id_salt(root.with("scroll"))
            .max_height(self.max_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let wrap_width = if self.wrap {
                    (ui.available_width() - gutter).max(0.0)
                } else {
                    f32::INFINITY
                };
                let job = self.job(theme, wrap_width);
                let galley = ui.fonts_mut(|fonts| fonts.layout_job(job.clone()));
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let (_, gutter_rect) = ui.allocate_space(vec2(gutter, galley.size().y));
                    let (pos, galley, response) = Label::new(job)
                        .selectable(true)
                        .wrap_mode(if self.wrap {
                            egui::TextWrapMode::Wrap
                        } else {
                            egui::TextWrapMode::Extend
                        })
                        .layout_in_ui(ui);
                    response.widget_info(|| {
                        WidgetInfo::labeled(Role::Code, ui.is_enabled(), self.name())
                    });
                    let code = self.code.to_string();
                    response
                        .ctx
                        .accesskit_node_builder(response.id, |node| node.set_value(code));
                    if ui.is_rect_visible(response.rect) {
                        if self.line_numbers {
                            self.numbers(ui, theme, &galley, pos, gutter_rect.right());
                        }
                        LabelSelectionState::label_text_selection(
                            ui,
                            &response,
                            pos,
                            galley,
                            p.foreground,
                            Stroke::NONE,
                        );
                    }
                    response
                })
                .inner
            })
            .inner
    }

    fn name(&self) -> String {
        match (&self.name, &self.language) {
            (Some(name), _) => name.to_string(),
            (None, Some(language)) => format!("{language} code"),
            (None, None) => String::from("Code"),
        }
    }

    fn gutter_width(&self, ui: &mut Ui, lines: usize) -> f32 {
        if !self.line_numbers {
            return 0.0;
        }
        let font = fonts::font_id(Family::Mono, Weight::Regular, TextSize::Sm.points());
        let digit = ui.fonts_mut(|fonts| fonts.glyph_width(&font, '0'));
        digit * lines.to_string().len() as f32 + tokens::GUTTER_GAP
    }

    /// One number per logical line, beside the first galley row of that line.
    fn numbers(&self, ui: &Ui, theme: &Theme, galley: &egui::Galley, pos: egui::Pos2, right: f32) {
        let mut line = 1;
        let mut starts_line = true;
        for placed in &galley.rows {
            if starts_line {
                let job = Text::new(line.to_string())
                    .family(Family::Mono)
                    .size(TextSize::Sm)
                    .color(TextColor::Muted)
                    .layout_job(theme, f32::INFINITY);
                let number = ui.fonts_mut(|fonts| fonts.layout_job(job));
                let origin = pos2(
                    right - tokens::GUTTER_GAP - number.size().x,
                    pos.y + placed.pos.y,
                );
                ui.painter()
                    .galley(origin, number, theme.palette.muted_foreground);
                line += 1;
            }
            starts_line = placed.ends_with_newline;
        }
    }

    fn job(&self, theme: &Theme, wrap_width: f32) -> LayoutJob {
        let size = TextSize::Sm.points();
        let base = TextFormat {
            font_id: fonts::font_id(Family::Mono, Weight::Regular, size),
            line_height: Some(size * TextSize::Sm.line_height()),
            color: theme.palette.foreground,
            ..Default::default()
        };
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;
        let code = self.code.as_ref();
        let spans = self
            .highlighter
            .and_then(|highlight| highlight(self.language.as_deref().unwrap_or(""), code))
            .unwrap_or_default();
        let mut cursor = 0;
        for span in spans {
            let valid = span.range.start >= cursor
                && span.range.end <= code.len()
                && span.range.start <= span.range.end
                && code.is_char_boundary(span.range.start)
                && code.is_char_boundary(span.range.end);
            if !valid {
                continue;
            }
            if span.range.start > cursor {
                job.append(&code[cursor..span.range.start], 0.0, base.clone());
            }
            let colored = TextFormat {
                color: span.color,
                ..base.clone()
            };
            job.append(&code[span.range.clone()], 0.0, colored);
            cursor = span.range.end;
        }
        if cursor < code.len() || code.is_empty() {
            job.append(&code[cursor..], 0.0, base);
        }
        job
    }
}
