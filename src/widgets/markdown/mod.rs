//! Markdown documents rendered from Dutchy tokens: headings, paragraphs,
//! emphasis, links, lists, task boxes, quotes and callouts, code, tables,
//! images and rules. Parsing uses pulldown-cmark; every visual decision here
//! is repository-owned and token-driven.

mod anchors;
mod blocks;
mod builder;
mod frames;
mod inline;
mod list;
mod model;
mod source;
mod table;
#[cfg(test)]
mod tests;

use crate::{Theme, widgets::code_block::CodeHighlighter};
use egui::{Context, Id, IdSalt, OpenUrl, Response, Sense, Ui, UiBuilder, WidgetInfo};
use model::Document;
use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

/// What activating a link does besides being reported to the host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LinkPolicy {
    /// Open `http(s)`/`mailto` destinations natively and scroll to `#fragment`
    /// headings of this document; relative destinations are only reported.
    #[default]
    Open,
    /// Report every activation and navigate nowhere; the host decides.
    Report,
}

/// A link activated by pointer or keyboard this pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownLink {
    /// Destination as written, after reference resolution.
    pub href: String,
    /// Optional link title.
    pub title: String,
    /// Visible link text as written.
    pub text: String,
}

impl MarkdownLink {
    /// Heading anchor for an in-document `#fragment` destination.
    pub fn fragment(&self) -> Option<&str> {
        self.href.strip_prefix('#')
    }
    /// Whether the destination has a scheme that leaves the application.
    pub fn is_external(&self) -> bool {
        self.href.contains("://") || self.href.starts_with("mailto:")
    }
}

/// Document region and the links activated during this pass.
pub struct MarkdownResponse {
    /// Named document region.
    pub response: Response,
    /// Links activated this pass, in document order.
    pub links: Vec<MarkdownLink>,
}

/// Markdown viewer over host-owned source text. The parsed tree is cached in
/// egui memory until the source changes; the host keeps the text.
#[must_use = "Call show to render the document"]
pub struct Markdown<'a> {
    id: IdSalt,
    source: &'a str,
    name: Cow<'a, str>,
    heading_offset: u8,
    links: LinkPolicy,
    images: bool,
    smart_punctuation: bool,
    code_copy: bool,
    line_numbers: bool,
    highlighter: Option<CodeHighlighter<'a>>,
}

impl<'a> Markdown<'a> {
    /// Bind a stable ID and the document source.
    pub fn new(id: impl egui::AsIdSalt, source: &'a str) -> Self {
        Self {
            id: IdSalt::new(id),
            source,
            name: Cow::Borrowed("Document"),
            heading_offset: 0,
            links: LinkPolicy::Open,
            images: true,
            smart_punctuation: false,
            code_copy: true,
            line_numbers: false,
            highlighter: None,
        }
    }
    /// Accessible name of the document region.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Render `#` as this many levels deeper, clamped so `######` stays level six.
    pub fn heading_offset(mut self, offset: u8) -> Self {
        self.heading_offset = offset.min(5);
        self
    }
    /// Native opening of external links and fragments, or host-only reporting.
    pub fn links(mut self, policy: LinkPolicy) -> Self {
        self.links = policy;
        self
    }
    /// Load images through host-installed egui loaders; otherwise show alt text.
    pub fn images(mut self, images: bool) -> Self {
        self.images = images;
        self
    }
    /// Replace straight quotes, dashes and ellipses with typographic forms.
    pub fn smart_punctuation(mut self, smart: bool) -> Self {
        self.smart_punctuation = smart;
        self
    }
    /// Offer a native copy control on fenced code blocks.
    pub fn code_copy(mut self, copy: bool) -> Self {
        self.code_copy = copy;
        self
    }
    /// Number the lines of fenced code blocks.
    pub fn line_numbers(mut self, numbers: bool) -> Self {
        self.line_numbers = numbers;
        self
    }
    /// Host syntax highlighting for fenced code blocks.
    pub fn highlighter(mut self, highlighter: CodeHighlighter<'a>) -> Self {
        self.highlighter = Some(highlighter);
        self
    }

    /// Render the document at the available width.
    pub fn show(self, ui: &mut Ui) -> MarkdownResponse {
        let root = ui.make_persistent_id(("dutchy-markdown", self.id));
        let document = cached_document(ui.ctx(), root, self.source, self.smart_punctuation);
        let mut renderer = Renderer {
            root,
            theme: Theme::from_context(ui.ctx()),
            options: &self,
            links: Vec::new(),
            counter: 0,
            depth: 0,
        };
        let result = ui.scope_builder(
            UiBuilder::new().scope_id(root).sense(Sense::hover()),
            |ui| {
                if ui.available_width().is_finite() {
                    ui.set_width(ui.available_width());
                }
                ui.spacing_mut().item_spacing.y = 0.0;
                blocks::show(&mut renderer, ui, &document.blocks);
            },
        );
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Document,
                ui.is_enabled(),
                self.name.as_ref(),
            )
        });
        MarkdownResponse {
            response: result.response,
            links: renderer.links,
        }
    }
}

/// Per-pass rendering state shared by the block and inline renderers.
pub(super) struct Renderer<'r, 'a> {
    pub root: Id,
    pub theme: Theme,
    pub options: &'r Markdown<'a>,
    pub links: Vec<MarkdownLink>,
    counter: usize,
    /// Current list nesting, for marker shapes.
    pub depth: usize,
}

impl Renderer<'_, '_> {
    /// Stable per-pass node IDs: the counter restarts every pass, so an
    /// unchanged document yields the same IDs for the same nodes.
    pub fn next_id(&mut self) -> Id {
        self.counter += 1;
        self.root.with(("node", self.counter))
    }

    /// Apply the link policy and record the activation.
    pub fn activate(&mut self, ctx: &Context, link: MarkdownLink) {
        if self.options.links == LinkPolicy::Open {
            if let Some(fragment) = link.fragment() {
                anchors::request(ctx, self.root, fragment);
            } else if link.is_external() {
                ctx.open_url(OpenUrl::new_tab(&link.href));
            }
        }
        self.links.push(link);
    }
}

struct Cached {
    hash: u64,
    document: Arc<Document>,
}

fn cached_document(ctx: &Context, root: Id, source: &str, smart: bool) -> Arc<Document> {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    smart.hash(&mut hasher);
    let hash = hasher.finish();
    let key = root.with("document");
    if let Some(cached) = ctx.data(|data| data.get_temp::<Arc<Cached>>(key))
        && cached.hash == hash
    {
        return cached.document.clone();
    }
    let document = Arc::new(Document::parse(source, smart));
    ctx.data_mut(|data| {
        data.insert_temp(
            key,
            Arc::new(Cached {
                hash,
                document: document.clone(),
            }),
        );
    });
    document
}
