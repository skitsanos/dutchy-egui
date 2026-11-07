//! Page requests, arithmetic boundaries, keyed focus and responsive controls.
mod common;
use dutchy_egui::{Input, Pagination, PaginationVariant, Theme};
use egui::{
    Key, Rect,
    accesskit::{AriaCurrent, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
struct State {
    page: usize,
    total: usize,
    apply: bool,
    disabled: bool,
    inserted: bool,
    compact: bool,
    requests: Vec<usize>,
    effective: Option<usize>,
    rects: Vec<Rect>,
    outside: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            page: 7,
            total: 20,
            apply: false,
            disabled: false,
            inserted: false,
            compact: false,
            requests: vec![],
            effective: None,
            rects: vec![],
            outside: String::new(),
        }
    }
}
fn harness(width: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size([width, 700.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.inserted {
                    ui.label("Inserted context");
                }
                let result = Pagination::new("records", state.page, state.total)
                    .accessible_name("Record pages")
                    .variant(if state.compact {
                        PaginationVariant::Compact
                    } else {
                        PaginationVariant::Standard
                    })
                    .enabled(!state.disabled)
                    .show(ui);
                state.effective = result.current_page;
                state.rects = result.controls.iter().map(|r| r.rect).collect();
                if let Some(page) = result.requested_page {
                    state.requests.push(page);
                    if state.apply {
                        state.page = page;
                    }
                }
                ui.add(Input::new("outside", &mut state.outside).accessible_name("Outside"));
            },
            State::default(),
        )
}
#[test]
fn numbered_requests_are_host_owned_and_current_page_is_explicit() {
    let mut h = harness(760.0);
    assert!(
        h.query_by_role_and_label(Role::Navigation, "Record pages")
            .is_some()
    );
    assert_eq!(
        h.get_by_label("Page 7")
            .accesskit_node()
            .data()
            .aria_current(),
        Some(AriaCurrent::Page)
    );
    h.get_by_label("Page 7").click();
    h.run();
    assert!(h.state().requests.is_empty());
    h.get_by_label("Page 8").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().requests, [8]);
    assert_eq!(h.state().page, 7);
    h.state_mut().apply = true;
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().page, 8);
    assert!(h.get_by_label("Page 8").is_focused());
    h.get_by_label("Next page").click();
    h.run_steps(2);
    assert_eq!(h.state().page, 9);
    h.get_by_label("Previous page").click();
    h.run_steps(2);
    assert_eq!(h.state().page, 8);
    h.check_accessibility();
}
#[test]
fn endpoints_empty_invalid_and_maximum_counts_are_safe() {
    let mut h = harness(760.0);
    for (page, total, effective) in [
        (0, 5, Some(1)),
        (99, 5, Some(5)),
        (1, 0, None),
        (usize::MAX, usize::MAX, Some(usize::MAX)),
    ] {
        h.state_mut().page = page;
        h.state_mut().total = total;
        h.run_steps(2);
        assert_eq!(h.state().effective, effective);
        assert_eq!(h.state().page, page);
        assert!(h.state().requests.is_empty());
        if total == 0 || effective == Some(1) {
            assert!(
                h.get_by_label("Previous page")
                    .accesskit_node()
                    .is_disabled()
            );
        }
        if total == 0 || effective == Some(total) {
            assert!(h.get_by_label("Next page").accesskit_node().is_disabled());
            h.get_by_label("Next page").click();
            h.run();
            assert!(h.state().requests.is_empty());
        }
    }
    assert!(h.state().rects.len() <= 9);
}
#[test]
fn narrow_controls_wrap_and_focus_ids_survive_layout_and_sibling_changes() {
    let mut h = harness(260.0);
    h.get_by_label("Page 8").focus();
    h.run();
    let id = h.get_by_label("Page 8").accesskit_node().id();
    h.state_mut().inserted = true;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Page 8").accesskit_node().id(), id);
    assert!(h.get_by_label("Page 8").is_focused());
    assert!(
        h.state()
            .rects
            .iter()
            .all(|r| r.left() >= 8.0 && r.right() <= 252.5),
        "{:?}",
        h.state().rects
    );
    assert!(h.state().rects.windows(2).any(|r| r[1].top() > r[0].top()));
    h.state_mut().disabled = true;
    h.run();
    h.get_by_label("Page 8").click();
    h.run();
    assert!(h.state().requests.is_empty());
}
#[test]
fn compact_navigation_uses_native_tab_order_and_never_invents_a_request() {
    let mut h = harness(420.0);
    h.state_mut().compact = true;
    h.run_steps(2);
    assert!(h.query_by_label("Page 7 of 20").is_some());
    assert!(h.query_by_label("Page 7").is_none());
    h.get_by_label("Previous page").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Next page").is_focused());
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().requests, [8]);
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Outside").is_focused());
    h.check_accessibility();
}

#[test]
fn compact_summary_is_centered_with_its_neighboring_controls() {
    let mut h = harness(760.0);
    h.state_mut().compact = true;
    h.run_steps(2);
    let summary = h.get_by_label("Page 7 of 20").rect();
    assert_eq!(
        summary.center().y,
        h.get_by_label("Previous page").rect().center().y
    );
    assert_eq!(
        summary.center().y,
        h.get_by_label("Next page").rect().center().y
    );
}

#[test]
fn compact_summaries_bold_the_page_numbers_on_a_muted_base() {
    let mut h = harness(600.0);
    h.state_mut().compact = true;
    h.run_steps(2);
    let p = Theme::default().palette;
    let galleys = common::galleys(&h.output().shapes);
    let summary = galleys
        .iter()
        .find(|galley| galley.job.text == "Page 7 of 20")
        .expect("summary painted");
    let colors: Vec<_> = summary
        .job
        .sections
        .iter()
        .map(|section| {
            (
                section.byte_range.start.0..section.byte_range.end.0,
                section.format.color,
            )
        })
        .collect();
    assert_eq!(
        colors,
        [
            (0..5, p.muted_foreground),
            (5..6, p.foreground),
            (6..10, p.muted_foreground),
            (10..12, p.foreground),
        ]
    );
    #[cfg(feature = "bundled-fonts")]
    assert_ne!(
        summary.job.sections[1].format.font_id.family,
        summary.job.sections[0].format.font_id.family,
        "numbers use the bold face"
    );
}

#[test]
fn page_numbers_rest_on_the_foreground_like_the_live_component() {
    let mut h = harness(600.0);
    h.run_steps(2);
    let color = h
        .output()
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) if text.galley.job.text == "8" => text.override_text_color,
            _ => None,
        })
        .expect("page 8 painted");
    assert_eq!(color, Theme::default().palette.foreground);
}
