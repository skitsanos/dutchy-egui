//! Popup attachments overlay the host without consuming layout space.
mod common;
use dutchy_egui::{
    Accent, Button, Dropdown, MenuBar, MenuBarItem, MenuBarState, MenuItem, Mode, Text, Theme,
    Toolbar, ToolbarItem, tokens::space,
};
use egui::{Key, Rect, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[derive(Default)]
struct State {
    bar: MenuBarState,
    geometry: Option<[Rect; 3]>,
}

fn harness(theme: Theme, scale: f32, width: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size([width, 650.0])
        .with_pixels_per_point(scale)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let children = [MenuItem::new(3, "PNG")];
                    let file = [MenuItem::new(1, "Export").submenu(&children)];
                    let view = [MenuItem::new(2, "Show grid").checked(true)];
                    let menus = [
                        MenuBarItem::new("file", "File", &file),
                        MenuBarItem::new("view", "View", &view),
                    ];
                    let bar = MenuBar::new("bar", &mut state.bar, &menus).show(ui);
                    ui.add_space(space::SM);
                    let tools = [ToolbarItem::action(4, "More tools").menu(&file)];
                    let toolbar = Toolbar::new("tools", &tools).show(ui);
                    ui.add_space(space::LG);
                    let following = ui.add(Text::new("Following content"));
                    state.geometry =
                        Some([bar.response.rect, toolbar.response.rect, following.rect]);
                });
            },
            State::default(),
        )
}

fn unchanged(h: &Harness<'_, State>, expected: [Rect; 3]) {
    for ((actual, expected), label) in h.state().geometry.unwrap().into_iter().zip(expected).zip([
        "menu bar",
        "toolbar",
        "following content",
    ]) {
        assert_eq!(
            actual, expected,
            "{label} moved when a menu changed visibility"
        );
    }
}

#[test]
fn opening_switching_and_closing_menu_surfaces_keeps_host_geometry_fixed() {
    for (accent, mode, scale, width) in [
        (Accent::Orange, Mode::Light, 1.0, 1000.0),
        (Accent::Orange, Mode::Dark, 1.5, 1000.0),
        (Accent::Blue, Mode::Light, 2.0, 240.0),
    ] {
        let mut h = harness(Theme::new(accent, mode), scale, width);
        h.run_steps(4);
        let baseline = h.state().geometry.unwrap();
        h.get_by_role_and_label(Role::MenuItem, "File").click();
        h.run_steps(4);
        unchanged(&h, baseline);
        h.key_press(Key::ArrowRight);
        h.run_steps(4);
        unchanged(&h, baseline);
        h.get_by_role_and_label(Role::MenuItem, "View").click();
        h.run_steps(4);
        unchanged(&h, baseline);
        h.key_press(Key::Escape);
        h.run_steps(3);
        unchanged(&h, baseline);
        h.get_by_role_and_label(Role::Button, "More tools").click();
        h.run_steps(4);
        unchanged(&h, baseline);
        h.key_press(Key::ArrowRight);
        h.run_steps(4);
        unchanged(&h, baseline);
        h.key_press(Key::Escape);
        h.run_steps(3);
        unchanged(&h, baseline);
        h.key_press(Key::Escape);
        h.run_steps(4);
        unchanged(&h, baseline);
        h.check_accessibility();
    }
}

#[test]
fn existing_trigger_attachments_do_not_advance_the_cursor_or_disable_the_host() {
    for context in [false, true] {
        for enabled in [false, true] {
            for open in [false, true] {
                let mut h = Harness::builder()
                    .with_size([650.0, 400.0])
                    .with_context_setup(|ctx| common::install(ctx, Theme::default()))
                    .build_ui(move |ui| {
                        let trigger = ui.add(Button::new("Existing target"));
                        let before = ui.cursor();
                        let items = [MenuItem::new(1, "Save")];
                        let mut open = open;
                        let dropdown =
                            Dropdown::new("attached", &mut open, &items).enabled(enabled);
                        let output = if context {
                            dropdown.show_context(ui, &trigger)
                        } else {
                            dropdown.show_for(ui, &trigger)
                        };
                        assert_eq!(before, ui.cursor(), "context={context}, enabled={enabled}");
                        assert_eq!(output.trigger.id, trigger.id);
                        assert!(
                            output.trigger.enabled(),
                            "the host owns its existing target"
                        );
                    });
                h.run_steps(4);
            }
        }
    }
}
