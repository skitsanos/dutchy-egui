//! Card action precedence, keyboard behavior, identity, and scoped content.

mod common;
use dutchy_egui::{Button, Card, CardVariant, Theme};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    opened: usize,
    child: usize,
    banner: bool,
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([640.0, 700.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.banner {
                    ui.label("Inserted content");
                }
                let card = Card::default()
                    .variant(CardVariant::Accent)
                    .show_interactive(ui, "project", "Open project", |ui| {
                        Card::title(ui, "Project Phoenix");
                        Card::description(ui, "A composable application card.");
                        if ui.add(Button::new("Child action")).clicked() {
                            state.child += 1;
                        }
                    });
                if card.response.clicked() {
                    state.opened += 1;
                }
                let disabled = Card::new().enabled(false).show_interactive(
                    ui,
                    "disabled",
                    "Disabled card",
                    |ui| {
                        Card::title(ui, "Disabled");
                    },
                );
                if disabled.response.clicked() {
                    state.opened += 100;
                }
                ui.add_enabled_ui(false, |ui| {
                    let disabled = Card::new().show_interactive(
                        ui,
                        "parent-disabled",
                        "Parent disabled card",
                        |ui| {
                            Card::title(ui, "Parent disabled");
                        },
                    );
                    if disabled.response.clicked() {
                        state.opened += 100;
                    }
                });
            },
            State::default(),
        )
}

#[test]
fn child_clicks_do_not_activate_the_card_and_keyboard_activation_is_native() {
    let mut h = harness();
    h.get_by_label("Child action").click();
    h.run();
    assert_eq!(h.state().child, 1);
    assert_eq!(h.state().opened, 0);
    h.get_by_label("Project Phoenix").click();
    h.run();
    assert_eq!(h.state().opened, 1);
    h.get_by_label("Open project").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    h.key_press(egui::Key::Space);
    h.run();
    assert_eq!(h.state().opened, 3);
}

#[test]
fn disabled_cards_cannot_activate_and_stable_ids_preserve_focus() {
    let mut h = harness();
    for name in ["Disabled card", "Parent disabled card"] {
        assert!(h.get_by_label(name).accesskit_node().is_disabled());
        h.get_by_label(name).click();
        h.run();
    }
    assert_eq!(h.state().opened, 0);
    h.get_by_label("Open project").focus();
    h.run();
    let id = h.get_by_label("Open project").accesskit_node().id();
    h.state_mut().banner = true;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Open project").accesskit_node().id(), id);
    assert!(h.get_by_label("Open project").is_focused());
}

#[test]
fn surfaces_do_not_leak_style_and_nested_cards_restore_text_colors() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.style().clone();
            for variant in CardVariant::ALL {
                Card::new().variant(variant).show(ui, |ui| {
                    let inverse = matches!(variant, CardVariant::Dark | CardVariant::Primary);
                    assert_eq!(ui.visuals().override_text_color.is_some(), inverse);
                    Card::new().show(ui, |ui| {
                        assert_eq!(ui.visuals().override_text_color, None);
                        assert_eq!(
                            ui.visuals().text_color(),
                            Theme::default().palette.foreground
                        );
                    });
                });
            }
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.run();
}

#[test]
fn header_and_footer_gaps_render_as_documented() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            Card::new().show(ui, |ui| {
                Card::header(ui, |ui| Card::title(ui, "Quarterly review"));
                ui.label("Body copy");
                Card::footer(ui, |ui| ui.add(Button::new("Approve")));
            });
        });
    h.run_steps(2);
    let title = h
        .get_by_role_and_label(egui::accesskit::Role::Heading, "Quarterly review")
        .rect();
    let body = h.get_by_label("Body copy").rect();
    let action = h.get_by_label("Approve").rect();
    let header = body.top() - title.bottom();
    let footer = action.top() - body.bottom();
    assert!((header - 16.0).abs() < 0.5, "header gap {header}");
    assert!(
        (footer - 41.0).abs() < 0.5,
        "footer gap, divider and gap {footer}"
    );
}
#[test]
fn card_titles_have_no_display_tracking() {
    let mut h = harness();
    h.run_steps(2);
    let spacing = common::galleys(&h.output().shapes)
        .into_iter()
        .find(|galley| galley.job.text.to_lowercase().contains("project phoenix"))
        .map(|galley| galley.job.sections[0].format.extra_letter_spacing)
        .expect("title painted");
    assert_eq!(spacing, 0.0);
}
