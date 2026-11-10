use dutchy_egui::{InlineEdit, InlineEditEvent, InlineEditState, Modal, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn escape_cancels_the_rename_before_it_can_dismiss_its_modal() {
    let mut h = Harness::builder()
        .with_size([800.0, 600.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            |ui, p: &mut (bool, InlineEditState, Vec<InlineEditEvent>)| {
                Modal::new("dialog", &mut p.0, "Notebook settings").show(ui.ctx(), |ui| {
                    if let Some(event) = InlineEdit::new("name", "Notebook", &mut p.1)
                        .accessible_name("Notebook name")
                        .show(ui)
                        .event
                    {
                        p.2.push(event);
                    }
                });
            },
            (true, InlineEditState::default(), Vec::new()),
        );
    h.run_steps(4);
    h.get_by_label("Rename Notebook name").click();
    h.run_steps(4);
    assert!(h.get_by_label("Notebook name").is_focused());
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.state().0);
    assert!(!h.state().1.is_editing());
    assert_eq!(h.state().2.last(), Some(&InlineEditEvent::Cancelled));
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.state().0);
}

#[test]
fn independent_keyed_rows_keep_drafts_and_identity_when_reordered() {
    struct Row {
        key: u8,
        edit: InlineEditState,
    }
    let mut h = Harness::builder()
        .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, rows: &mut Vec<Row>| {
                for row in rows {
                    InlineEdit::new(row.key, "Notes", &mut row.edit)
                        .accessible_name(format!("Row {}", row.key))
                        .show(ui);
                }
            },
            vec![
                Row {
                    key: 1,
                    edit: InlineEditState::default(),
                },
                Row {
                    key: 2,
                    edit: InlineEditState::default(),
                },
            ],
        );
    h.run_steps(3);
    h.get_by_label("Rename Row 1").click();
    h.run_steps(3);
    h.get_by_label("Row 1").type_text("First draft");
    h.run_steps(3);
    let id = h.get_by_label("Row 1").accesskit_node().id();
    h.get_by_label("Rename Row 2").click();
    h.run_steps(3);
    h.get_by_label("Row 2").type_text("Second draft");
    h.run_steps(3);
    h.state_mut().reverse();
    h.run_steps(3);
    assert_eq!(h.get_by_label("Row 1").accesskit_node().id(), id);
    assert_eq!(h.state()[0].edit.draft(), Some("Second draft"));
    assert_eq!(h.state()[1].edit.draft(), Some("First draft"));
    h.get_by_role_and_label(Role::Button, "Cancel Row 1")
        .click();
    h.run_steps(3);
    assert!(!h.state()[1].edit.is_editing());
    assert_eq!(h.state()[0].edit.draft(), Some("Second draft"));
    h.check_accessibility();
}
