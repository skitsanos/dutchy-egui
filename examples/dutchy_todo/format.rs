//! Plain-text exchange format for import and export.
//!
//! One record per line, tab separated, so no serialization crate is needed:
//!
//! ```text
//! # dutchy-todo 1
//! project<TAB>id<TAB>name
//! task<TAB>id<TAB>project<TAB>done<TAB>priority<TAB>due<TAB>title<TAB>notes
//! ```
//!
//! `done` is `0` or `1`, `priority` is `low`, `medium` or `high`, `due` is an
//! ISO date or `-`. Tabs, newlines and backslashes inside text are escaped as
//! `\t`, `\n`, `\r` and `\\`. Blank lines and `#` comments are ignored.

use crate::store::{Priority, Project, ProjectId, Store, Task, TaskId};
use dutchy_egui::calendar::Date;
use std::fmt;

/// First line of every file.
pub const HEADER: &str = "# dutchy-todo 1";

/// Why a file could not be imported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportError {
    /// One-based line number, zero for a missing header.
    pub line: usize,
    /// What was wrong.
    pub message: String,
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "Line {}: {}", self.line, self.message)
        }
    }
}

/// Serialize every project and task.
pub fn export(store: &Store) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for project in store.projects() {
        out.push_str(&format!(
            "project\t{}\t{}\n",
            project.id.0,
            escape(&project.name)
        ));
    }
    for task in store.tasks() {
        out.push_str(&format!(
            "task\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            task.id.0,
            task.project.0,
            u8::from(task.done),
            task.priority.key(),
            task.due
                .map_or_else(|| "-".to_owned(), |date| date.to_string()),
            escape(&task.title),
            escape(&task.notes)
        ));
    }
    out
}

/// Parse a file written by [`export`].
pub fn import(text: &str) -> Result<Store, ImportError> {
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line));
    match lines.next() {
        Some((_, header)) if header.trim_end() == HEADER => {}
        _ => return Err(fail(0, format!("Expected the header `{HEADER}`"))),
    }
    let mut store = Store::new();
    for (number, line) in lines {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields[0] {
            "project" => store.insert_project(project(number, &fields)?),
            "task" => {
                let task = task(number, &fields)?;
                if store.project(task.project).is_none() {
                    return Err(fail(number, format!("Unknown project {}", task.project.0)));
                }
                store.insert_task(task);
            }
            other => return Err(fail(number, format!("Unknown record `{other}`"))),
        }
    }
    Ok(store)
}

fn project(number: usize, fields: &[&str]) -> Result<Project, ImportError> {
    if fields.len() != 3 {
        return Err(fail(number, "A project record needs an id and a name"));
    }
    Ok(Project {
        id: ProjectId(id(number, fields[1])?),
        name: unescape(number, fields[2])?,
    })
}

fn task(number: usize, fields: &[&str]) -> Result<Task, ImportError> {
    if fields.len() != 8 {
        return Err(fail(number, "A task record needs eight fields"));
    }
    let done = match fields[3] {
        "0" => false,
        "1" => true,
        other => return Err(fail(number, format!("Done must be 0 or 1, not `{other}`"))),
    };
    let priority = Priority::parse(fields[4])
        .ok_or_else(|| fail(number, format!("Unknown priority `{}`", fields[4])))?;
    let due = match fields[5] {
        "-" => None,
        text => {
            Some(Date::parse(text).ok_or_else(|| fail(number, format!("Invalid date `{text}`")))?)
        }
    };
    Ok(Task {
        id: TaskId(id(number, fields[1])?),
        project: ProjectId(id(number, fields[2])?),
        title: unescape(number, fields[6])?,
        notes: unescape(number, fields[7])?,
        due,
        priority,
        done,
    })
}

fn id(number: usize, text: &str) -> Result<u64, ImportError> {
    text.parse()
        .map_err(|_| fail(number, format!("Invalid id `{text}`")))
}

fn fail(line: usize, message: impl Into<String>) -> ImportError {
    ImportError {
        line,
        message: message.into(),
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}

fn unescape(number: usize, text: &str) -> Result<String, ImportError> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            other => {
                let shown = other.map_or_else(String::new, String::from);
                return Err(fail(number, format!("Unknown escape `\\{shown}`")));
            }
        }
    }
    Ok(out)
}
