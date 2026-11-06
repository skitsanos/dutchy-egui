use super::UrlImageOptions;
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

pub(super) fn decode(
    bytes: &[u8],
    options: UrlImageOptions,
    max_side: u32,
) -> Result<egui::ColorImage, String> {
    let format = image::guess_format(bytes).map_err(|error| error.to_string())?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err("URL images support PNG and JPEG".into());
    }
    let reader = || {
        let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(max_side);
        limits.max_image_height = Some(max_side);
        limits.max_alloc = Some(options.max_pixels.saturating_mul(16));
        reader.limits(limits);
        reader
    };
    let (width, height) = reader()
        .into_dimensions()
        .map_err(|error| error.to_string())?;
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > options.max_pixels {
        return Err("Image exceeds the decoded pixel limit".into());
    }
    let rgba = reader()
        .decode()
        .map_err(|error| error.to_string())?
        .into_rgba8();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &rgba,
    ))
}
