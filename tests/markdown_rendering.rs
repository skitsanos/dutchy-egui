//! Markdown rendering baselines across themes, scales and widths.

#![cfg(all(feature = "bundled-fonts", feature = "markdown"))]

mod common;
use dutchy_egui::{Accent, Markdown, Mode, Theme};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

const DOCUMENT: &str = "# Level one\n\n## Level two\n\n### Level three\n\n#### Level four\n\n##### Level five\n\n###### Level six\n\nBody with **strong**, *emphasis*, ~~struck~~ and `inline code`, then a\n[link to the reference](https://example.com/reference \"Reference\") that\nwraps when the column is narrow, plus an inline ![icon](icon.png) image.\n\n> Plain quotes get the accent rule and muted text.\n\n> [!WARNING]\n> Callouts take the status palette.\n\n- Bullet one\n- Bullet two\n  - Nested outline\n    - Deeper square\n- [x] Done task\n- [ ] Open task\n\n7. Seven\n8. Eight\n9. Nine\n10. Ten\n\n```rust\nfn main() {\n    println!(\"sharp\");\n}\n```\n\n| Token | Value | Use |\n| :--- | ---: | :--- |\n| `space::LG` | 16 | Block gap |\n| `border::ACCENT` | 4 | Quote rule |\n\n---\n\n<div>HTML blocks keep their text.</div>\n";

fn document(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    Markdown::new("baseline", DOCUMENT)
        .line_numbers(true)
        .images(false)
        .show(ui);
}

fn harness(accent: Accent, mode: Mode, scale: f32, size: [f32; 2]) -> Harness<'static> {
    let theme = Theme::new(accent, mode).with_reduced_motion(true);
    Harness::builder()
        .with_size(size)
        .with_render_cursor(false)
        .with_theme(match mode {
            Mode::Light => egui::Theme::Light,
            Mode::Dark => egui::Theme::Dark,
        })
        .with_pixels_per_point(scale)
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui(document)
}

#[test]
fn markdown_renders_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "markdown-light-1x"),
        (Accent::Blue, Mode::Dark, 1.5, "markdown-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "markdown-purple-2x"),
    ] {
        let mut harness = harness(accent, mode, scale, [720.0, 1180.0]);
        harness.run();
        harness.snapshot(name);
        results.extend_harness(&mut harness);
    }
    results.unwrap();
}

#[test]
fn narrow_documents_wrap_text_and_scroll_wide_tables() {
    let mut harness = harness(Accent::Orange, Mode::Light, 1.0, [300.0, 1400.0]);
    harness.run();
    let link = harness.get_by_role_and_label(Role::Link, "link to the reference");
    assert!(
        link.rect().right() <= 300.0,
        "link rows stay inside the width"
    );
    for node in harness.get_all_by_role(Role::Paragraph) {
        assert!(node.rect().right() <= 300.0 + 0.5, "{:?}", node.rect());
    }
    harness.snapshot("markdown-narrow");
}

#[test]
fn hovered_and_focused_links_render_their_underlines() {
    let mut harness = harness(Accent::Orange, Mode::Light, 1.0, [720.0, 1180.0]);
    harness.run();
    harness
        .get_by_role_and_label(Role::Link, "link to the reference")
        .focus();
    harness.run();
    harness.snapshot("markdown-link-focus");
    let copy = harness.get_by_role_and_label(Role::Button, "Copy");
    copy.hover();
    harness.run();
    harness.snapshot("markdown-copy-hover");
}
