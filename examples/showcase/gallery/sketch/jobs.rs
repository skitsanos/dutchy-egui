use dutchy_egui::{ScribbleExportOptions, ScribbleSnapshot};
use egui::{ColorImage, Context};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
};
#[derive(Clone, Copy)]
pub enum Format {
    Svg,
    #[cfg(feature = "png-export")]
    Png,
}
impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            #[cfg(feature = "png-export")]
            Self::Png => "png",
        }
    }
}
pub enum Product {
    Preview { revision: u64, image: ColorImage },
    Saved { name: String, bytes: usize },
}
pub type Task = Receiver<Result<Product, String>>;
pub fn preview(ctx: Context, snapshot: ScribbleSnapshot, options: ScribbleExportOptions) -> Task {
    worker(ctx, move || {
        snapshot
            .rasterize(options)
            .map(|image| Product::Preview {
                revision: snapshot.revision(),
                image,
            })
            .map_err(|e| e.to_string())
    })
}
pub fn choose_path(format: Format) -> Option<PathBuf> {
    let ext = format.extension();
    rfd::FileDialog::new()
        .set_title("Export drawing")
        .set_file_name(format!("dutchy-scribble.{ext}"))
        .add_filter(format!("Drawing ({ext})"), &[ext])
        .save_file()
}
pub fn save(
    ctx: Context,
    snapshot: ScribbleSnapshot,
    options: ScribbleExportOptions,
    format: Format,
    path: PathBuf,
) -> Task {
    worker(ctx, move || {
        let bytes = match format {
            Format::Svg => snapshot.to_svg(options).map(String::into_bytes),
            #[cfg(feature = "png-export")]
            Format::Png => snapshot.to_png(options),
        }
        .map_err(|e| e.to_string())?;
        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        Ok(Product::Saved {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            bytes: bytes.len(),
        })
    })
}
fn worker(ctx: Context, job: impl FnOnce() -> Result<Product, String> + Send + 'static) -> Task {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(job());
        ctx.request_repaint();
    });
    receiver
}
