//! Key-event modifier snapshots must determine popup Tab exits.
mod common;
use dutchy_egui::{
    Button, Dropdown, MenuBar, MenuBarItem, MenuBarState, MenuItem, Theme, Toolbar, ToolbarItem,
};
use egui::{Event, Key, Modifiers};
use egui_kittest::{Harness, kittest::Queryable};

#[derive(Clone, Copy, Debug)]
enum Surface {
    Dropdown,
    Attached,
    MenuBar,
    Toolbar,
}
const SURFACES: [Surface; 4] = [
    Surface::Dropdown,
    Surface::Attached,
    Surface::MenuBar,
    Surface::Toolbar,
];
#[derive(Default)]
struct State {
    open: bool,
    bar: MenuBarState,
}
fn harness(surface: Surface) -> Harness<'static, State> {
    Harness::builder()
        .with_size([900.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            move |ui, state: &mut State| {
                let _ = ui.button("Before surface");
                let children = [MenuItem::new(2, "Nested command")];
                let items = [MenuItem::new(1, "Export").submenu(&children)];
                match surface {
                    Surface::Dropdown => {
                        Dropdown::new("commands", &mut state.open, &items)
                            .show(ui, |ui| ui.add(Button::new("Commands")));
                    }
                    Surface::Attached => {
                        let trigger = ui.add(Button::new("Commands"));
                        Dropdown::new("attached", &mut state.open, &items).show_for(ui, &trigger);
                    }
                    Surface::MenuBar => {
                        MenuBar::new(
                            "bar",
                            &mut state.bar,
                            &[MenuBarItem::new("file", "Commands", &items)],
                        )
                        .show(ui);
                    }
                    Surface::Toolbar => {
                        Toolbar::new("tools", &[ToolbarItem::action(0, "Commands").menu(&items)])
                            .show(ui);
                    }
                }
                Toolbar::new("following", &[ToolbarItem::action(3, "After surface")]).show(ui);
            },
            State::default(),
        )
}
fn open_nested(h: &mut Harness<'_, State>) {
    h.get_by_label("Commands").click();
    h.run_steps(4);
    h.key_press(Key::ArrowRight);
    h.run_steps(4);
    assert!(h.get_by_label("Nested command").is_focused());
}
fn raw_tab(h: &mut Harness<'_, State>, key_modifiers: Modifiers, frame_modifiers: Modifiers) {
    h.input_mut()
        .events
        .push(Event::ModifiersChanged(key_modifiers));
    for pressed in [true, false] {
        h.input_mut().events.push(Event::Key {
            key: Key::Tab,
            physical_key: Some(Key::Tab),
            pressed,
            repeat: false,
            modifiers: key_modifiers,
        });
    }
    h.input_mut()
        .events
        .push(Event::ModifiersChanged(frame_modifiers));
    h.run_steps(4);
}
#[test]
fn tab_follows_the_key_snapshot_even_when_frame_modifiers_change() {
    for surface in SURFACES {
        for (key, frame) in [
            (Modifiers::NONE, Modifiers::SHIFT),
            (Modifiers::SHIFT, Modifiers::NONE),
        ] {
            let mut h = harness(surface);
            open_nested(&mut h);
            raw_tab(&mut h, key, frame);
            let expected = if key.shift {
                "Before surface"
            } else {
                "After surface"
            };
            assert!(
                h.get_by_label(expected).is_focused(),
                "{surface:?}: key={key:?}, frame={frame:?}; expected {expected}"
            );
        }
    }
}
#[test]
fn control_tab_is_left_for_host_shortcuts() {
    for surface in SURFACES {
        let mut h = harness(surface);
        open_nested(&mut h);
        raw_tab(&mut h, Modifiers::CTRL, Modifiers::NONE);
        assert!(
            h.query_by_label("Nested command").is_some(),
            "{surface:?} closed for Ctrl+Tab"
        );
    }
}
#[test]
fn tab_from_a_root_menu_moves_past_its_trigger_in_both_directions() {
    for surface in SURFACES {
        for shift in [false, true] {
            let mut h = harness(surface);
            h.get_by_label("Commands").click();
            h.run_steps(4);
            assert!(h.get_by_label("Export").is_focused(), "{surface:?} opened");
            let modifiers = if shift {
                Modifiers::SHIFT
            } else {
                Modifiers::NONE
            };
            h.key_press_modifiers(modifiers, Key::Tab);
            h.run_steps(4);
            let expected = if shift {
                "Before surface"
            } else {
                "After surface"
            };
            assert!(h.query_by_label("Export").is_none(), "{surface:?} closed");
            assert!(
                h.get_by_label(expected).is_focused(),
                "{surface:?}: shift={shift}; expected {expected}"
            );
        }
    }
}
