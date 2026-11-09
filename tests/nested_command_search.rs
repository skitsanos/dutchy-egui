//! Native command controls and interaction regressions.
mod common;
use dutchy_egui::{
    CommandPalette, MenuItem, SearchState, Theme,
    egui::{Key, accesskit::Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
struct State {
    search: SearchState,
    actions: Vec<u8>,
    disabled: bool,
    renamed: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([700.0, 550.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, s: &mut State| {
                let png = [MenuItem::new(1, "PNG image")];
                let svg = [MenuItem::new(1, "SVG vector")];
                let children = [
                    MenuItem::new(2, "Raster").submenu(&png),
                    MenuItem::new(3, "Vector").submenu(&svg),
                ];
                let items = [
                    MenuItem::new(4, if s.renamed { "Publish" } else { "Export" })
                        .submenu(&children)
                        .enabled(!s.disabled),
                    MenuItem::new(5, "Empty").submenu(&[]),
                ];
                if let Some(out) = CommandPalette::new("nested", &mut s.search, &items)
                    .shortcut(None)
                    .show(ui.ctx())
                    && let Some(action) = out.action
                {
                    s.actions.push(action);
                }
            },
            State {
                search: SearchState {
                    open: true,
                    query: String::new(),
                },
                actions: Vec::new(),
                disabled: false,
                renamed: false,
            },
        )
}
#[test]
fn paths_are_searchable_and_branches_never_execute() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_label("Search commands").type_text("export PNG");
    h.run_steps(3);
    h.get_by_role_and_label(Role::ListBoxOption, "PNG image");
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "SVG vector")
            .is_none()
    );
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Export")
            .is_none()
    );
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().actions, [1]);
    h.check_accessibility();
}
#[test]
fn full_key_paths_distinguish_equal_leaf_values_and_labels_can_change() {
    let mut h = harness();
    h.run_steps(3);
    let png = h
        .get_by_role_and_label(Role::ListBoxOption, "PNG image")
        .accesskit_node()
        .id();
    let svg = h
        .get_by_role_and_label(Role::ListBoxOption, "SVG vector")
        .accesskit_node()
        .id();
    assert_ne!(png, svg);
    h.state_mut().renamed = true;
    h.run_steps(3);
    assert_eq!(
        h.get_by_role_and_label(Role::ListBoxOption, "PNG image")
            .accesskit_node()
            .id(),
        png
    );
    h.state_mut().disabled = true;
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "PNG image")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().actions.is_empty());
}
