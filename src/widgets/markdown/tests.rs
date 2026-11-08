//! Document model: event mapping, anchors, implicit paragraphs and helpers.

use super::{
    model::{Block, Callout, CellAlign, Document, Inline, Style},
    source::{is_line_break, plain_text, slug, strip_tags},
};

fn parse(source: &str) -> Vec<Block> {
    Document::parse(source, false).blocks
}

fn text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) | Block::Heading { inlines, .. } => plain_text(inlines),
        other => panic!("not a text block: {other:?}"),
    }
}

#[test]
fn headings_get_github_anchors_and_duplicates_are_numbered() {
    let blocks = parse("# Getting Started\n\n## Getting Started\n\n## Set-up & Use {#custom}\n");
    let anchors: Vec<_> = blocks
        .iter()
        .map(|block| match block {
            Block::Heading { level, anchor, .. } => (*level, anchor.as_str()),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        anchors,
        [
            (1, "getting-started"),
            (2, "getting-started-1"),
            (2, "custom")
        ]
    );
    assert_eq!(text(&blocks[2]), "Set-up & Use");
}

#[test]
fn inline_styles_merge_runs_and_breaks_become_spaces_or_newlines() {
    let blocks =
        parse("Plain **bold** and *soft*\nbreak, `code`, ~~gone~~, hard  \nbreak<br/>tag.");
    let Block::Paragraph(inlines) = &blocks[0] else {
        panic!("{blocks:?}");
    };
    let runs: Vec<(&str, Style)> = inlines
        .iter()
        .map(|inline| match inline {
            Inline::Run(run) => (run.text.as_str(), run.style),
            other => panic!("{other:?}"),
        })
        .collect();
    let plain = Style::default();
    let bold = Style {
        strong: true,
        ..plain
    };
    let italic = Style {
        emphasis: true,
        ..plain
    };
    let code = Style {
        code: true,
        ..plain
    };
    let strike = Style {
        strike: true,
        ..plain
    };
    assert_eq!(
        runs,
        [
            ("Plain ", plain),
            ("bold", bold),
            (" and ", plain),
            ("soft", italic),
            (" break, ", plain),
            ("code", code),
            (", ", plain),
            ("gone", strike),
            (", hard\nbreak\ntag.", plain),
        ]
    );
}

#[test]
fn links_keep_destination_title_and_nested_runs() {
    let blocks = parse("See [the **guide**](https://example.com/g \"Guide\") or [local](#intro).");
    let Block::Paragraph(inlines) = &blocks[0] else {
        panic!("{blocks:?}");
    };
    let Inline::Link {
        href,
        title,
        inlines: content,
    } = &inlines[1]
    else {
        panic!("{inlines:?}");
    };
    assert_eq!(href, "https://example.com/g");
    assert_eq!(title, "Guide");
    assert_eq!(plain_text(content), "the guide");
    assert!(matches!(&inlines[3], Inline::Link { href, .. } if href == "#intro"));
}

#[test]
fn tight_list_items_hold_implicit_paragraphs_tasks_and_nested_lists() {
    let blocks = parse("- [x] done\n- [ ] open\n  - nested\n\n3. three\n4. four\n");
    let Block::List { start: None, items } = &blocks[0] else {
        panic!("{blocks:?}");
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].task, Some(true));
    assert_eq!(text(&items[0].blocks[0]), "done");
    assert_eq!(items[1].task, Some(false));
    assert_eq!(items[1].blocks.len(), 2, "paragraph then nested list");
    assert!(matches!(&items[1].blocks[1], Block::List { start: None, items } if items.len() == 1));
    let Block::List {
        start: Some(3),
        items,
    } = &blocks[1]
    else {
        panic!("{blocks:?}");
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].task, None);
    assert_eq!(text(&items[1].blocks[0]), "four");
}

#[test]
fn image_only_paragraphs_become_figures_even_when_linked() {
    let blocks = parse(
        "![Alt text](a.png \"Title\")\n\n[![badge](b.svg)](https://x.y)\n\nText ![inline](c.png) here.",
    );
    assert_eq!(
        blocks[0],
        Block::Image {
            src: "a.png".into(),
            alt: "Alt text".into(),
            title: "Title".into(),
        }
    );
    assert!(matches!(&blocks[1], Block::Image { src, .. } if src == "b.svg"));
    let Block::Paragraph(inlines) = &blocks[2] else {
        panic!("{blocks:?}");
    };
    assert!(matches!(&inlines[1], Inline::Image { alt, .. } if alt == "inline"));
}

#[test]
fn fenced_code_keeps_its_language_and_drops_the_final_newline() {
    let blocks = parse("```rust,ignore\nfn main() {}\n\n```\n\n    indented\n");
    assert_eq!(
        blocks[0],
        Block::Code {
            language: "rust".into(),
            code: "fn main() {}\n".into(),
        }
    );
    assert_eq!(
        blocks[1],
        Block::Code {
            language: String::new(),
            code: "indented".into(),
        }
    );
}

#[test]
fn tables_record_alignments_head_and_rows() {
    let blocks = parse(
        "| Name | Qty | Note |\n| :--- | ---: | :---: |\n| Bolt | 12 | **ok** |\n| Nut | 3 |\n",
    );
    let Block::Table(table) = &blocks[0] else {
        panic!("{blocks:?}");
    };
    assert_eq!(
        table.alignments,
        [CellAlign::Left, CellAlign::Right, CellAlign::Center]
    );
    assert_eq!(table.head.len(), 3);
    assert_eq!(plain_text(&table.head[1]), "Qty");
    assert_eq!(table.rows.len(), 2);
    assert_eq!(plain_text(&table.rows[0][2]), "ok");
    assert_eq!(
        table.rows[1].len(),
        3,
        "short rows are padded by the parser"
    );
}

#[test]
fn quotes_callouts_rules_and_html_blocks_map_to_blocks() {
    let blocks = parse(
        "> quoted\n\n> [!WARNING]\n> careful\n\n---\n\n<div class=\"x\">kept &amp; text</div>\n\n<img src=\"a.png\">\n",
    );
    assert!(
        matches!(&blocks[0], Block::Quote { callout: None, blocks } if text(&blocks[0]) == "quoted")
    );
    assert!(
        matches!(&blocks[1], Block::Quote { callout: Some(Callout::Warning), blocks } if text(&blocks[0]) == "careful")
    );
    assert_eq!(blocks[2], Block::Rule);
    assert_eq!(blocks[3], Block::Html("kept & text".into()));
    assert_eq!(blocks.len(), 4, "an image tag without text renders nothing");
}

#[test]
fn smart_punctuation_is_opt_in() {
    let plain = Document::parse("\"quoted\" -- dash", false).blocks;
    assert_eq!(text(&plain[0]), "\"quoted\" -- dash");
    let smart = Document::parse("\"quoted\" -- dash", true).blocks;
    assert_eq!(text(&smart[0]), "“quoted” – dash");
}

#[test]
fn source_helpers_follow_github_slugs_and_html_rules() {
    assert_eq!(slug("  Hello, World!  "), "hello-world");
    assert_eq!(slug("Set-up_and use"), "set-up_and-use");
    assert_eq!(slug("Ünïcode Ok"), "ünïcode-ok");
    assert!(is_line_break("<br>"));
    assert!(is_line_break("<BR />"));
    assert!(!is_line_break("<b>"));
    assert_eq!(strip_tags("<p>a &lt;b&gt;\n\n  c</p>"), "a <b> c");
}
