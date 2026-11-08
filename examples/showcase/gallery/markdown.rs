use super::section;
use dutchy_egui::{
    Family, LinkPolicy, Markdown, MarkdownLink, Switch, Text, TextColor, TextSize, Textarea,
};
use egui::Ui;

pub struct MarkdownState {
    pub source: String,
    pub links: Vec<MarkdownLink>,
    pub report_links: bool,
    pub line_numbers: bool,
    pub smart_punctuation: bool,
    pub demote_headings: bool,
}

impl Default for MarkdownState {
    fn default() -> Self {
        Self {
            source: SAMPLE.to_string(),
            links: Vec::new(),
            report_links: true,
            line_numbers: true,
            smart_punctuation: false,
            demote_headings: false,
        }
    }
}

const SAMPLE: &str = r#"# Release notes

Dutchy renders **CommonMark** with *GitHub* extensions: tables, task lists,
~~strikethrough~~ and `inline code`. Links such as
[the web design system](https://github.com/skitsanos/dutchy-design-system "Web reference")
open natively or report to the host, and [fragments](#tables) scroll.

## Checklist

- [x] Headings, paragraphs and emphasis
- [x] Lists, tasks and nested items
  - Square markers follow the nesting level
- [ ] Footnotes and math stay literal

1. Numbered markers use the mono face
2. Wide tables scroll horizontally
3. Code blocks copy with one action

> [!TIP]
> Callouts use the status palette; plain quotes get the accent rule.

> Sharp geometry, warm colors, bold typography.

### Code

```rust
fn install(ctx: &egui::Context) {
    dutchy_egui::install(ctx, dutchy_egui::Theme::default());
}
```

### Tables

| Token | Value | Use |
| :--- | ---: | :--- |
| `space::LG` | 16 | Block gap |
| `border::ACCENT` | 4 | Quote rule |
| `TextSize::Xl4` | 36 | Level-one heading |

---

###### Section label

Images load through host loaders; without one, the alternative text appears.
"#;

pub fn show(ui: &mut Ui, state: &mut MarkdownState) {
    ui.add(Text::label("Content / Markdown").color(TextColor::Primary));
    ui.add(Text::heading("Documents, rendered in Dutchy.").size(TextSize::Xl4));
    ui.add(
        Text::new(
            "CommonMark plus tables, task lists, strikethrough and callouts, parsed once per edit and drawn from the shared tokens. The host keeps the source text and decides what links do.",
        )
        .color(TextColor::Muted),
    );
    section(ui, "01", "Options", |ui| {
        ui.add(Switch::new(
            "md-report",
            &mut state.report_links,
            "Report links to the host instead of opening them",
        ));
        ui.add(Switch::new(
            "md-numbers",
            &mut state.line_numbers,
            "Number code lines",
        ));
        ui.add(Switch::new(
            "md-smart",
            &mut state.smart_punctuation,
            "Smart punctuation",
        ));
        ui.add(Switch::new(
            "md-demote",
            &mut state.demote_headings,
            "Demote headings one level",
        ));
    });
    section(ui, "02", "Document", |ui| {
        let policy = if state.report_links {
            LinkPolicy::Report
        } else {
            LinkPolicy::Open
        };
        let result = Markdown::new("showcase-markdown", &state.source)
            .accessible_name("Release notes")
            .links(policy)
            .line_numbers(state.line_numbers)
            .smart_punctuation(state.smart_punctuation)
            .heading_offset(u8::from(state.demote_headings))
            .show(ui);
        state.links.extend(result.links);
        let excess = state.links.len().saturating_sub(5);
        state.links.drain(..excess);
    });
    section(ui, "03", "Activated links", |ui| {
        if state.links.is_empty() {
            ui.add(Text::new("No links activated yet.").color(TextColor::Muted));
        }
        for link in &state.links {
            ui.add(
                Text::new(format!("{} / {}", link.text, link.href))
                    .family(Family::Mono)
                    .size(TextSize::Sm),
            );
        }
    });
    section(ui, "04", "Source", |ui| {
        ui.add(
            Textarea::new("md-source", &mut state.source)
                .rows(12)
                .accessible_name("Markdown source"),
        );
    });
}
