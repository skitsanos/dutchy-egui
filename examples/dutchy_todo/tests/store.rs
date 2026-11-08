//! Store operations, counts, filters and undo data.

use crate::store::{Filter, Priority, Store, TaskDraft};
use crate::support::data::FIXED_TODAY;

fn draft(title: &str, priority: Priority, due: Option<i64>) -> TaskDraft {
    TaskDraft {
        title: title.into(),
        notes: String::new(),
        due: due.and_then(|days| FIXED_TODAY.add_days(days)),
        priority,
    }
}

#[test]
fn tasks_keep_stable_ids_and_restore_under_the_same_key() {
    let mut store = Store::new();
    let project = store.add_project("  Garden  ");
    assert_eq!(store.project(project).unwrap().name, "Garden");
    let first = store.add_task(project, draft("First", Priority::Low, None));
    let second = store.add_task(project, draft("Second", Priority::Low, None));
    let removed = store.remove_task(first).unwrap();
    assert!(store.task(first).is_none());
    let third = store.add_task(project, draft("Third", Priority::Low, None));
    assert_ne!(third, first, "ids are never reused");
    store.restore_task(removed);
    let order: Vec<_> = store.tasks().map(|task| task.id).collect();
    assert_eq!(
        order,
        [first, second, third],
        "creation order survives undo"
    );
}

#[test]
fn blank_titles_fall_back_and_updates_keep_the_old_title() {
    let mut store = Store::new();
    let project = store.add_project("");
    assert_eq!(store.project(project).unwrap().name, "Untitled project");
    let id = store.add_task(project, draft("   ", Priority::High, None));
    assert_eq!(store.task(id).unwrap().title, "Untitled task");
    let mut update = draft("", Priority::Low, Some(2));
    update.notes = "Keep going\n\n".into();
    assert!(store.update_task(id, update));
    let task = store.task(id).unwrap();
    assert_eq!(task.title, "Untitled task");
    assert_eq!(task.notes, "Keep going");
    assert_eq!(task.priority, Priority::Low);
    assert_eq!(task.due, FIXED_TODAY.add_days(2));
    assert!(!store.update_task(crate::store::TaskId(99), draft("x", Priority::Low, None)));
}

#[test]
fn counts_distinguish_open_done_overdue_and_due_today() {
    let mut store = Store::new();
    let project = store.add_project("Garden");
    store.add_task(project, draft("Overdue", Priority::Medium, Some(-1)));
    store.add_task(project, draft("Today", Priority::Medium, Some(0)));
    store.add_task(project, draft("Later", Priority::Medium, Some(3)));
    let done = store.add_task(project, draft("Done but late", Priority::Medium, Some(-5)));
    store.set_done(done, true);
    let counts = store.counts(project, FIXED_TODAY);
    assert_eq!(
        (counts.total, counts.done, counts.overdue, counts.due_today),
        (4, 1, 1, 1)
    );
    assert_eq!(counts.active(), 3);
    assert_eq!(counts.percent(), 25.0);
    assert_eq!(
        store
            .counts(project, FIXED_TODAY.add_days(10).unwrap())
            .overdue,
        3
    );
    assert_eq!(Store::new().counts(project, FIXED_TODAY).percent(), 0.0);
}

#[test]
fn visible_applies_filter_search_and_a_stable_order() {
    let mut store = Store::new();
    let project = store.add_project("Garden");
    let other = store.add_project("Kitchen");
    let low = store.add_task(project, draft("Water ferns", Priority::Low, Some(1)));
    let high = store.add_task(project, draft("Order trays", Priority::High, Some(5)));
    let undated = store.add_task(project, draft("Compare tanks", Priority::High, None));
    let done = store.add_task(project, draft("Clear gutters", Priority::High, Some(-1)));
    store.set_done(done, true);
    store.add_task(other, draft("Measure alcove", Priority::High, None));
    let ids = |store: &Store, filter, query: &str| -> Vec<_> {
        store
            .visible(project, filter, query)
            .iter()
            .map(|task| task.id)
            .collect()
    };
    assert_eq!(ids(&store, Filter::All, ""), [high, undated, low, done]);
    assert_eq!(ids(&store, Filter::Active, ""), [high, undated, low]);
    assert_eq!(ids(&store, Filter::Done, ""), [done]);
    assert_eq!(ids(&store, Filter::All, "  TRAY "), [high]);
    assert!(ids(&store, Filter::Done, "tray").is_empty());
    assert!(store.toggle_done(high) == Some(true));
    assert_eq!(ids(&store, Filter::Active, ""), [undated, low]);
    assert!(store.toggle_done(crate::store::TaskId(42)).is_none());
}

#[test]
fn removing_a_project_takes_its_tasks_and_restores_them_in_place() {
    let mut store = Store::new();
    let garden = store.add_project("Garden");
    let kitchen = store.add_project("Kitchen");
    let reading = store.add_project("Reading");
    let task = store.add_task(kitchen, draft("Measure", Priority::Medium, None));
    store.add_task(garden, draft("Dig", Priority::Medium, None));
    let removed = store.remove_project(kitchen).unwrap();
    assert_eq!(removed.index, 1);
    assert_eq!(removed.tasks.len(), 1);
    assert!(store.task(task).is_none());
    assert_eq!(store.tasks().count(), 1);
    let names: Vec<_> = store.projects().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Garden", "Reading"]);
    store.restore_project(removed);
    let names: Vec<_> = store.projects().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Garden", "Kitchen", "Reading"]);
    assert_eq!(store.task(task).unwrap().project, kitchen);
    assert!(store.remove_project(reading).is_some());
    assert!(store.remove_project(reading).is_none());
}

#[test]
fn seeded_data_is_deterministic_and_relative_to_today() {
    let (first, selected) = crate::data::seeded(FIXED_TODAY);
    let (second, _) = crate::data::seeded(FIXED_TODAY);
    assert_eq!(first, second);
    assert_eq!(first.projects().len(), 3);
    assert_eq!(first.project(selected).unwrap().name, "Garden renovation");
    let counts = first.counts(selected, FIXED_TODAY);
    assert_eq!(
        (counts.total, counts.done, counts.overdue, counts.due_today),
        (6, 1, 2, 1)
    );
    let later = FIXED_TODAY.add_days(30).unwrap();
    let (shifted, _) = crate::data::seeded(later);
    assert_eq!(
        shifted.counts(selected, later),
        counts,
        "seeds follow the supplied date"
    );
    assert_eq!(
        first.counts(selected, later).overdue,
        4,
        "an old data set ages against a later date"
    );
}
