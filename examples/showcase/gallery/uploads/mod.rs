mod native;
use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, Checkbox, ControlSize, FileInfo, FileItem,
    FileList, FileRules, FileStatus, FileUpload, FileUploadVariant, Spinner, Text, TextColor,
};
use egui::{Context, Ui};
use std::{path::PathBuf, sync::mpsc::Receiver};

struct Selected {
    key: u64,
    info: FileInfo,
    path: Option<PathBuf>,
    status: FileStatus,
    started: Option<f64>,
    fail_once: bool,
}
#[derive(Default)]
pub struct UploadState {
    files: Vec<Selected>,
    next: u64,
    pending: Option<Receiver<native::Batch>>,
    pub disabled: bool,
    pub fail_next: bool,
    messages: Vec<String>,
}
fn rules() -> FileRules {
    FileRules::new()
        .extensions(["pdf", "png", "jpg", "jpeg", "txt"])
        .max_bytes(10_000_000)
        .max_files(5)
}
impl UploadState {
    pub fn tick(&mut self, ctx: &Context) {
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(batch) => {
                    self.pending = None;
                    let mut candidates = Vec::new();
                    for (path, result) in batch {
                        match result {
                            Ok(info) => candidates.push((Some(path), info)),
                            Err(error) => self
                                .messages
                                .push(format!("{}: {error}", FileInfo::from_path(&path).name)),
                        }
                    }
                    self.add(candidates);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.messages
                        .push("File inspection did not finish. Choose the files again.".into());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        let now = ctx.input(|input| input.time);
        let mut running = false;
        for file in &mut self.files {
            let Some(started) = file.started else {
                continue;
            };
            let progress = ((now - started).max(0.0) / 3.0 * 100.0).min(100.0);
            if file.fail_once && progress >= 45.0 {
                file.fail_once = false;
                file.started = None;
                file.status = FileStatus::Failed(
                    "The demo connection was interrupted. Retry to continue.".into(),
                );
            } else if progress >= 100.0 {
                file.started = None;
                file.status = FileStatus::Complete;
            } else {
                file.status = FileStatus::Uploading(Some(progress));
                running = true;
            }
        }
        if running {
            ctx.request_repaint_after(dutchy_egui::tokens::motion::FRAME_INTERVAL);
        }
    }
    fn add(&mut self, candidates: Vec<(Option<PathBuf>, FileInfo)>) {
        let mut unique: Vec<(Option<PathBuf>, FileInfo)> = Vec::new();
        for (path, info) in candidates {
            let duplicate = path.as_ref().is_some_and(|path| {
                self.files
                    .iter()
                    .any(|file| file.path.as_ref() == Some(path))
                    || unique.iter().any(|(other, _)| other.as_ref() == Some(path))
            });
            if duplicate {
                self.messages
                    .push(format!("{} is already selected.", info.name));
            } else {
                unique.push((path, info));
            }
        }
        let metadata: Vec<_> = unique.iter().map(|(_, info)| info.clone()).collect();
        let result = rules().validate(&metadata, self.files.len());
        for rejection in result.rejected {
            self.messages.push(format!(
                "{}: {}",
                metadata[rejection.index].name, rejection.reason
            ));
        }
        for index in result.accepted {
            let (path, info) = unique[index].clone();
            self.files.push(Selected {
                key: self.next,
                info,
                path,
                status: FileStatus::Ready,
                started: None,
                fail_once: false,
            });
            self.next += 1;
        }
    }
    fn select(&mut self, ui: &Ui, result: dutchy_egui::FileUploadResponse) -> egui::Id {
        let id = result.response.id;
        let paths = if result.browse_requested {
            let paths = native::browse().unwrap_or_default();
            ui.memory_mut(|memory| memory.request_focus(id));
            paths
        } else {
            result
                .dropped
                .iter()
                .map(|file| file.path().to_owned())
                .collect()
        };
        if !paths.is_empty() {
            self.pending = Some(native::inspect(ui.ctx().clone(), paths));
        }
        id
    }
    pub fn show(&mut self, ui: &mut Ui) {
        ui.add(Text::heading("Choose. Drop. Keep control."));
        ui.add(Text::new(
            "Files stay on this device. Upload progress is simulated; no files are sent anywhere.",
        ));
        ui.add(Checkbox::new(
            "disable-files",
            &mut self.disabled,
            "Disable file controls",
        ));
        let enabled = !self.disabled && self.pending.is_none();
        let mut return_focus = None;
        section(ui, "01", "Drop zone / native file picker", |ui| {
            let result = FileUpload::new("project-files")
                .label("Drop project files")
                .hint("PDF, PNG, JPEG or TXT · up to 10 MB each · 5 files maximum")
                .enabled(enabled)
                .show(ui);
            return_focus = Some(self.select(ui, result));
        });
        section(
            ui,
            "02",
            "Compact selection / same validation policy",
            |ui| {
                let result = FileUpload::new("compact-files")
                    .variant(FileUploadVariant::Compact)
                    .label("Choose project files")
                    .enabled(enabled)
                    .show(ui);
                self.select(ui, result);
            },
        );
        section(ui, "03", "Host-owned files / transfer simulation", |ui| {
            if self.pending.is_some() {
                ui.horizontal(|ui| {
                    ui.add(Spinner::new());
                    ui.add(Text::new("Inspecting selected files…"));
                });
            }
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add(
                        Button::new("Add sample files")
                            .variant(ButtonVariant::Outline)
                            .size(ControlSize::Small)
                            .enabled(enabled),
                    )
                    .clicked()
                {
                    self.add(vec![
                        (None, FileInfo::new("project-brief.pdf").size(2_400_000)),
                        (None, FileInfo::new("canal-photo.jpg").size(840_000)),
                    ]);
                }
                let ready = self
                    .files
                    .iter()
                    .any(|file| file.status == FileStatus::Ready);
                if ui
                    .add(
                        Button::new("Start demo upload")
                            .size(ControlSize::Small)
                            .enabled(!self.disabled && ready),
                    )
                    .clicked()
                {
                    let now = ui.input(|input| input.time);
                    for file in &mut self.files {
                        if file.status == FileStatus::Ready {
                            file.started = Some(now);
                            file.status = FileStatus::Uploading(Some(0.0));
                            file.fail_once = self.fail_next;
                            self.fail_next = false;
                        }
                    }
                    ui.ctx().request_repaint();
                }
            });
            ui.add(Checkbox::new(
                "fail-file",
                &mut self.fail_next,
                "Interrupt the next demo transfer once",
            ));
            if !self.messages.is_empty() {
                let response = Alert::new("file-feedback", self.messages.join("\n"))
                    .title("Some files were not added")
                    .variant(AlertVariant::Warning)
                    .dismissible(true)
                    .show(ui);
                if response.dismissed {
                    self.messages.clear();
                }
            }
            ui.add(
                Text::label(format!("{} / 5 files selected", self.files.len()))
                    .color(TextColor::Muted),
            );
            let items: Vec<_> = self
                .files
                .iter()
                .map(|file| {
                    let mut item = FileItem::new(file.key, file.info.name.as_str())
                        .status(file.status.clone());
                    item.bytes = file.info.bytes;
                    item
                })
                .collect();
            let mut list = FileList::new("project-list", &items).enabled(!self.disabled);
            if let Some(id) = return_focus {
                list = list.empty_focus(id);
            }
            let result = list.show(ui);
            if let Some(key) = result.remove_requested.or(result.cancel_requested) {
                self.files.retain(|file| file.key != key);
            }
            if let Some(key) = result.retry_requested
                && let Some(file) = self.files.iter_mut().find(|file| file.key == key)
            {
                file.started = Some(ui.input(|input| input.time));
                file.status = FileStatus::Uploading(Some(0.0));
                ui.ctx().request_repaint();
            }
        });
    }
}
