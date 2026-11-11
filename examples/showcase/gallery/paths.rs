use dutchy_egui::validation::ValidationState;
use dutchy_egui::{Field, PathBrowseRequest, PathInput, Text, TextColor, TextSize, tokens::space};
use egui::Ui;
use std::path::PathBuf;

#[derive(Default)]
pub struct PathsState {
    file: PathField,
    folder: PathField,
    save: PathField,
}

#[derive(Default)]
struct PathField {
    value: String,
    validation: ValidationState,
}

pub fn show(ui: &mut Ui, state: &mut PathsState) {
    ui.add(Text::heading("Path input").size(TextSize::Xl5));
    ui.add(Text::new("Type a path or choose one in a native dialog.").color(TextColor::Muted));
    ui.add_space(space::LG);
    show_field(
        ui,
        &mut state.file,
        "Project file",
        PathBrowseRequest::OpenFile,
    );
    ui.add_space(space::LG);
    show_field(
        ui,
        &mut state.folder,
        "Workspace folder",
        PathBrowseRequest::OpenFolder,
    );
    ui.add_space(space::LG);
    show_field(
        ui,
        &mut state.save,
        "Export destination",
        PathBrowseRequest::SaveFile,
    );
}

fn show_field(ui: &mut Ui, field: &mut PathField, label: &str, mode: PathBrowseRequest) {
    let mut pending = None;
    let editor = Field::new(label)
        .validation(&field.validation)
        .show(ui, |ui| {
            let result = PathInput::new("path", &mut field.value, mode)
                .accessible_name(label)
                .placeholder("Enter a path…")
                .show(ui);
            pending = result.request.map(|request| (request, result.browse));
            result.response
        });
    if editor.changed() {
        field.validation = ValidationState::Unchecked;
    }
    if let Some((request, browse)) = pending {
        if let Some(path) = choose(request) {
            match path.into_os_string().into_string() {
                Ok(path) => {
                    field.value = path;
                    field.validation = ValidationState::Unchecked;
                }
                Err(_) => {
                    field.validation =
                        ValidationState::Invalid("Choose a path with a Unicode name.".into());
                }
            }
        }
        browse.request_focus();
        ui.ctx().request_repaint();
    }
}

fn choose(request: PathBrowseRequest) -> Option<PathBuf> {
    let dialog = rfd::FileDialog::new();
    match request {
        PathBrowseRequest::OpenFile => dialog.set_title("Open project file").pick_file(),
        PathBrowseRequest::OpenFolder => dialog.set_title("Choose workspace folder").pick_folder(),
        PathBrowseRequest::SaveFile => dialog
            .set_title("Choose export destination")
            .set_file_name("export.txt")
            .save_file(),
        _ => None,
    }
}
