//! Disclosure and tab rendering plus host-managed application workflows.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{
    Accent, Accordion, AccordionItem, AccordionState, AccordionVariant, Input, Mode, Tab, Tabs,
    TabsVariant, Text, TextColor, TextSize, Theme,
};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

struct State {
    single: AccordionState<u8>,
    multiple: AccordionState<u8>,
    selected: Option<u8>,
    interval: Option<u8>,
    document: Option<u8>,
    input: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            single: AccordionState::Single(Some(1)),
            multiple: AccordionState::Multiple(vec![2]),
            selected: Some(1),
            interval: Some(1),
            document: Some(1),
            input: "Studio workspace".into(),
        }
    }
}
fn matrix(ui: &mut egui::Ui, state: &mut State) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Reveal. Navigate.").size(TextSize::Xl4));
    ui.add(Text::new("Controlled state / native input / sharp focus").color(TextColor::Muted));
    ui.add_space(16.0);
    let items = [
        AccordionItem::new(1, "A clear starting point"),
        AccordionItem::new(2, "More context"),
        AccordionItem::new(3, "Unavailable section").enabled(false),
    ];
    Accordion::new("default", &mut state.single, &items)
        .accessible_name("Getting started")
        .show(ui, |ui, _| {
            ui.add(
                Text::new(
                    "Host-owned expansion and meaningful guidance. No hidden panel is rendered.",
                )
                .size(TextSize::Sm),
            );
        });
    ui.add_space(24.0);
    let items = [
        AccordionItem::new(1, "Borders give structure"),
        AccordionItem::new(2, "Interactive content"),
    ];
    Accordion::new("bordered", &mut state.multiple, &items)
        .variant(AccordionVariant::Bordered)
        .accessible_name("Workspace details")
        .show(ui, |ui, _| {
            ui.add(Input::new("name", &mut state.input).accessible_name("Example name"));
        });
    ui.add_space(24.0);
    let tabs = [
        Tab::new(1, "Overview"),
        Tab::new(2, "Features"),
        Tab::new(3, "History").enabled(false),
    ];
    Tabs::new("content", &mut state.selected, &tabs)
        .accessible_name("Content views")
        .show(ui, |ui, _| {
            ui.add(
                Text::new("Arrow keys move focus; Enter or Space switches the visible panel.")
                    .size(TextSize::Sm),
            );
        });
    ui.add_space(16.0);
    let tabs = [Tab::new(1, "Monthly"), Tab::new(2, "Yearly")];
    Tabs::new("interval", &mut state.interval, &tabs)
        .variant(TabsVariant::Segmented)
        .accessible_name("Billing interval")
        .show(ui, |ui, _| {
            ui.add(Text::new("Flat surfaces. Equal-width segments.").size(TextSize::Sm));
        });
    let tabs = [
        Tab::new(1, "Document one").closable(true),
        Tab::new(2, "Document two").closable(true),
    ];
    Tabs::new("documents", &mut state.document, &tabs)
        .accessible_name("Documents")
        .show(ui, |ui, _| {
            ui.add(
                Text::new("Closing is a request. Your application owns the data.")
                    .size(TextSize::Sm),
            );
        });
}

#[test]
fn disclosure_states_render_across_themes_and_display_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "disclosure-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "disclosure-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "disclosure-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([760.0, 1140.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(matrix, State::default());
        h.get_by_role_and_label(Role::Tab, "Overview").focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn unselected_keyboard_tab_retains_its_focus_cue() {
    let mut h = Harness::builder()
        .with_size([760.0, 1140.0])
        .with_render_cursor(false)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(matrix, State::default());
    h.get_by_role_and_label(Role::Tab, "Overview").focus();
    h.run_steps(2);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(h.state().selected, Some(1));
    h.snapshot("tab-roving-focus");
}

#[test]
fn accordion_focus_has_four_complete_edges_above_adjacent_content() {
    let mut h = Harness::builder()
        .with_size([760.0, 1140.0])
        .with_render_cursor(false)
        .with_theme(egui::Theme::Dark)
        .with_pixels_per_point(1.5)
        .with_context_setup(|ctx| {
            common::install(
                ctx,
                Theme::new(Accent::Orange, Mode::Dark).with_reduced_motion(true),
            )
        })
        .build_ui_state(matrix, State::default());
    h.get_by_role_and_label(Role::Button, "Borders give structure")
        .focus();
    h.run_steps(2);
    h.snapshot("accordion-keyboard-focus");
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
fn disclosure_pages_render_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (page, prefix) in [
        (gallery::Page::Accordions, "accordions"),
        (gallery::Page::Tabs, "tabs"),
    ] {
        for (size, suffix) in [([1200.0, 2200.0], "desktop"), ([480.0, 3200.0], "narrow")] {
            let mut h = example(page, size);
            h.run_steps(2);
            h.check_accessibility();
            h.render()
                .unwrap()
                .save(format!("target/visual-proof/{prefix}-{suffix}.png"))
                .unwrap();
        }
    }
}

#[test]
fn accordion_example_preserves_its_form_and_keeps_child_actions_independent() {
    let mut h = example(gallery::Page::Accordions, [1200.0, 2000.0]);
    h.get_by_role_and_label(Role::Button, "Can I compose interactive content?")
        .click();
    h.run_steps(2);
    h.get_by_label("Example workspace name").focus();
    h.run();
    h.get_by_label("Example workspace name")
        .type_text(" with collaborators");
    h.run();
    let name = h.state().disclosures.name.clone();
    h.get_by_label("Apply example name").click();
    h.run_steps(2);
    assert_eq!(h.state().disclosures.applies, 1);
    assert!(h.state().disclosures.single.is_open(&"controls"));
    h.get_by_role_and_label(Role::Button, "What is Dutchy?")
        .click();
    h.run_steps(2);
    assert!(h.query_by_label("Example workspace name").is_none());
    h.get_by_role_and_label(Role::Button, "Can I compose interactive content?")
        .click();
    h.run_steps(2);
    assert_eq!(h.state().disclosures.name, name);
}

#[test]
fn tab_example_saves_settings_and_closing_documents_retains_host_data() {
    let mut h = example(gallery::Page::Tabs, [1200.0, 2300.0]);
    h.get_by_role_and_label(Role::Tab, "Workspace settings")
        .click();
    h.run_steps(2);
    h.get_by_label("Save tab settings").click();
    h.run_steps(2);
    assert_eq!(h.state().tabbed.saves, 1);
    h.get_by_role_and_label(Role::Tab, "Yearly").click();
    h.run_steps(2);
    assert_eq!(h.state().tabbed.billing, Some("yearly"));
    h.get_by_label("Document 1 note").focus();
    h.run();
    h.get_by_label("Document 1 note").type_text("Keep my note");
    h.run();
    h.get_by_label("Close Document 1").click();
    h.run_steps(3);
    assert_eq!(h.state().tabbed.document, Some(2));
    assert_eq!(
        h.state().tabbed.notes.get(&1).map(String::as_str),
        Some("Keep my note")
    );
    assert!(
        h.get_by_role_and_label(Role::Tab, "Document 2")
            .is_focused()
    );
    h.state_mut().tabbed.documents.retain(|id| *id == 2);
    h.run_steps(2);
    h.key_press(egui::Key::Delete);
    h.run_steps(3);
    assert!(h.state().tabbed.documents.is_empty());
    assert!(h.get_by_label("Add document").is_focused());
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().tabbed.document, Some(6));
}
