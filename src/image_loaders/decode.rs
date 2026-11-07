use super::UrlImageOptions;
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

/// Decode PNG/JPEG bytes within the option limits. A codec panic becomes a
/// failure (native targets only: wasm aborts on panic) so retry stays offered.
pub(super) fn decode(
    bytes: &[u8],
    options: UrlImageOptions,
    max_side: u32,
) -> Result<egui::ColorImage, String> {
    guarded(|| decode_image(bytes, options, max_side))
}

fn guarded<T>(decode: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(decode))
        .unwrap_or_else(|_| Err("Image decoder failed".into()))
}

fn decode_image(
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

#[cfg(test)]
mod tests {
    use super::guarded;
    #[test]
    fn a_panicking_decoder_reports_a_failure_instead_of_unwinding() {
        let result: Result<u8, String> = guarded(|| panic!("codec bug"));
        assert_eq!(result.unwrap_err(), "Image decoder failed");
    }
    #[test]
    fn a_completed_decode_passes_through_the_guard() {
        assert_eq!(guarded(|| Ok::<u8, String>(7)), Ok(7));
        assert_eq!(
            guarded(|| Err::<u8, String>("bad header".into())),
            Err("bad header".to_string())
        );
    }
}
