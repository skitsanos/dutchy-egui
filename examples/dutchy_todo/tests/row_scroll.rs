use super::harness;
use crate::store::TaskDraft;
use egui::{Key, accesskit::Role};
use egui_kittest::kittest::Queryable;

const FIRST: &str = "Order seed trays";
const LAST: &str = "Clear the gutters";

#[test]
fn keyboard_navigation_and_completion_reveal_the_focused_task() {
    let mut h = harness();
    let project = h.state().selected.unwrap();
    for index in 0..28 {
        h.state_mut()
            .store
            .add_task(
                project,
                TaskDraft {
                    title: format!("Extra task {index}"),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::ListItem, LAST).rect().top() > 900.0);
    h.get_by_role_and_label(Role::ListItem, FIRST).focus();
    h.run();
    h.key_press(Key::End);
    h.run_steps(4);
    let last = h.get_by_role_and_label(Role::ListItem, LAST);
    assert!(last.is_focused());
    assert!(last.rect().bottom() <= 900.0);
    h.key_press(Key::Home);
    h.run_steps(4);
    assert!(h.get_by_role_and_label(Role::ListItem, FIRST).is_focused());
    h.key_press(Key::Space);
    h.run_steps(4);
    assert!(
        h.state()
            .store
            .tasks()
            .any(|task| task.title == FIRST && task.done)
    );
    let completed = h.get_by_role_and_label(Role::ListItem, FIRST);
    assert!(completed.is_focused());
    assert!(completed.rect().bottom() <= 900.0);
}
