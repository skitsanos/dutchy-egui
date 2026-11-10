use dutchy_egui::{Accent, Mode, Theme, TreeNode, TreeState, TreeView};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
#[test]
fn tree_hierarchy_renders_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, "blue-2x"),
    ] {
        let mut state = TreeState::default();
        state.expanded.extend([1, 2]);
        state.rows.selected.insert(3);
        let mut h = Harness::builder()
            .with_size([460.0, 340.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_context_setup(move |ctx| {
                super::common::install(ctx, Theme::new(accent, mode).with_reduced_motion(true))
            })
            .build_ui_state(
                |ui, state: &mut TreeState<u32>| {
                    let leaves = [
                        TreeNode::new(3, "Design brief.md"),
                        TreeNode::new(4, "Research.md").enabled(false),
                    ];
                    // Archive stays closed so the baselines show both chevrons.
                    let archived = [TreeNode::new(7, "2025 notes.md")];
                    let folders = [
                        TreeNode::new(2, "Projects").children(&leaves),
                        TreeNode::new(5, "Archive").children(&archived),
                    ];
                    let roots = [
                        TreeNode::new(1, "Workspace").children(&folders),
                        TreeNode::new(6, "Trash"),
                    ];
                    TreeView::new("visual", state, &roots)
                        .accessible_name("Folders")
                        .max_height(288.0)
                        .show(ui);
                },
                state,
            );
        h.run_steps(3);
        h.get_by_role_and_label(egui::accesskit::Role::TreeItem, "Design brief.md")
            .focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(format!("tree-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
