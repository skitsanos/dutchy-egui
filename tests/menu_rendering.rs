//! Menu surfaces, popup placement, modal rendering, and showcase workflows.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{
    Accent, Button, CommandPalette, Dropdown, Menu, MenuItem, MenuItemVariant, MenuWidth, Mode,
    SearchState, SearchableSelect, SelectOption, Text, Theme,
};
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

fn items() -> [MenuItem<'static, u8>; 4] {
    [
        MenuItem::new(1, "Create project")
            .group("Workspace")
            .shortcut("⌘ N"),
        MenuItem::new(2, "Duplicate project")
            .group("Workspace")
            .shortcut("⌘ D"),
        MenuItem::new(3, "Archive project")
            .group("Manage")
            .enabled(false),
        MenuItem::new(4, "Delete project")
            .group("Manage")
            .variant(MenuItemVariant::Destructive),
    ]
}
#[derive(Default)]
struct State {
    open: bool,
    search: SearchState,
    value: Option<u8>,
    palette: SearchState,
}
fn matrix(ui: &mut egui::Ui, state: &mut State) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Choose with clarity."));
    Menu::new("menu", &items())
        .accessible_name("Workspace actions")
        .show(ui);
    ui.add_space(24.0);
    Dropdown::new("actions", &mut state.open, &items())
        .width(MenuWidth::Small)
        .show(ui, |ui| ui.add(Button::new("More actions")));
    ui.add_space(24.0);
    let options = [
        SelectOption::new(1, "egui"),
        SelectOption::new(2, "Iced"),
        SelectOption::new(3, "Slint").enabled(false),
    ];
    SearchableSelect::new("framework", &mut state.value, &mut state.search, &options)
        .accessible_name("Search frameworks")
        .show(ui);
    CommandPalette::new("commands", &mut state.palette, &items()).show(ui.ctx());
}
#[test]
fn menus_and_search_results_render_across_themes_and_scales() {
    let mut snapshots = SnapshotResults::new();
    for (accent, mode, scale, prefix) in [
        (Accent::Orange, Mode::Light, 1.0, "menus-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "menus-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "menus-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([560.0, 740.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(matrix, State::default());
        h.get_by_role_and_label(Role::MenuItem, "Delete project")
            .focus();
        h.run_steps(3);
        h.snapshot(format!("{prefix}-inline"));
        h.get_by_label("MORE ACTIONS").click();
        h.run_steps(4);
        h.key_press(Key::ArrowDown);
        h.run_steps(2);
        h.check_accessibility();
        h.snapshot(format!("{prefix}-dropdown"));
        h.key_press(Key::Escape);
        h.run_steps(3);
        h.get_by_label("Search frameworks").focus();
        h.run_steps(2);
        h.key_press(Key::ArrowDown);
        h.run_steps(4);
        h.snapshot(format!("{prefix}-search"));
        h.key_press(Key::Escape);
        h.run_steps(3);
        h.key_press_modifiers(Modifiers::COMMAND, Key::K);
        h.run_steps(4);
        h.check_accessibility();
        h.snapshot(format!("{prefix}-commands"));
        snapshots.extend_harness(&mut h);
    }
    snapshots.unwrap();
}
fn example(page: gallery::Page, size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn menu_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Menus, "menus"),
        (gallery::Page::Dropdowns, "dropdowns"),
    ] {
        for (size, suffix) in [([1200.0, 2000.0], "desktop"), ([480.0, 2800.0], "narrow")] {
            let mut h = example(page, size);
            h.run_steps(3);
            h.check_accessibility();
            h.render()
                .unwrap()
                .save(format!("target/visual-proof/{prefix}-{suffix}.png"))
                .unwrap();
        }
    }
}
#[test]
fn showcase_dropdown_search_and_global_commands_update_host_state() {
    let mut h = example(gallery::Page::Dropdowns, [1200.0, 2000.0]);
    h.get_by_label("PROJECT ACTIONS").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Duplicate project")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().menus.actions, 1);
    assert!(!h.state().menus.open);
    h.get_by_label("Search frameworks").focus();
    h.run_steps(2);
    h.get_by_label("Search frameworks").type_text("egui");
    h.run_steps(4);
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().menus.framework, Some("egui"));
    h.get_by_label("OPEN COMMAND PALETTE").click();
    h.run_steps(4);
    h.get_by_label("Search commands").type_text("create");
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert_eq!(h.state().menus.actions, 2);
    h.state_mut().page = gallery::Page::Overview;
    h.run_steps(3);
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    assert!(h.state().menus.palette.open);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert!(!h.state().menus.palette.open);
}
#[test]
fn command_dialog_fits_a_small_viewport() {
    let mut h = Harness::builder()
        .with_size([280.0, 220.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            matrix,
            State {
                palette: SearchState {
                    open: true,
                    query: String::new(),
                },
                ..Default::default()
            },
        );
    h.run_steps(5);
    let rect = h
        .get_by_role_and_label(Role::Dialog, "Command palette")
        .rect();
    assert!(
        rect.left() >= 0.0 && rect.right() <= 280.0 && rect.top() >= 0.0 && rect.bottom() <= 220.0,
        "{rect:?}"
    );
    h.check_accessibility();
}
