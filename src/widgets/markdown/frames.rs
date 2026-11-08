//! Open containers while events stream in, and the tags that open them.

use super::model::{Block, Callout, CellAlign, Inline, Item, Table};
use pulldown_cmark::{Alignment, BlockQuoteKind, CodeBlockKind, Tag};

pub(super) enum Frame {
    Blocks(Vec<Block>),
    Paragraph {
        inlines: Vec<Inline>,
        implicit: bool,
    },
    Heading {
        level: u8,
        id: Option<String>,
        inlines: Vec<Inline>,
    },
    Link {
        href: String,
        title: String,
        inlines: Vec<Inline>,
    },
    Image {
        src: String,
        title: String,
        alt: String,
    },
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Item(Item),
    Quote {
        callout: Option<Callout>,
        blocks: Vec<Block>,
    },
    Code {
        language: String,
        code: String,
    },
    Table {
        table: Table,
        row: Vec<Vec<Inline>>,
    },
    Cell(Vec<Inline>),
    Html(String),
}

pub(super) fn block_frame(tag: Tag<'_>) -> Frame {
    match tag {
        Tag::Paragraph => Frame::Paragraph {
            inlines: Vec::new(),
            implicit: false,
        },
        Tag::Heading { level, id, .. } => Frame::Heading {
            level: level as u8,
            id: id.map(|id| id.to_string()),
            inlines: Vec::new(),
        },
        Tag::BlockQuote(kind) => Frame::Quote {
            callout: kind.map(callout),
            blocks: Vec::new(),
        },
        Tag::CodeBlock(kind) => Frame::Code {
            language: language(&kind),
            code: String::new(),
        },
        Tag::HtmlBlock => Frame::Html(String::new()),
        Tag::List(start) => Frame::List {
            start,
            items: Vec::new(),
        },
        Tag::Item => Frame::Item(Item {
            task: None,
            blocks: Vec::new(),
        }),
        Tag::Table(alignments) => Frame::Table {
            table: Table {
                alignments: alignments.iter().map(|a| align(*a)).collect(),
                ..Table::default()
            },
            row: Vec::new(),
        },
        _ => Frame::Blocks(Vec::new()),
    }
}

fn language(kind: &CodeBlockKind<'_>) -> String {
    match kind {
        CodeBlockKind::Indented => String::new(),
        CodeBlockKind::Fenced(info) => info
            .split(|c: char| c.is_whitespace() || c == ',' || c == '{')
            .next()
            .unwrap_or_default()
            .to_string(),
    }
}

fn callout(kind: BlockQuoteKind) -> Callout {
    match kind {
        BlockQuoteKind::Note => Callout::Note,
        BlockQuoteKind::Tip => Callout::Tip,
        BlockQuoteKind::Important => Callout::Important,
        BlockQuoteKind::Warning => Callout::Warning,
        BlockQuoteKind::Caution => Callout::Caution,
    }
}

fn align(alignment: Alignment) -> CellAlign {
    match alignment {
        Alignment::None | Alignment::Left => CellAlign::Left,
        Alignment::Center => CellAlign::Center,
        Alignment::Right => CellAlign::Right,
    }
}
