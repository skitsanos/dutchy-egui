# Markdown

`Markdown` renders host-owned CommonMark text from the shared tokens. The host
keeps the source string; the widget parses it with pulldown-cmark once per
change (the tree is cached in egui memory under the widget ID) and walks the
cached tree every pass. Every visual decision is repository-owned: headings,
paragraphs, links, lists, quotes, code and tables reuse `Text`, `Image`,
`CodeBlock`, the focus policy and the geometry tokens in `tokens::markdown`.

```rust
use dutchy_egui::{LinkPolicy, Markdown};

fn notes_ui(ui: &mut egui::Ui, source: &str, route: &mut Option<String>) {
    let result = Markdown::new("release-notes", source)
        .accessible_name("Release notes")
        .links(LinkPolicy::Report)
        .line_numbers(true)
        .show(ui);
    for link in result.links {
        if link.fragment().is_none() && !link.is_external() {
            *route = Some(link.href); // relative destinations belong to the host
        }
    }
}
```

The `markdown` feature is enabled by default and adds pulldown-cmark 0.13.6
without its default features (parser only: `bitflags`, `memchr`, `unicase`).
Disable default features and re-enable `bundled-fonts` for a parser-free build;
`CodeBlock` stays available either way.

## Supported syntax

CommonMark plus GitHub tables, strikethrough, task lists, heading attributes
(`{#id}`) and callouts (`> [!NOTE]`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`).
Smart punctuation is opt-in through `.smart_punctuation(true)`. Footnotes,
math, definition lists, wiki links and raw HTML are not rendered: footnote and
math syntax stays literal text, inline HTML other than `<br>` is dropped, and an
HTML block contributes only its text content as a muted paragraph.

## Typography and geometry

| Markdown | Treatment |
| --- | --- |
| `#` / `##` | Display bold, uppercase, 36 / 30 points, tighter / tight tracking |
| `###` – `#####` | Display bold mixed case, 24 / 20 / 18 points |
| `######` | Mono uppercase 12-point label in muted text |
| Paragraph | Body 16 points, 1.5 line height; `**strong**` bold, `*emphasis*` italic, `~~strike~~` one-point strikethrough |
| `` `code` `` | Mono at the surrounding size on the muted surface, two-point background expansion |
| Link | Accent color with a one-point underline; two points on hover or keyboard focus, plus the shared focus ring |
| `>` quote | Four-point accent rule, 16-point padding, muted text |
| `> [!KIND]` | Rule and mono label in the status color (Note info, Tip success, Important accent, Warning warning, Caution destructive); regular text |
| `-` list | Six-point filled square, then an outlined square, then a four-point square per nesting level |
| `1.` list | Mono medium 14-point numbers in muted text, right-aligned to the widest number |
| `- [x]` | Read-only 16-point task box: accent fill with a check, or background with the border |
| Fenced code | `CodeBlock`: muted surface, one-point border, mono 14-point text, language label, outline Copy control, optional line numbers, horizontal scrolling, 480-point height cap |
| Table | Content-sized columns, muted uppercase header, 16 × 8 cell padding, one-point row separators, two-point outer border |
| `---` | Two-point border-colored rule |
| Image | `Image` with alt text and the title as caption; `.images(false)` shows muted italic alt text |

Blocks are separated by 16 points; headings add 8 more above. List items are
4 points apart and the blocks of a loose item 8. `.heading_offset(n)` renders
`#` as level `1 + n`, clamped to six, for documents embedded under a host title.

Tables take their natural width. When that exceeds the available width, every
column gives up the slack above its widest word in proportion, so words never
break mid-column; if the widest words alone overflow, the table scrolls
horizontally. Alignment follows the delimiter row.

## Links

Every activation is returned in `MarkdownResponse::links` as a `MarkdownLink`
with `href`, `title` and visible `text`, in document order. `LinkPolicy::Open`
(default) also opens `http(s)` and `mailto` destinations through
`Context::open_url` in a new tab and scrolls `#fragment` destinations to the
heading that owns the anchor; relative destinations are only reported.
Other URI schemes are reported without native opening; scheme recognition is
case-insensitive. `LinkPolicy::Report` never navigates. Anchors follow GitHub
rules: lowercase, spaces become hyphens, hyphens and underscores stay, other
punctuation drops, and repeated anchors receive `-1`, `-2` suffixes. Heading
attributes override them.

Each link is one focusable widget with the Link role, its destination as the
URL and the title as description and tooltip. Tab reaches it once even when it
wraps over several rows; Enter or Space activate it; the pointer gets a hand
cursor over every row. Continuation rows share the first row's identity.

## Accessibility

The region is a Document named by `.accessible_name` ("Document" by default).
Headings carry the Heading role with their rendered level and the text as
written (uppercase is visual). Paragraphs, lists, items, block quotes, tables
with row and column counts, column headers, cells, read-only check boxes and
code (with the code text as value) expose their roles. Links and the Copy
control are the only inputs; both are named.

## Limits

Inline images render as their alt text; only image-only paragraphs (including
linked badges, whose link is dropped) become figures. Images need host-installed
loaders; `url-images` covers HTTP(S) PNG/JPEG. Tables have no column spans or
row headers. Code blocks keep host highlighting optional: pass a
`CodeHighlighter` that returns colored byte spans built with `CodeSpan::new`. Text selection spans one
block at a time. Percent-encoded fragments must match the anchor text exactly.
Nested tables inside lists wrap at the item width. See
[DS-071](issues/DS-071.md) and the
[current verification record](verification-fixes-markdown.md).
