use super::support::*;
use dutchy_egui::{StatusAlign, StatusBar, StatusItem, Theme};
use egui::{Rect, accesskit::Role, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const INSET: f32 = 8.0;
const GAP: f32 = 4.0;
const GROUP_GAP: f32 = 16.0;

fn run(page: Page) -> Harness<'static, Page> {
    let mut h = harness(page);
    h.run_steps(3);
    h
}
/// Width every segment takes at a roomy width, by key.
fn widths() -> Vec<(u32, f32)> {
    let h = run(Page::default());
    h.state()
        .segments
        .iter()
        .map(|(k, r)| (*k, r.width()))
        .collect()
}
/// Bar width needed for the given start and end keys.
fn needed(widths: &[(u32, f32)], start: &[u32], end: &[u32]) -> f32 {
    let sum = |keys: &[u32]| -> f32 {
        keys.iter()
            .map(|k| widths.iter().find(|(key, _)| key == k).unwrap().1)
            .sum()
    };
    let gaps = |keys: &[u32]| GAP * keys.len().saturating_sub(1) as f32;
    let groups = if start.is_empty() || end.is_empty() {
        0.0
    } else {
        GROUP_GAP
    };
    2.0 * INSET + sum(start) + sum(end) + gaps(start) + gaps(end) + groups
}

#[test]
fn segments_sit_at_both_ends_in_reading_order_on_a_fixed_height_strip() {
    let h = run(Page::default());
    let p = h.state();
    assert_eq!((p.bar.width(), p.bar.height()), (960.0, 32.0));
    assert!(p.below.top() >= p.bar.bottom());
    let keys: Vec<u32> = p.segments.iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, [1, 2, 3, 4, 5]);
    assert!(p.hidden.is_empty());
    for (_, r) in &p.segments {
        assert_eq!((r.top(), r.bottom()), (p.bar.top(), p.bar.bottom()));
    }
    assert_eq!(segment(p, 1).left(), p.bar.left() + INSET);
    assert_eq!(segment(p, 2).left(), segment(p, 1).right() + GAP);
    assert_eq!(segment(p, 3).left(), segment(p, 2).right() + GAP);
    assert_eq!(segment(p, 5).right(), p.bar.right() - INSET);
    assert_eq!(segment(p, 4).right() + GAP, segment(p, 5).left());
}
#[test]
fn lowest_priority_segments_hide_first_latest_on_ties_and_are_reported() {
    let w = widths();
    let all = needed(&w, &[1, 2, 3], &[4, 5]);
    for (width, hidden) in [
        (all, vec![]),
        (all - 1.0, vec![5]),
        (needed(&w, &[1, 2, 3], &[]), vec![4, 5]),
        (needed(&w, &[1, 2, 3], &[]) - 1.0, vec![2, 4, 5]),
        (needed(&w, &[1], &[]), vec![2, 3, 4, 5]),
    ] {
        let h = run(Page {
            width,
            ..Default::default()
        });
        let p = h.state();
        assert_eq!(p.hidden, hidden, "width {width}");
        for key in &hidden {
            assert!(!p.segments.iter().any(|(k, _)| k == key));
        }
    }
    let h = run(Page {
        width: needed(&w, &[1, 2, 3], &[]) - 1.0,
        ..Default::default()
    });
    let p = h.state();
    assert_eq!(segment(p, 3).left(), segment(p, 1).right() + GAP);
    assert!(
        h.query_by_role_and_label(Role::Label, "Selected: 3")
            .is_none()
    );
    assert!(
        h.query_by_role_and_label(Role::Label, "Unread: 42")
            .is_some()
    );
    assert!(h.query_by_role_and_label(Role::Label, "Commands").is_none());
    assert!(h.query_by_role_and_label(Role::Button, "Sync").is_some());
}
struct Probe {
    bar: Rect,
    segments: Vec<Rect>,
    hidden: Vec<u32>,
}
/// A bar inside a child region exactly `width` wide, with nothing beside it.
fn bar_in(width: f32, items: fn() -> Vec<StatusItem<'static, u32>>) -> Harness<'static, Probe> {
    let mut h = Harness::builder()
        .with_size([400.0, 200.0])
        .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
        .build_ui_state(
            move |ui, p: &mut Probe| {
                let region = Rect::from_min_size(ui.cursor().min, vec2(width, 100.0));
                let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(region));
                let items = items();
                let r = StatusBar::new("probe", &items).show(&mut ui);
                p.bar = r.response.rect;
                p.segments = r.segments.iter().map(|(_, s)| s.rect).collect();
                p.hidden = r.hidden;
            },
            Probe {
                bar: Rect::NOTHING,
                segments: Vec::new(),
                hidden: Vec::new(),
            },
        );
    h.run_steps(3);
    h
}
#[test]
fn a_lone_segment_that_cannot_fit_is_elided_but_keeps_its_full_name() {
    let h = bar_in(120.0, || {
        vec![StatusItem::text(9, "Synchronising every folder")]
    });
    let p = h.state();
    assert!(p.hidden.is_empty());
    assert!(p.segments[0].width() <= 120.0 - 2.0 * INSET);
    let output = h.output();
    let elided = super::common::galleys(&output.shapes).iter().any(|g| {
        g.rows
            .iter()
            .flat_map(|row| row.glyphs.iter())
            .any(|glyph| glyph.chr == '…')
    });
    assert!(elided, "the segment text is elided");
    let node = h.get_by_role_and_label(Role::Label, "Synchronising every folder");
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Synchronising every folder"),
        "an elided segment's tooltip carries the full text"
    );
}

#[test]
fn oversized_labels_and_adornments_fit_inside_the_bar() {
    let h = bar_in(120.0, || {
        vec![
            StatusItem::labeled(
                1,
                "Detailed synchronization status for this account",
                "Ready",
            )
            .align(StatusAlign::End)
            .clickable(true),
        ]
    });
    let p = h.state();
    assert!(p.hidden.is_empty());
    assert_eq!(p.segments.len(), 1);
    assert!(
        p.bar.contains_rect(p.segments[0]),
        "bar={:?}, segment={:?}",
        p.bar,
        p.segments[0]
    );
    let node = h.get_by_role_and_label(
        Role::Button,
        "Detailed synchronization status for this account",
    );
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Detailed synchronization status for this account: Ready")
    );

    let h = bar_in(100.0, || {
        vec![
            StatusItem::hint(1, "Control+Shift+Alt+F12", "Commands")
                .progress(50.0)
                .align(StatusAlign::End),
        ]
    });
    assert!(h.state().hidden.is_empty());
    assert!(h.state().bar.contains_rect(h.state().segments[0]));
    let node = h.get_by_role_and_label(Role::ProgressIndicator, "Commands");
    assert_eq!(node.accesskit_node().numeric_value(), Some(50.0));
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Control+Shift+Alt+F12 Commands")
    );
}

#[test]
fn newlines_in_status_text_and_hints_are_laid_out_on_one_line() {
    let h = bar_in(380.0, || {
        vec![StatusItem::labeled(
            1,
            "Sync\nState",
            "Failed\r\nRetry later",
        )]
    });
    let output = h.output();
    let galleys = super::common::galleys(&output.shapes);
    assert_eq!(galleys.len(), 2);
    assert!(galleys.iter().all(|g| g.rows.len() == 1));
    assert!(galleys.iter().any(|g| g.job.text == "FAILED RETRY LATER"));

    let h = bar_in(380.0, || {
        vec![StatusItem::hint(1, "Ctrl+\nK", "Open\nCommands")]
    });
    let output = h.output();
    let galleys = super::common::galleys(&output.shapes);
    assert_eq!(galleys.len(), 2);
    assert!(galleys.iter().all(|g| g.rows.len() == 1));
}
#[test]
fn unbounded_and_zero_widths_keep_geometry_finite() {
    let h = bar_in(f32::INFINITY, mail_items);
    let p = h.state();
    assert_eq!(p.bar.width(), 640.0);
    // How many segments fit depends on the host font; those shown fit inside.
    assert!(!p.segments.is_empty());
    assert!(p.segments.iter().all(|r| p.bar.contains_rect(*r)));
    for width in [0.0, 8.0, 28.0] {
        let h = bar_in(width, mail_items);
        let p = h.state();
        assert!(p.bar.is_finite() && p.bar.width() == width);
        assert_eq!(p.hidden, [1, 2, 3, 4, 5]);
        assert!(p.segments.is_empty());
        assert!(h.query_by_role_and_label(Role::Button, "Sync").is_none());
    }
}
