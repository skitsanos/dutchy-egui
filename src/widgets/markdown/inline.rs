//! Paragraph-level text: one galley per block, Dutchy typography per run, and
//! links as focusable widgets placed over their glyph rows.

use super::{MarkdownLink, Renderer, model::Inline, model::Style, source::plain_text};
use crate::{
    Family, Leading, TextColor, TextSize, Theme, Tracking, Weight, fonts,
    interaction::{focus_is_visible, paint_focus},
    tokens::{border, markdown as tokens},
};
use egui::{
    Align, Color32, Galley, Label, Pos2, Rect, Response, Sense, Stroke, Ui, WidgetInfo,
    accesskit::Role,
    pos2,
    text::{LayoutJob, TextFormat},
    text_selection::LabelSelectionState,
};
use std::{borrow::Cow, ops::Range};

/// Base typography of a text block; inline styles modify it per run.
#[derive(Clone, Copy, Debug)]
pub(super) struct Typography {
    pub family: Family,
    pub size: TextSize,
    pub weight: Weight,
    pub tracking: Tracking,
    pub leading: Option<Leading>,
    pub uppercase: bool,
    pub color: TextColor,
}

impl Typography {
    pub fn body() -> Self {
        Self {
            family: Family::Body,
            size: TextSize::Base,
            weight: Weight::Regular,
            tracking: Tracking::Normal,
            leading: None,
            uppercase: false,
            color: TextColor::Foreground,
        }
    }
    /// Small body text for table cells.
    pub fn cell() -> Self {
        Self {
            size: TextSize::Sm,
            ..Self::body()
        }
    }
    /// Table column headings, as in the data table.
    pub fn header() -> Self {
        Self {
            size: TextSize::Xs,
            weight: Weight::Bold,
            tracking: Tracking::Wider,
            uppercase: true,
            ..Self::body()
        }
    }
    /// Display headings: levels one and two uppercase, six as a mono label.
    pub fn heading(level: u8) -> Self {
        let display = |size, tracking, uppercase| Self {
            family: Family::Display,
            size,
            weight: Weight::Bold,
            tracking,
            uppercase,
            ..Self::body()
        };
        match level {
            1 => display(TextSize::Xl4, Tracking::Tighter, true),
            2 => display(TextSize::Xl3, Tracking::Tight, true),
            3 => display(TextSize::Xl2, Tracking::Tight, false),
            4 => display(TextSize::Xl, Tracking::Normal, false),
            5 => display(TextSize::Lg, Tracking::Normal, false),
            _ => Self {
                family: Family::Mono,
                size: TextSize::Xs,
                weight: Weight::Bold,
                tracking: Tracking::Widest,
                uppercase: true,
                color: TextColor::Muted,
                leading: None,
            },
        }
    }

    fn format(&self, theme: &Theme, style: Style, override_color: Option<Color32>) -> TextFormat {
        let size = self.size.points();
        let family = if style.code {
            Family::Mono
        } else {
            self.family
        };
        let weight = if style.strong {
            Weight::Bold
        } else {
            self.weight
        };
        let color = match (self.color, override_color) {
            (TextColor::Foreground | TextColor::Muted, Some(color)) => color,
            (color, _) => color.resolve(theme),
        };
        let strikethrough = if style.strike {
            Stroke::new(border::THIN, color)
        } else {
            Stroke::NONE
        };
        TextFormat {
            font_id: fonts::font_id(family, weight, size),
            extra_letter_spacing: self.tracking.em() * size,
            line_height: Some(
                size * self
                    .leading
                    .map_or(self.size.line_height(), Leading::factor),
            ),
            color,
            background: if style.code {
                theme.palette.muted
            } else {
                Color32::TRANSPARENT
            },
            expand_bg: tokens::INLINE_CODE_PAD,
            italics: style.emphasis,
            strikethrough,
            ..Default::default()
        }
    }
}

struct LinkSpan {
    range: Range<usize>,
    href: String,
    title: String,
    text: String,
}

struct JobBuilder<'t> {
    theme: &'t Theme,
    typography: Typography,
    override_color: Option<Color32>,
    job: LayoutJob,
    links: Vec<LinkSpan>,
}

impl JobBuilder<'_> {
    fn append(&mut self, inlines: &[Inline], in_link: bool) {
        for inline in inlines {
            match inline {
                Inline::Run(run) => self.run(&run.text, run.style, None, in_link),
                Inline::Link {
                    href,
                    title,
                    inlines,
                } => {
                    let start = self.job.text.len();
                    self.append(inlines, true);
                    self.links.push(LinkSpan {
                        range: start..self.job.text.len(),
                        href: href.clone(),
                        title: title.clone(),
                        text: plain_text(inlines),
                    });
                }
                Inline::Image { alt, .. } => {
                    let alt = if alt.is_empty() { "image" } else { alt };
                    let style = Style {
                        emphasis: true,
                        ..Style::default()
                    };
                    self.run(
                        alt,
                        style,
                        Some(self.theme.palette.muted_foreground),
                        in_link,
                    );
                }
            }
        }
    }

    fn run(&mut self, text: &str, style: Style, color: Option<Color32>, in_link: bool) {
        let mut format = self
            .typography
            .format(self.theme, style, self.override_color);
        if let Some(color) = color {
            format.color = color;
        }
        if in_link {
            let primary = self.theme.palette.primary;
            format.color = primary;
            format.underline = Stroke::new(tokens::LINK_UNDERLINE, primary);
        }
        let text = if self.typography.uppercase && !style.code {
            Cow::Owned(text.to_uppercase())
        } else {
            Cow::Borrowed(text)
        };
        self.job.append(&text, 0.0, format);
    }
}

fn build(
    theme: &Theme,
    typography: Typography,
    inlines: &[Inline],
    override_color: Option<Color32>,
    wrap_width: f32,
    halign: Align,
) -> (LayoutJob, Vec<LinkSpan>) {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    job.halign = halign;
    let mut builder = JobBuilder {
        theme,
        typography,
        override_color,
        job,
        links: Vec::new(),
    };
    builder.append(inlines, false);
    (builder.job, builder.links)
}

/// Unwrapped and narrowest (widest word) widths, for table column sizing.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Measure {
    pub natural: f32,
    pub minimum: f32,
}

pub(super) fn measure(
    ui: &mut Ui,
    theme: &Theme,
    inlines: &[Inline],
    typography: Typography,
) -> Measure {
    let override_color = ui.visuals().override_text_color;
    let (job, _) = build(
        theme,
        typography,
        inlines,
        override_color,
        f32::INFINITY,
        Align::LEFT,
    );
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    Measure {
        natural: galley.size().x,
        minimum: widest_word(&galley),
    }
}

/// Width of the longest whitespace-free glyph run; columns never wrap inside it.
fn widest_word(galley: &Galley) -> f32 {
    let mut widest = 0.0_f32;
    for placed in &galley.rows {
        let mut start: Option<f32> = None;
        let mut end = 0.0;
        for glyph in &placed.row.glyphs {
            if glyph.chr.is_whitespace() {
                if let Some(start) = start.take() {
                    widest = widest.max(end - start);
                }
            } else {
                start.get_or_insert(glyph.pos.x);
                end = glyph.pos.x + glyph.advance_width;
            }
        }
        if let Some(start) = start {
            widest = widest.max(end - start);
        }
    }
    widest
}

/// Lay out, paint and wire one text block. Links become separate widgets.
pub(super) fn paragraph(
    r: &mut Renderer,
    ui: &mut Ui,
    inlines: &[Inline],
    typography: Typography,
    halign: Align,
    role: Role,
) -> Response {
    let override_color = ui.visuals().override_text_color;
    let (job, links) = build(
        &r.theme,
        typography,
        inlines,
        override_color,
        ui.available_width(),
        halign,
    );
    let (pos, galley, response) = Label::new(job).halign(halign).layout_in_ui(ui);
    let text = plain_text(inlines);
    response.widget_info(|| WidgetInfo::labeled(role, ui.is_enabled(), text.as_str()));
    if ui.is_rect_visible(response.rect) {
        let color = override_color.unwrap_or_else(|| typography.color.resolve(&r.theme));
        LabelSelectionState::label_text_selection(
            ui,
            &response,
            pos,
            galley.clone(),
            color,
            Stroke::NONE,
        );
    }
    for link in links {
        show_link(r, ui, &galley, pos, link);
    }
    response
}

fn show_link(r: &mut Renderer, ui: &mut Ui, galley: &Galley, origin: Pos2, link: LinkSpan) {
    let rects = link_rects(galley, origin, &link.range);
    let Some((&first, rest)) = rects.split_first() else {
        return;
    };
    let id = r.next_id();
    let mut response = ui.interact(first, id, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), link.text.as_str()));
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_url(link.href.clone());
        if !link.title.is_empty() {
            node.set_description(link.title.clone());
        }
    });
    let mut clicked = response.clicked();
    let mut hovered = response.hovered();
    for (index, &rect) in rest.iter().enumerate() {
        let extra = ui.interact(rect, id.with(index), Sense::CLICK);
        clicked |= extra.clicked();
        hovered |= extra.hovered();
    }
    let primary = r.theme.palette.primary;
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if hovered || focus_is_visible(&response) {
        let stroke = Stroke::new(tokens::LINK_HOVER_UNDERLINE, primary);
        for rect in &rects {
            let y = rect.bottom() - tokens::LINK_HOVER_UNDERLINE / 2.0;
            ui.painter().hline(rect.x_range(), y, stroke);
        }
    }
    paint_focus(ui, &response, primary);
    if !link.title.is_empty() {
        response = response.on_hover_text(link.title.clone());
    }
    if clicked {
        r.activate(
            &response.ctx,
            MarkdownLink {
                href: link.href,
                title: link.title,
                text: link.text,
            },
        );
    }
}

/// One rectangle per galley row that contains glyphs of the byte range.
fn link_rects(galley: &Galley, origin: Pos2, range: &Range<usize>) -> Vec<Rect> {
    let offsets: Vec<usize> = galley.job.text.char_indices().map(|(i, _)| i).collect();
    let mut rects = Vec::new();
    let mut index = 0;
    for placed in &galley.rows {
        let mut span: Option<(f32, f32)> = None;
        for glyph in &placed.row.glyphs {
            let byte = offsets.get(index).copied().unwrap_or(usize::MAX);
            index += 1;
            if range.contains(&byte) {
                let end = glyph.pos.x + glyph.advance_width;
                span = Some(span.map_or((glyph.pos.x, end), |(start, _)| (start, end)));
            }
        }
        if placed.ends_with_newline {
            index += 1;
        }
        if let Some((start, end)) = span {
            let top = origin + placed.pos.to_vec2();
            rects.push(Rect::from_min_max(
                pos2(top.x + start, top.y),
                pos2(top.x + end, top.y + placed.row.size.y),
            ));
        }
    }
    rects
}
