//! Native opening follows the documented URI allowlist; hosts receive every link.
#![cfg(feature = "markdown")]
mod common;
use dutchy_egui::{LinkPolicy, Markdown, MarkdownLink, Theme};
use egui::{OutputCommand, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn native_policy_opens_only_web_and_mail_schemes_and_reports_every_activation() {
    for (href, external, opens) in [
        ("https://example.test/docs", true, true),
        ("HTTP://example.test/docs", true, true),
        ("file:///tmp/document.txt", true, false),
        ("app://workspace", true, false),
        ("javascript:alert", true, false),
        ("MAILTO:dev@example.test", true, true),
        ("guide.md", false, false),
        ("folder/readme:md", false, false),
    ] {
        let source = format!("[Open]({href})");
        let mut h = Harness::builder()
            .with_size([420.0, 180.0])
            .with_context_setup(|ctx| {
                common::install(ctx, Theme::default().with_reduced_motion(true))
            })
            .build_ui_state(
                move |ui, links: &mut Vec<MarkdownLink>| {
                    links.extend(
                        Markdown::new("links", &source)
                            .links(LinkPolicy::Open)
                            .show(ui)
                            .links,
                    );
                },
                Vec::new(),
            );
        h.run();
        h.get_by_role_and_label(Role::Link, "Open").click();
        h.step();
        let opened: Vec<_> = h
            .output()
            .platform_output
            .commands
            .iter()
            .filter_map(|c| match c {
                OutputCommand::OpenUrl(url) => Some(url.url.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(opened, if opens { vec![href] } else { vec![] }, "{href}");
        assert_eq!(h.state().len(), 1);
        assert_eq!(h.state()[0].href, href);
        assert_eq!(h.state()[0].is_external(), external, "{href}");
    }
}
