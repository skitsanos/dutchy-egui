use dutchy_egui::{Accent, List, ListItem, ListSelectionMode, ListState, Mode, Text, Theme};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

const LONG: &str = "A very long row that ends in an ellipsis instead of growing or wrapping";

fn workspace(ui: &mut egui::Ui, state: &mut ListState<u32>) {
    ui.add(Text::heading("Your workspace."));
    let items = [
        ListItem::new(0, "Inbox"),
        ListItem::new(1, "Product sketches"),
        ListItem::new(2, "Research archive"),
        ListItem::new(3, "Unavailable").enabled(false),
        ListItem::new(4, LONG),
    ];
    let _ = List::new("preview", state, &items)
        .selection_mode(ListSelectionMode::Multiple)
        .accessible_name("Workspace")
        .max_height(240.0)
        .show(ui);
}

#[test]
fn list_surfaces_render_in_three_themes_and_scales_with_elided_content() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, "blue-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut state = ListState::default();
        state.selected.extend([1, 2]);
        state.focused = Some(2);
        let mut h = Harness::builder()
            .with_size([460.0, 310.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_context_setup(move |ctx| super::common::install(ctx, theme))
            .build_ui_state(workspace, state);
        h.run_steps(3);
        h.get_by_role_and_label(egui::accesskit::Role::ListBoxOption, "Research archive")
            .focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(format!("list-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
