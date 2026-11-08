//! CodeBlock stays functional with or without the Markdown parser feature.
mod common;
use dutchy_egui::{Card, CardVariant, CodeBlock, Theme};
use egui::{OutputCommand, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn wrapped_code_respects_the_viewport_and_keeps_its_full_accessible_value() {
    let code =
        "let message = a_long_expression + another_expression + a_third_expression;".repeat(3);
    let source = code.clone();
    let mut h = Harness::builder()
        .with_size([240.0, 300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(move |ui| {
            ui.add(
                CodeBlock::new("code", source.as_str())
                    .wrap(true)
                    .line_numbers(true)
                    .copy(false),
            );
        });
    h.run();
    let node = h.get_by_role_and_label(Role::Code, "Code");
    assert_eq!(
        node.accesskit_node().value().as_deref(),
        Some(code.as_str())
    );
    let galley = common::galleys(&h.output().shapes)
        .into_iter()
        .find(|g| g.job.text == code)
        .unwrap();
    assert!(galley.rows.len() > 1, "long code wraps when requested");
    assert!(
        galley.size().x <= 208.0,
        "wrapped width: {:?}",
        galley.size()
    );
    assert!(h.query_by_role_and_label(Role::Button, "Copy").is_none());
}

#[test]
fn copy_and_disabled_parent_behavior_are_available_without_the_parser() {
    let mut h = Harness::builder()
        .with_size([500.0, 300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, enabled: &mut bool| {
                ui.add_enabled_ui(*enabled, |ui| {
                    ui.add(CodeBlock::new("code", "let value = 42;").language("rust"));
                });
            },
            true,
        );
    h.run();
    h.get_by_role_and_label(Role::Button, "Copy").click();
    h.step();
    assert!(
        h.output()
            .platform_output
            .commands
            .iter()
            .any(|c| matches!(c, OutputCommand::CopyText(t) if t == "let value = 42;"))
    );
    *h.state_mut() = false;
    h.run();
    assert!(
        h.get_by_role_and_label(Role::Button, "Copied")
            .accesskit_node()
            .is_disabled()
    );
    h.get_by_role_and_label(Role::Button, "Copied").click();
    h.step();
    assert!(
        !h.output()
            .platform_output
            .commands
            .iter()
            .any(|c| matches!(c, OutputCommand::CopyText(_)))
    );
}

#[test]
fn owned_code_surface_resets_inverse_card_ink_for_its_language_label() {
    let mut h = Harness::builder()
        .with_size([500.0, 300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            Card::new().variant(CardVariant::Dark).show(ui, |ui| {
                ui.add(CodeBlock::new("code", "let value = 42;").language("rust"));
            });
        });
    h.run();
    let label = common::galleys(&h.output().shapes)
        .into_iter()
        .find(|g| g.job.text == "RUST")
        .unwrap();
    assert_eq!(
        label.job.sections[0].format.color,
        Theme::default().palette.muted_foreground
    );
}
