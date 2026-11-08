//! Plain-text helpers: heading slugs, HTML text extraction and line breaks.

use super::model::Inline;

/// Text content of inlines; images contribute their alternative text.
pub(super) fn plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    append_plain(inlines, &mut out);
    out
}

fn append_plain(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Run(run) => out.push_str(&run.text),
            Inline::Link { inlines, .. } => append_plain(inlines, out),
            Inline::Image { alt, .. } => out.push_str(alt),
        }
    }
}

/// GitHub-style heading anchor: lowercase, spaces become hyphens, hyphens and
/// underscores stay, other punctuation is dropped.
pub(super) fn slug(text: &str) -> String {
    let mut out = String::new();
    for ch in text.trim().chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if ch.is_whitespace() {
            out.push('-');
        } else if ch == '-' || ch == '_' {
            out.push(ch);
        }
    }
    out
}

/// Whether raw inline HTML is a `<br>` tag in any common spelling.
pub(super) fn is_line_break(html: &str) -> bool {
    html.trim()
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
        .is_some_and(|inner| {
            inner
                .trim_end_matches('/')
                .trim()
                .eq_ignore_ascii_case("br")
        })
}

/// Text between tags with common entities decoded and whitespace collapsed.
pub(super) fn strip_tags(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    let decoded = decode_entities(&text);
    let mut out = String::with_capacity(decoded.len());
    for word in decoded.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

fn decode_entities(text: &str) -> String {
    [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&nbsp;", " "),
        ("&amp;", "&"),
    ]
    .iter()
    .fold(text.to_string(), |acc, (entity, plain)| {
        acc.replace(entity, plain)
    })
}
