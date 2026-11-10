use dutchy_egui::{List, ListItem, ListState, Theme, tokens::list::PADDING};
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
