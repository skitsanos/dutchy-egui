use super::support::{Page, harness, selected};
use dutchy_egui::{List, ListItem, ListState, Theme, tokens::list::PADDING};
use egui::accesskit::Role;
use egui_kittest::{Harness, kittest::Queryable};

const WIDTH: f32 = 300.0;
const LONG: &str =
    "A very long row that must elide inside the viewport without growing or wrapping";

fn content_harness(items: Vec<ListItem<'static, u32>>) -> Harness<'static, ListState<u32>> {
    let mut h = Harness::builder()
        .with_size([WIDTH, 200.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            move |ui, state: &mut ListState<u32>| {
                let _ = List::new("content", state, &items)
                    .accessible_name("Content")
                    .empty_text("No projects yet")
                    .show(ui);
            },
            ListState::default(),
        );
    h.run_steps(2);
    h
}

#[test]
fn empty_lists_show_the_host_supplied_text() {
    let h = content_harness(Vec::new());
    h.get_by_label("No projects yet");
    assert!(h.query_by_label("No items").is_none());
    h.check_accessibility();
}

#[test]
fn default_rows_elide_long_labels_inside_the_row_padding() {
    let h = content_harness(vec![ListItem::new(0, LONG)]);
    let galleys = super::common::galleys(&h.output().shapes);
    let galley = galleys
        .iter()
        .find(|galley| galley.job.text == LONG)
        .expect("the long label is painted");
    assert!(galley.elided, "the label is cut with an ellipsis");
    assert!(galley.size().x <= WIDTH - 2.0 * PADDING);
}

fn row_text(node: &egui_kittest::kittest::AccessKitNode<'_>) -> bool {
    node.role() == Role::Label && node.value().as_deref() == Some("Row 00002")
}

#[test]
fn clicking_composed_row_text_hovers_and_selects_that_row() {
    let mut h = harness(Page::new(5));
    // Click the text itself, not the empty space to its right.
    let text = h.get_by(row_text);
    let (rect, row) = (
        text.rect(),
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
            .rect(),
    );
    assert!(rect.width() < row.width() / 2.0, "{rect:?} in {row:?}");
    text.hover();
    h.run_steps(2);
    assert_eq!(h.state().hovered, Some(2));
    h.get_by(row_text).click();
    h.run_steps(3);
    assert_eq!(selected(&h), [2]);
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
            .is_focused()
    );
}
