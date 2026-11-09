//! Import and export through the host file dialog. The dialog runs on the UI
//! thread like the showcase's; reading and writing happen on a worker that
//! posts one outcome through a channel and requests a repaint.

use crate::{format, store::Store};
use egui::Context;
use std::{
    path::Path,
    sync::mpsc::{self, Receiver, TryRecvError},
};

/// Result of a finished job.
pub enum Outcome {
    /// The file was written.
    Exported {
        /// File name shown in the notification.
        name: String,
        /// Number of tasks written.
        tasks: usize,
    },
    /// The file was parsed.
    Imported {
        /// File name shown in the notification.
        name: String,
        /// Parsed records, replacing the current store.
        store: Store,
    },
    /// Reading, writing or parsing failed.
    Failed(String),
}

/// At most one job runs at a time.
#[derive(Default)]
pub struct Io {
    pending: Option<Receiver<Outcome>>,
}

impl Io {
    /// Ask for a destination and write the export text there.
    pub fn export(&mut self, ctx: &Context, text: String, tasks: usize) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Export tasks")
            .set_file_name("dutchy-todo.txt")
            .add_filter("Dutchy tasks (txt)", &["txt"])
            .save_file()
        else {
            return;
        };
        self.spawn(ctx, move || {
            std::fs::write(&path, text)
                .map(|()| Outcome::Exported {
                    name: file_name(&path),
                    tasks,
                })
                .unwrap_or_else(|error| Outcome::Failed(error.to_string()))
        });
    }

    /// Ask for a file and parse it.
    pub fn import(&mut self, ctx: &Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Import tasks")
            .add_filter("Dutchy tasks (txt)", &["txt"])
            .pick_file()
        else {
            return;
        };
        self.spawn(ctx, move || {
            let text = match std::fs::read_to_string(&path) {
                Ok(text) => text,
                Err(error) => return Outcome::Failed(error.to_string()),
            };
            match format::import(&text) {
                Ok(store) => Outcome::Imported {
                    name: file_name(&path),
                    store,
                },
                Err(error) => Outcome::Failed(error.to_string()),
            }
        });
    }

    /// Whether a job is still running.
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    /// Take a finished outcome, if any.
    pub fn poll(&mut self) -> Option<Outcome> {
        let outcome = match self.pending.as_ref()?.try_recv() {
            Ok(outcome) => outcome,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Outcome::Failed("The file operation ended without a result.".into())
            }
        };
        self.pending = None;
        Some(outcome)
    }

    fn spawn(
        &mut self,
        ctx: &Context,
        job: impl FnOnce() -> Outcome + Send + std::panic::UnwindSafe + 'static,
    ) {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(job).unwrap_or_else(|_| {
                Outcome::Failed("The file operation failed unexpectedly.".into())
            });
            let _ = sender.send(outcome);
            ctx.request_repaint();
        });
        self.pending = Some(receiver);
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn disconnected_jobs_report_failure_and_release_busy_state() {
        let (sender, receiver) = mpsc::channel();
        let mut io = Io {
            pending: Some(receiver),
        };
        drop(sender);
        assert!(matches!(io.poll(), Some(Outcome::Failed(_))));
        assert!(!io.busy());
        assert!(io.poll().is_none());
    }

    fn wait(io: &mut Io) -> Outcome {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(outcome) = io.poll() {
                return outcome;
            }
            assert!(Instant::now() < deadline, "file worker failed to complete");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn a_panicking_worker_reports_failure_and_a_later_job_succeeds() {
        let ctx = Context::default();
        let mut io = Io::default();
        io.spawn(&ctx, || panic!("simulated worker failure"));
        assert!(matches!(wait(&mut io), Outcome::Failed(_)));
        assert!(!io.busy());
        io.spawn(&ctx, || Outcome::Imported {
            name: "valid.txt".into(),
            store: Store::new(),
        });
        assert!(matches!(wait(&mut io), Outcome::Imported { .. }));
        assert!(!io.busy());
    }
}
