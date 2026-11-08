//! Block rendering: headings, paragraphs, quotes and callouts, code, rules,
//! figures and HTML text, with the shared vertical rhythm.

use super::{
    Renderer, anchors,
    inline::{self, Typography},
    list,
    model::{Block, Callout},
    table,
};
use crate::{
    Text, TextColor,
    tokens::{markdown as tokens, space},
    widgets::{code_block::CodeBlock, media::Image},
};
use egui::{Align, Frame, Margin, Rect, Ui, WidgetInfo, accesskit::Role, pos2, vec2};

/// Render blocks separated by the document gap.
pub(super) fn show(r: &mut Renderer, ui: &mut Ui, blocks: &[Block]) {
    show_spaced(r, ui, blocks, tokens::BLOCK_GAP);
}

/// Render blocks with an explicit gap; headings receive extra space above.
pub(super) fn show_spaced(r: &mut Renderer, ui: &mut Ui, blocks: &[Block], gap: f32) {
    for (index, block) in blocks.iter().enumerate() {
        if index > 0 {
            let heading = matches!(block, Block::Heading { .. });
            ui.add_space(gap + if heading { tokens::HEADING_GAP } else { 0.0 });
        }
        show_block(r, ui, block);
    }
}

fn show_block(r: &mut Renderer, ui: &mut Ui, block: &Block) {
    match block {
        Block::Heading {
            level,
            anchor,
            inlines,
        } => heading(r, ui, *level, anchor, inlines),
        Block::Paragraph(inlines) => {
            inline::paragraph(
                r,
                ui,
                inlines,
                Typography::body(),
                Align::LEFT,
                Role::Paragraph,
            );
        }
        Block::Quote { callout, blocks } => quote(r, ui, *callout, blocks),
        Block::Code { language, code } => code_block(r, ui, language, code),
        Block::List { start, items } => list::show(r, ui, *start, items),
        Block::Table(table) => table::show(r, ui, table),
        Block::Rule => rule(r, ui),
        Block::Image { src, alt, title } => image(r, ui, src, alt, title),
        Block::Html(text) => {
            ui.add(Text::new(text.as_str()).color(TextColor::Muted));
        }
    }
}

fn heading(
    r: &mut Renderer,
    ui: &mut Ui,
    level: u8,
    anchor: &str,
    inlines: &[super::model::Inline],
) {
    let level = level.saturating_add(r.options.heading_offset).min(6);
    let response = inline::paragraph(
        r,
        ui,
        inlines,
        Typography::heading(level),
        Align::LEFT,
        Role::Heading,
    );
    response
        .ctx
        .accesskit_node_builder(response.id, |node| node.set_level(usize::from(level)));
    anchors::reveal(ui, r.root, anchor, response.rect, r.theme.reduced_motion);
}

fn quote(r: &mut Renderer, ui: &mut Ui, callout: Option<Callout>, blocks: &[Block]) {
    let p = r.theme.palette;
    let accent = match callout {
        None | Some(Callout::Important) => p.primary,
        Some(Callout::Note) => p.info,
        Some(Callout::Tip) => p.success,
        Some(Callout::Warning) => p.warning,
        Some(Callout::Caution) => p.destructive,
    };
    let margin = Margin {
        left: (tokens::QUOTE_RULE + tokens::QUOTE_PADDING) as i8,
        right: 0,
        top: space::XS as i8,
        bottom: space::XS as i8,
    };
    let result = Frame::NONE.inner_margin(margin).show(ui, |ui| {
        ui.set_width(ui.available_width());
        match callout {
            Some(callout) => {
                ui.add(Text::label(callout.label()).color(TextColor::Custom(accent)));
                ui.add_space(space::SM);
            }
            None => ui.style_mut().visuals.override_text_color = Some(p.muted_foreground),
        }
        show(r, ui, blocks);
    });
    let rect = result.response.rect;
    ui.painter().rect_filled(
        Rect::from_min_max(rect.min, pos2(rect.min.x + tokens::QUOTE_RULE, rect.max.y)),
        0,
        accent,
    );
    result
        .response
        .widget_info(|| WidgetInfo::new(Role::Blockquote));
}

fn code_block(r: &mut Renderer, ui: &mut Ui, language: &str, code: &str) {
    let mut block = CodeBlock::new(r.next_id(), code)
        .copy(r.options.code_copy)
        .line_numbers(r.options.line_numbers);
    if !language.is_empty() {
        block = block.language(language);
    }
    if let Some(highlighter) = r.options.highlighter {
        block = block.highlighter(highlighter);
    }
    ui.add(block);
}

fn rule(r: &mut Renderer, ui: &mut Ui) {
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), tokens::RULE));
    ui.painter().rect_filled(rect, 0, r.theme.palette.border);
}

fn image(r: &mut Renderer, ui: &mut Ui, src: &str, alt: &str, title: &str) {
    let alt = if alt.is_empty() { "Image" } else { alt };
    if !r.options.images {
        ui.add(Text::new(alt).color(TextColor::Muted).italic(true));
        return;
    }
    let mut image =
        Image::new(r.next_id(), egui::ImageSource::Uri(src.into()), alt).retryable(true);
    if !title.is_empty() {
        image = image.caption(title);
    }
    if image.show(ui).retry_requested {
        ui.ctx().forget_image(src);
    }
}
