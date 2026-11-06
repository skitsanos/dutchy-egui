use dutchy_egui::FileInfo;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
};

pub type Batch = Vec<(PathBuf, Result<FileInfo, String>)>;
pub fn browse() -> Option<Vec<PathBuf>> {
    rfd::FileDialog::new()
        .set_title("Choose project files")
        .add_filter("Project files", &["pdf", "png", "jpg", "jpeg", "txt"])
        .pick_files()
}
pub fn inspect(ctx: egui::Context, paths: Vec<PathBuf>) -> Receiver<Batch> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let batch = paths
            .into_iter()
            .map(|path| {
                let metadata = std::fs::metadata(&path)
                    .map_err(|error| error.to_string())
                    .and_then(|metadata| {
                        if metadata.is_file() {
                            Ok(FileInfo::from_path(&path).size(metadata.len()))
                        } else {
                            Err("Choose a regular file".into())
                        }
                    });
                (path, metadata)
            })
            .collect();
        let _ = sender.send(batch);
        ctx.request_repaint();
    });
    receiver
}
