//! Block/inline tree built from pulldown-cmark events. Parsing happens once
//! per source change; the renderer walks the finished tree every pass.

use super::builder::Builder;
use pulldown_cmark::{Options, Parser};

/// Inline emphasis flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Style {
    pub strong: bool,
    pub emphasis: bool,
    pub strike: bool,
    pub code: bool,
}

/// Styled text run; a `\n` is a hard line break.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Run {
    pub text: String,
    pub style: Style,
}

/// Inline content of a paragraph, heading or table cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Inline {
    Run(Run),
    Link {
        href: String,
        title: String,
        inlines: Vec<Inline>,
    },
    Image {
        src: String,
        alt: String,
        title: String,
    },
}

/// GitHub-style blockquote callouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Callout {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl Callout {
    pub fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }
}

/// Column alignment from the delimiter row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Table {
    pub alignments: Vec<CellAlign>,
    pub head: Vec<Vec<Inline>>,
    pub rows: Vec<Vec<Vec<Inline>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Item {
    pub task: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Block {
    Heading {
        level: u8,
        anchor: String,
        inlines: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Quote {
        callout: Option<Callout>,
        blocks: Vec<Block>,
    },
    Code {
        language: String,
        code: String,
    },
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Table(Table),
    Rule,
    Image {
        src: String,
        alt: String,
        title: String,
    },
    /// Text content of a raw HTML block; tags are not rendered.
    Html(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Document {
    pub blocks: Vec<Block>,
}

impl Document {
    /// Parse CommonMark with tables, strikethrough, task lists, heading
    /// attributes and GitHub callouts. Footnotes and math stay literal.
    pub fn parse(source: &str, smart_punctuation: bool) -> Self {
        let mut options = Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_GFM;
        if smart_punctuation {
            options |= Options::ENABLE_SMART_PUNCTUATION;
        }
        let mut builder = Builder::new();
        for event in Parser::new_ext(source, options) {
            builder.event(event);
        }
        builder.finish()
    }
}
