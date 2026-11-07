use super::{ScribbleError, ScribbleExportOptions, ScribbleSnapshot};
use egui::{Color32, Pos2, Vec2, pos2, vec2};
use std::fmt::Write;

pub(super) struct Layout {
    pub size: [usize; 2],
    pub scale: f32,
    pub offset: Vec2,
}
impl Layout {
    pub fn new(
        snapshot: &ScribbleSnapshot,
        options: ScribbleExportOptions,
    ) -> Result<Self, ScribbleError> {
        let width = (snapshot.size[0] as f64 * f64::from(options.scale)).round();
        let height = (snapshot.size[1] as f64 * f64::from(options.scale)).round();
        if !width.is_finite()
            || !height.is_finite()
            || width.min(height) < 1.0
            || width.max(height) > 8192.0
        {
            return Err(ScribbleError::InvalidExportSize);
        }
        let output = vec2(width as f32, height as f32);
        let drawing = vec2(snapshot.size[0] as f32, snapshot.size[1] as f32);
        let scale = (output / drawing).min_elem();
        Ok(Self {
            size: [width as usize, height as usize],
            scale,
            offset: (output - drawing * scale) / 2.0,
        })
    }
    pub fn point(&self, point: Pos2) -> Pos2 {
        pos2(
            point.x * self.scale + self.offset.x,
            point.y * self.scale + self.offset.y,
        )
    }
}
fn color(color: Color32) -> (String, f32) {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    (format!("#{r:02x}{g:02x}{b:02x}"), f32::from(a) / 255.0)
}
impl ScribbleSnapshot {
    /// Export static, self-contained SVG without scripts, links, UI borders or cursor.
    /// The background is transparent by default. Dimensions are bounded; SVG does
    /// not have the raster pixel/work limits. Performs no filesystem/network I/O.
    pub fn to_svg(&self, options: ScribbleExportOptions) -> Result<String, ScribbleError> {
        let layout = Layout::new(self, options)?;
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" overflow=\"hidden\" role=\"img\"><title>Scribble drawing</title>\n",
            layout.size[0], layout.size[1], layout.size[0], layout.size[1]
        );
        if let Some(background) = options.background {
            let (color, alpha) = color(background);
            let _ = writeln!(
                svg,
                "<rect width=\"{}\" height=\"{}\" fill=\"{color}\" fill-opacity=\"{alpha:.4}\"/>",
                layout.size[0], layout.size[1]
            );
        }
        for stroke in &self.strokes {
            let (color, alpha) = color(stroke.brush().ink());
            let width = stroke.brush().pen_width().points() * layout.scale;
            if stroke.points().len() == 1 {
                let p = layout.point(stroke.points()[0]);
                let _ = writeln!(
                    svg,
                    "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{:.3}\" fill=\"{color}\" fill-opacity=\"{alpha:.4}\"/>",
                    p.x,
                    p.y,
                    width / 2.0
                );
            } else {
                svg.push_str("<polyline points=\"");
                for p in stroke.points() {
                    let p = layout.point(*p);
                    let _ = write!(svg, "{:.3},{:.3} ", p.x, p.y);
                }
                let _ = writeln!(
                    svg,
                    "\" fill=\"none\" stroke=\"{color}\" stroke-opacity=\"{alpha:.4}\" stroke-width=\"{width}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
                );
            }
        }
        svg.push_str("</svg>\n");
        Ok(svg)
    }
    /// CPU raster export with analytic edge coverage and round caps/joins.
    /// Produces an egui image, independent of display scale or a GPU screenshot.
    /// Rejects more than 16,777,216 pixels or 100 million estimated sampling units.
    /// Hosts should run large exports on a worker.
    pub fn rasterize(
        &self,
        options: ScribbleExportOptions,
    ) -> Result<egui::ColorImage, ScribbleError> {
        super::raster::render(self, options)
    }
    /// Encode the same bounded CPU raster as an RGBA PNG (`png-export` feature).
    /// Alpha is unpremultiplied for PNG; no file is written by the library.
    #[cfg(feature = "png-export")]
    pub fn to_png(&self, options: ScribbleExportOptions) -> Result<Vec<u8>, ScribbleError> {
        use image::ImageEncoder as _;
        let image = self.rasterize(options)?;
        let rgba: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(Color32::to_srgba_unmultiplied)
            .collect();
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(
                &rgba,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|error| ScribbleError::Encoding(error.to_string()))?;
        Ok(png)
    }
}
