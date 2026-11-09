//! Opt-in hooks for native verification runs, read from the environment:
//!
//! - `DUTCHY_EXAMPLE_ZOOM=1.5` sets the display scale without touching the
//!   operating system's scaling.
//! - `DUTCHY_EXAMPLE_MODE=dark` (or `light`) picks the starting color mode.
//! - `DUTCHY_EXAMPLE_CAPTURE=/path/frame.png` saves the window once it has
//!   settled, then closes it. PNG needs the `png-export` feature; without it a
//!   binary PPM is written next to the requested path.
//!
//! Without these variables the hooks do nothing.

use dutchy_egui::Mode;
use egui::{ColorImage, Context, ViewportCommand};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
};

/// Frames to draw before the capture, so fonts and layout have settled.
const SETTLE_FRAMES: u32 = 12;

/// Display-scale override between 0.5 and 4.
pub fn zoom() -> Option<f32> {
    std::env::var("DUTCHY_EXAMPLE_ZOOM")
        .ok()?
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|zoom| (0.5..=4.0).contains(zoom))
}

/// Starting color mode.
pub fn mode() -> Option<Mode> {
    match std::env::var("DUTCHY_EXAMPLE_MODE").ok()?.trim() {
        "dark" => Some(Mode::Dark),
        "light" => Some(Mode::Light),
        _ => None,
    }
}

/// One pending window capture, or nothing.
#[derive(Default)]
pub struct Capture {
    path: Option<PathBuf>,
    frames: u32,
    pending: Option<Receiver<Arc<ColorImage>>>,
}

impl Capture {
    /// Read the capture path from the environment.
    pub fn from_env() -> Self {
        Self {
            path: std::env::var_os("DUTCHY_EXAMPLE_CAPTURE").map(PathBuf::from),
            ..Self::default()
        }
    }

    /// Call once per frame after drawing. Requests the screenshot after the
    /// settle frames, writes it when it arrives and closes the window.
    pub fn tick(&mut self, ctx: &Context) {
        let Some(path) = self.path.clone() else {
            return;
        };
        self.frames += 1;
        match &self.pending {
            Some(receiver) => {
                if let Ok(image) = receiver.try_recv() {
                    match write(&path, &image) {
                        Ok(written) => eprintln!("Captured {}", written.display()),
                        Err(error) => eprintln!("Capture failed: {error}"),
                    }
                    self.path = None;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                    return;
                }
            }
            None if self.frames >= SETTLE_FRAMES => {
                let (sender, receiver) = mpsc::channel();
                ctx.request_screenshot(move |image| {
                    let _ = sender.send(image);
                });
                self.pending = Some(receiver);
            }
            None => {}
        }
        ctx.request_repaint();
    }
}

#[cfg(feature = "png-export")]
fn write(path: &std::path::Path, image: &ColorImage) -> Result<PathBuf, String> {
    let [width, height] = image.size;
    let pixels = image.as_raw().to_vec();
    let buffer = image::RgbaImage::from_raw(width as u32, height as u32, pixels)
        .ok_or("pixel buffer size mismatch")?;
    buffer.save(path).map_err(|error| error.to_string())?;
    Ok(path.to_path_buf())
}

#[cfg(not(feature = "png-export"))]
fn write(path: &std::path::Path, image: &ColorImage) -> Result<PathBuf, String> {
    let path = path.with_extension("ppm");
    let [width, height] = image.size;
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    bytes.extend(
        image
            .pixels
            .iter()
            .flat_map(|pixel| [pixel.r(), pixel.g(), pixel.b()]),
    );
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
    Ok(path)
}
