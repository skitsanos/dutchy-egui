//! Event walk: inline runs merge, tight lists open implicit paragraphs, and
//! finished containers hand their content to the frame below.

use super::{
    frames::{Frame, block_frame},
    model::{Block, Document, Inline, Run, Style},
    source::{is_line_break, plain_text, slug, strip_tags},
};
use pulldown_cmark::{Event, Tag, TagEnd};
use std::collections::HashMap;

pub(super) struct Builder {
    stack: Vec<Frame>,
    style: Style,
    anchors: HashMap<String, usize>,
    /// Fallback inline target; never reachable with well-formed events.
    scratch: Vec<Inline>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            stack: vec![Frame::Blocks(Vec::new())],
            style: Style::default(),
            anchors: HashMap::new(),
            scratch: Vec::new(),
        }
    }

    pub fn finish(mut self) -> Document {
        while self.stack.len() > 1 {
            self.close_top();
        }
        match self.stack.pop() {
            Some(Frame::Blocks(blocks)) => Document { blocks },
            _ => Document::default(),
        }
    }

    pub fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(code) => self.run(
                &code,
                Style {
                    code: true,
                    ..self.style
                },
            ),
            Event::SoftBreak => self.run(" ", self.style),
            Event::HardBreak => self.run("\n", self.style),
            Event::Html(html) | Event::InlineHtml(html) => self.html(&html),
            Event::Rule => {
                self.close_implicit();
                self.push_block(Block::Rule);
            }
            Event::TaskListMarker(checked) => self.task(checked),
            Event::InlineMath(_) | Event::DisplayMath(_) | Event::FootnoteReference(_) => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Emphasis => self.style.emphasis = true,
            Tag::Strong => self.style.strong = true,
            Tag::Strikethrough => self.style.strike = true,
            Tag::Superscript | Tag::Subscript => {}
            Tag::Link {
                dest_url, title, ..
            } => self.stack.push(Frame::Link {
                href: dest_url.to_string(),
                title: title.to_string(),
                inlines: Vec::new(),
            }),
            Tag::Image {
                dest_url, title, ..
            } => self.stack.push(Frame::Image {
                src: dest_url.to_string(),
                title: title.to_string(),
                alt: String::new(),
            }),
            Tag::TableHead | Tag::TableRow => {
                if let Some(Frame::Table { row, .. }) = self.stack.last_mut() {
                    row.clear();
                }
            }
            Tag::TableCell => self.stack.push(Frame::Cell(Vec::new())),
            tag => {
                self.close_implicit();
                self.stack.push(block_frame(tag));
            }
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Emphasis => self.style.emphasis = false,
            TagEnd::Strong => self.style.strong = false,
            TagEnd::Strikethrough => self.style.strike = false,
            TagEnd::Superscript | TagEnd::Subscript => {}
            TagEnd::TableHead => {
                if let Some(Frame::Table { table, row }) = self.stack.last_mut() {
                    table.head = std::mem::take(row);
                }
            }
            TagEnd::TableRow => {
                if let Some(Frame::Table { table, row }) = self.stack.last_mut() {
                    table.rows.push(std::mem::take(row));
                }
            }
            TagEnd::Paragraph | TagEnd::TableCell | TagEnd::Link | TagEnd::Image => {
                self.close_top();
            }
            _ => {
                self.close_implicit();
                self.close_top();
            }
        }
    }

    /// Finish the top frame and hand its content to the frame below.
    fn close_top(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        let Some(frame) = self.stack.pop() else {
            return;
        };
        match frame {
            Frame::Blocks(blocks) => blocks.into_iter().for_each(|block| self.push_block(block)),
            Frame::Paragraph { inlines, .. } => self.finish_paragraph(inlines),
            Frame::Heading { level, id, inlines } => {
                let anchor = self.anchor(id, &inlines);
                self.push_block(Block::Heading {
                    level,
                    anchor,
                    inlines,
                });
            }
            Frame::Link {
                href,
                title,
                inlines,
            } => self.push_inline(Inline::Link {
                href,
                title,
                inlines,
            }),
            Frame::Image { src, title, alt } => {
                self.push_inline(Inline::Image { src, alt, title });
            }
            Frame::List { start, items } => self.push_block(Block::List { start, items }),
            Frame::Item(item) => match self.stack.last_mut() {
                Some(Frame::List { items, .. }) => items.push(item),
                _ => item
                    .blocks
                    .into_iter()
                    .for_each(|block| self.push_block(block)),
            },
            Frame::Quote { callout, blocks } => self.push_block(Block::Quote { callout, blocks }),
            Frame::Code { language, code } => {
                let code = code
                    .strip_suffix('\n')
                    .map_or(code.as_str(), |rest| {
                        rest.strip_suffix('\r').unwrap_or(rest)
                    })
                    .to_string();
                self.push_block(Block::Code { language, code });
            }
            Frame::Table { table, .. } => self.push_block(Block::Table(table)),
            Frame::Cell(inlines) => {
                if let Some(Frame::Table { row, .. }) = self.stack.last_mut() {
                    row.push(inlines);
                }
            }
            Frame::Html(html) => {
                let text = strip_tags(&html);
                if !text.is_empty() {
                    self.push_block(Block::Html(text));
                }
            }
        }
    }

    fn close_implicit(&mut self) {
        if matches!(
            self.stack.last(),
            Some(Frame::Paragraph { implicit: true, .. })
        ) {
            self.close_top();
        }
    }

    fn text(&mut self, text: &str) {
        match self.stack.last_mut() {
            Some(Frame::Code { code, .. }) => code.push_str(text),
            Some(Frame::Html(html)) => html.push_str(text),
            _ => self.run(text, self.style),
        }
    }

    fn html(&mut self, html: &str) {
        match self.stack.last_mut() {
            Some(Frame::Html(buffer)) => buffer.push_str(html),
            _ if is_line_break(html) => self.run("\n", self.style),
            _ => {}
        }
    }

    fn run(&mut self, text: &str, style: Style) {
        if let Some(Frame::Image { alt, .. }) = self.stack.last_mut() {
            alt.push_str(text);
            return;
        }
        let inlines = self.inlines();
        match inlines.last_mut() {
            Some(Inline::Run(last)) if last.style == style => last.text.push_str(text),
            _ => inlines.push(Inline::Run(Run {
                text: text.to_string(),
                style,
            })),
        }
    }

    fn push_inline(&mut self, inline: Inline) {
        self.inlines().push(inline);
    }

    /// The open inline container, opening a tight-list paragraph when needed.
    fn inlines(&mut self) -> &mut Vec<Inline> {
        if !matches!(
            self.stack.last(),
            Some(
                Frame::Paragraph { .. }
                    | Frame::Heading { .. }
                    | Frame::Link { .. }
                    | Frame::Cell(_)
            )
        ) {
            self.stack.push(Frame::Paragraph {
                inlines: Vec::new(),
                implicit: true,
            });
        }
        match self.stack.last_mut() {
            Some(
                Frame::Paragraph { inlines, .. }
                | Frame::Heading { inlines, .. }
                | Frame::Link { inlines, .. }
                | Frame::Cell(inlines),
            ) => inlines,
            _ => &mut self.scratch,
        }
    }

    fn push_block(&mut self, block: Block) {
        for frame in self.stack.iter_mut().rev() {
            match frame {
                Frame::Blocks(blocks) | Frame::Quote { blocks, .. } => return blocks.push(block),
                Frame::Item(item) => return item.blocks.push(block),
                _ => {}
            }
        }
    }

    fn task(&mut self, checked: bool) {
        if let Some(Frame::Item(item)) = self
            .stack
            .iter_mut()
            .rev()
            .find(|frame| matches!(frame, Frame::Item(_)))
        {
            item.task = Some(checked);
        }
    }

    /// A paragraph made only of images (possibly linked) becomes figures.
    fn finish_paragraph(&mut self, inlines: Vec<Inline>) {
        let mut images = Vec::new();
        if only_images(&inlines, &mut images) {
            for (src, alt, title) in images {
                self.push_block(Block::Image { src, alt, title });
            }
        } else if !inlines.is_empty() {
            self.push_block(Block::Paragraph(inlines));
        }
    }

    fn anchor(&mut self, id: Option<String>, inlines: &[Inline]) -> String {
        let base = id.unwrap_or_else(|| slug(&plain_text(inlines)));
        let count = self.anchors.entry(base.clone()).or_insert(0);
        let anchor = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        anchor
    }
}

fn only_images(inlines: &[Inline], images: &mut Vec<(String, String, String)>) -> bool {
    let mut found = false;
    for inline in inlines {
        match inline {
            Inline::Run(run) if run.text.trim().is_empty() => {}
            Inline::Image { src, alt, title } => {
                images.push((src.clone(), alt.clone(), title.clone()));
                found = true;
            }
            Inline::Link { inlines, .. } if only_images(inlines, images) => found = true,
            _ => return false,
        }
    }
    found
}
