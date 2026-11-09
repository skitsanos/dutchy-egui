//! The plain-text exchange format round-trips and rejects malformed input.

use crate::format::{HEADER, export, import};
use crate::store::{Priority, Store, TaskDraft};
use crate::support::data::FIXED_TODAY;

fn sample() -> Store {
    let mut store = Store::new();
    let garden = store.add_project("Garden\trenovation");
    let kitchen = store.add_project("Kitchen");
    let id = store.add_task(
        garden,
        TaskDraft {
            title: "Order seed trays".into(),
            notes: "Line one\nLine \"two\" with \\ backslash".into(),
            due: Some(FIXED_TODAY),
            priority: Priority::High,
        },
    );
    store.set_done(id, true);
    store.add_task(
        kitchen,
        TaskDraft {
            title: "Measure the alcove".into(),
            notes: String::new(),
            due: None,
            priority: Priority::Low,
        },
    );
    store
}

#[test]
fn export_is_readable_and_round_trips() {
    let store = sample();
    let text = export(&store);
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines[0], HEADER);
    assert_eq!(lines[1], "project\t0\tGarden\\trenovation");
    assert_eq!(
        lines[3],
        "task\t0\t0\t1\thigh\t2026-11-08\tOrder seed trays\tLine one\\nLine \"two\" with \\\\ backslash"
    );
    assert_eq!(lines[4], "task\t1\t1\t0\tlow\t-\tMeasure the alcove\t");
    let imported = import(&text).unwrap();
    assert_eq!(imported, store);
    assert_eq!(export(&imported), text);
}

#[test]
fn imported_stores_continue_numbering_after_the_highest_id() {
    let mut imported = import(&export(&sample())).unwrap();
    let project = imported.add_project("Reading");
    assert_eq!(project.0, 2);
    let task = imported.add_task(project, TaskDraft::default());
    assert_eq!(task.0, 2);
}

#[test]
fn import_accepts_comments_blank_lines_and_windows_line_endings() {
    let text = format!(
        "{HEADER}\r\n\r\n# a comment\r\nproject\t7\tGarden\r\ntask\t3\t7\t0\tmedium\t-\tDig\t\r\n"
    );
    let store = import(&text).unwrap();
    assert_eq!(store.projects()[0].id.0, 7);
    assert_eq!(store.tasks().next().unwrap().title, "Dig");
}

#[test]
fn import_reports_the_failing_line() {
    let error = import("hello").unwrap_err();
    assert_eq!(error.line, 0);
    assert_eq!(error.to_string(), format!("Expected the header `{HEADER}`"));
    let cases = [
        ("project\tx\tName", "Invalid id `x`"),
        ("project\t1", "A project record needs an id and a name"),
        ("task\t1\t9\t0\tlow\t-\tTitle\t", "Unknown project 9"),
        ("note\t1", "Unknown record `note`"),
    ];
    for (line, message) in cases {
        let error = import(&format!("{HEADER}\n{line}\n")).unwrap_err();
        assert_eq!(error.line, 2, "{line}");
        assert_eq!(error.message, message, "{line}");
    }
    let base = format!("{HEADER}\nproject\t1\tGarden\n");
    let task_cases = [
        (
            "task\t1\t1\t2\tlow\t-\tTitle\t",
            "Done must be 0 or 1, not `2`",
        ),
        (
            "task\t1\t1\t0\turgent\t-\tTitle\t",
            "Unknown priority `urgent`",
        ),
        (
            "task\t1\t1\t0\tlow\t2026-13-01\tTitle\t",
            "Invalid date `2026-13-01`",
        ),
        (
            "task\t1\t1\t0\tlow\t-\tBad \\q escape\t",
            "Unknown escape `\\q`",
        ),
        (
            "task\t1\t1\t0\tlow\t-\tTitle",
            "A task record needs eight fields",
        ),
    ];
    for (line, message) in task_cases {
        let error = import(&format!("{base}{line}\n")).unwrap_err();
        assert_eq!(error.to_string(), format!("Line 3: {message}"), "{line}");
    }
}
