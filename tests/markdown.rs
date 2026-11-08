//! Markdown semantics, link policies, keyboard activation, fragments, tasks,
//! tables, code copy and host-owned source changes.

#![cfg(feature = "markdown")]

mod common;
use dutchy_egui::{LinkPolicy, Markdown, MarkdownLink, Theme};
use egui::{
    Key, OutputCommand,
    accesskit::{Role, Toggled},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const SOURCE: &str = "# Title\n\nIntro with [docs](https://example.com/docs \"Docs\") and [below](#details).\n\n- [x] done task\n- [ ] open task\n\n## Details\n\n| Name | Qty |\n| --- | ---: |\n| Bolt | 12 |\n\n```rust\nfn main() {}\n```\n";

struct State {
    policy: LinkPolicy,
    links: Vec<MarkdownLink>,
    source: String,
    offset: u8,
    images: bool,
    disabled: bool,
}

fn harness(policy: LinkPolicy) -> Harness<'static, State> {
    Harness::builder()
        .with_size([640.0, 1000.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                ui.add_enabled_ui(!state.disabled, |ui| {
                    let result = Markdown::new("doc", &state.source)
                        .accessible_name("Sample document")
                        .links(state.policy)
                        .heading_offset(state.offset)
                        .images(state.images)
                        .show(ui);
                    state.links.extend(result.links);
                });
            },
            State {
                policy,
                links: Vec::new(),
                source: SOURCE.to_string(),
                offset: 0,
                images: true,
                disabled: false,
            },
        )
}

fn opened_urls(harness: &Harness<'_, State>) -> Vec<String> {
    harness
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|command| match command {
            OutputCommand::OpenUrl(open) => Some(open.url.clone()),
            _ => None,
        })
        .collect()
}

fn docs_link() -> MarkdownLink {
    MarkdownLink {
        href: "https://example.com/docs".into(),
        title: "Docs".into(),
        text: "docs".into(),
    }
}

#[test]
fn documents_expose_headings_links_tasks_tables_and_code() {
    let mut h = harness(LinkPolicy::Open);
    h.run();
    h.get_by_role_and_label(Role::Document, "Sample document");
    let title = h.get_by_role_and_label(Role::Heading, "Title");
    assert_eq!(title.accesskit_node().level(), Some(1));
    let details = h.get_by_role_and_label(Role::Heading, "Details");
    assert_eq!(details.accesskit_node().level(), Some(2));
    let docs = h.get_by_role_and_label(Role::Link, "docs");
    assert_eq!(
        docs.accesskit_node().url(),
        Some("https://example.com/docs")
    );
    assert_eq!(docs.accesskit_node().description().as_deref(), Some("Docs"));
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "done task")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
    let open = h.get_by_role_and_label(Role::CheckBox, "open task");
    assert_eq!(open.accesskit_node().toggled(), Some(Toggled::False));
    assert!(open.accesskit_node().is_read_only());
    let table = h.get_by_role(Role::Table);
    assert_eq!(table.accesskit_node().data().row_count(), Some(2));
    assert_eq!(table.accesskit_node().data().column_count(), Some(2));
    assert_eq!(h.get_all_by_role(Role::ColumnHeader).count(), 2);
    assert_eq!(h.get_all_by_role(Role::Cell).count(), 2);
    h.get_by_label("Bolt");
    assert_eq!(h.get_all_by_role(Role::ListItem).count(), 2);
    let code = h.get_by_role_and_label(Role::Code, "rust code");
    assert_eq!(
        code.accesskit_node().value().as_deref(),
        Some("fn main() {}")
    );
    // The intro plus the text of both list items.
    assert_eq!(h.get_all_by_role(Role::Paragraph).count(), 3);
}

#[test]
fn external_links_open_natively_and_are_reported_once() {
    let mut h = harness(LinkPolicy::Open);
    h.run();
    h.get_by_role_and_label(Role::Link, "docs").click();
    h.step();
    assert_eq!(opened_urls(&h), ["https://example.com/docs"]);
    h.run();
    assert_eq!(h.state().links, [docs_link()]);
}

#[test]
fn report_policy_leaves_navigation_to_the_host() {
    let mut h = harness(LinkPolicy::Report);
    h.run();
    h.get_by_role_and_label(Role::Link, "docs").click();
    h.step();
    assert!(opened_urls(&h).is_empty());
    h.get_by_role_and_label(Role::Link, "below").click();
    h.run();
    let hrefs: Vec<&str> = h.state().links.iter().map(|l| l.href.as_str()).collect();
    assert_eq!(hrefs, ["https://example.com/docs", "#details"]);
    assert_eq!(h.state().links[1].fragment(), Some("details"));
    assert!(h.state().links[0].is_external());
    assert!(!h.state().links[1].is_external());
}

#[test]
fn keyboard_focus_reaches_links_and_enter_or_space_activate_them() {
    let mut h = harness(LinkPolicy::Report);
    h.run();
    h.get_by_role_and_label(Role::Link, "docs").focus();
    h.run();
    assert!(h.get_by_role_and_label(Role::Link, "docs").is_focused());
    h.key_press(Key::Enter);
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().links, [docs_link(), docs_link()]);
    h.key_press(Key::Tab);
    h.run();
    assert!(
        h.get_by_role_and_label(Role::Link, "below").is_focused(),
        "a link is one tab stop even when it has several rows"
    );
}

#[test]
fn fragment_links_scroll_their_heading_into_view() {
    let filler: String = (0..30)
        .map(|i| format!("Paragraph {i} keeps the heading far below the fold.\n\n"))
        .collect();
    let source = format!("# Top\n\n[Jump](#details)\n\n{filler}## Details\n\nFound.\n\n{filler}");
    let mut h = Harness::builder()
        .with_size([480.0, 300.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, source: &mut String| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    Markdown::new("long", source).show(ui);
                });
            },
            source,
        );
    h.run();
    let before = h.get_by_role_and_label(Role::Heading, "Details").rect();
    assert!(before.top() > 300.0, "the target starts below the viewport");
    h.get_by_role_and_label(Role::Link, "Jump").click();
    h.run();
    let after = h.get_by_role_and_label(Role::Heading, "Details").rect();
    assert!(
        after.top() >= 0.0 && after.bottom() <= 300.0,
        "the heading is revealed: {after:?}"
    );
    assert!(
        after.top() < 40.0,
        "it aligns with the viewport top: {after:?}"
    );
}

#[test]
fn code_copy_writes_the_clipboard_and_confirms() {
    let mut h = harness(LinkPolicy::Open);
    h.run();
    h.get_by_role_and_label(Role::Button, "Copy").click();
    h.step();
    let copied: Vec<&str> = h
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|command| match command {
            OutputCommand::CopyText(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(copied, ["fn main() {}"]);
    h.run();
    h.get_by_role_and_label(Role::Button, "Copied");
}

#[test]
fn source_changes_rerender_without_losing_link_focus_and_offsets_demote() {
    let mut h = harness(LinkPolicy::Report);
    h.run();
    h.get_by_role_and_label(Role::Link, "docs").focus();
    h.run();
    h.state_mut()
        .source
        .push_str("\n## Added later\n\nMore text.\n");
    h.run();
    assert_eq!(
        h.get_by_role_and_label(Role::Heading, "Added later")
            .accesskit_node()
            .level(),
        Some(2)
    );
    assert!(h.get_by_role_and_label(Role::Link, "docs").is_focused());
    h.state_mut().offset = 5;
    h.run();
    assert_eq!(
        h.get_by_role_and_label(Role::Heading, "Title")
            .accesskit_node()
            .level(),
        Some(6)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::Heading, "Details")
            .accesskit_node()
            .level(),
        Some(6),
        "levels clamp at six"
    );
}

#[test]
fn images_use_host_loaders_or_fall_back_to_alternative_text() {
    let mut h = harness(LinkPolicy::Open);
    h.state_mut().source = "![Diagram](missing.png \"Figure 1\")\n".into();
    h.run();
    h.get_by_role_and_label(Role::Image, "Diagram");
    h.get_by_label("Figure 1");
    h.state_mut().images = false;
    h.run();
    assert!(h.query_by_role_and_label(Role::Image, "Diagram").is_none());
    h.get_by_role_and_label(Role::Label, "Diagram");
}

#[test]
fn disabled_parents_disable_links_and_code_copy() {
    let mut h = harness(LinkPolicy::Report);
    h.state_mut().disabled = true;
    h.run();
    let docs = h.get_by_role_and_label(Role::Link, "docs");
    assert!(docs.accesskit_node().is_disabled());
    docs.click();
    h.run();
    assert!(h.state().links.is_empty());
    assert!(
        h.get_by_role_and_label(Role::Button, "Copy")
            .accesskit_node()
            .is_disabled()
    );
}
