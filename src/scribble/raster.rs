use super::{ScribbleError, ScribbleExportOptions, ScribbleSnapshot, export::Layout};
use egui::{Color32, ColorImage, Pos2, Rect, pos2};

const MAX_PIXELS: usize = 16_777_216;
const MAX_WORK: usize = 100_000_000;
struct Budget(usize);
impl Budget {
    fn spend(&mut self, amount: usize) -> Result<(), ScribbleError> {
        self.0 = self.0.saturating_add(amount);
        if self.0 > MAX_WORK {
            Err(ScribbleError::RasterTooComplex)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy)]
struct Bounds {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
}
impl Bounds {
    fn new(rect: Rect, size: [usize; 2]) -> Self {
        Self {
            left: rect.left().floor().clamp(0.0, size[0] as f32) as usize,
            top: rect.top().floor().clamp(0.0, size[1] as f32) as usize,
            right: rect.right().ceil().clamp(0.0, size[0] as f32) as usize,
            bottom: rect.bottom().ceil().clamp(0.0, size[1] as f32) as usize,
        }
    }
    fn width(self) -> usize {
        self.right.saturating_sub(self.left)
    }
    fn area(self) -> usize {
        self.width() * self.bottom.saturating_sub(self.top)
    }
}
pub(super) fn render(
    snapshot: &ScribbleSnapshot,
    options: ScribbleExportOptions,
) -> Result<ColorImage, ScribbleError> {
    let layout = Layout::new(snapshot, options)?;
    let pixels = layout.size[0] * layout.size[1];
    if pixels > MAX_PIXELS {
        return Err(ScribbleError::RasterTooLarge);
    }
    let prepared = prepare(snapshot, &layout, pixels)?;
    let mut image = ColorImage::filled(
        layout.size,
        options.background.unwrap_or(Color32::TRANSPARENT),
    );
    for stroke in prepared {
        let mut mask = vec![0.0f32; stroke.bounds.area()];
        if stroke.points.len() == 1 {
            segment(
                &mut mask,
                stroke.bounds,
                stroke.points[0],
                stroke.points[0],
                stroke.radius,
            );
        } else {
            for pair in stroke.points.windows(2) {
                segment(&mut mask, stroke.bounds, pair[0], pair[1], stroke.radius);
            }
        }
        composite(&mut image, &mask, stroke.bounds, stroke.color);
    }
    Ok(image)
}
fn distance(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let delta = b - a;
    let t = if delta.length_sq() > 0.0 {
        ((point - a).dot(delta) / delta.length_sq()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    point.distance(a + delta * t)
}
fn segment(mask: &mut [f32], bounds: Bounds, a: Pos2, b: Pos2, radius: f32) {
    let segment = segment_bounds(bounds, a, b, radius);
    for y in segment.top..segment.bottom {
        for x in segment.left..segment.right {
            let coverage = (radius + 0.5 - distance(pos2(x as f32 + 0.5, y as f32 + 0.5), a, b))
                .clamp(0.0, 1.0);
            let index = (y - bounds.top) * bounds.width() + x - bounds.left;
            // A connected semitransparent stroke is composited once at its joins.
            mask[index] = mask[index].max(coverage);
        }
    }
}
fn composite(image: &mut ColorImage, mask: &[f32], bounds: Bounds, color: Color32) {
    for y in bounds.top..bounds.bottom {
        for x in bounds.left..bounds.right {
            let coverage = mask[(y - bounds.top) * bounds.width() + x - bounds.left];
            if coverage > 0.0 {
                let index = y * image.size[0] + x;
                image.pixels[index] = blend(image.pixels[index], color, coverage);
            }
        }
    }
}
fn blend(destination: Color32, source: Color32, coverage: f32) -> Color32 {
    let s = source.to_srgba_unmultiplied();
    let d = destination.to_srgba_unmultiplied();
    let sa = f32::from(s[3]) / 255.0 * coverage;
    let da = f32::from(d[3]) / 255.0;
    let alpha = sa + da * (1.0 - sa);
    if alpha <= 0.0 {
        return Color32::TRANSPARENT;
    }
    let channel = |n: usize| {
        ((f32::from(s[n]) * sa + f32::from(d[n]) * da * (1.0 - sa)) / alpha).round() as u8
    };
    Color32::from_rgba_unmultiplied(
        channel(0),
        channel(1),
        channel(2),
        (alpha * 255.0).round() as u8,
    )
}

struct RasterStroke {
    points: Vec<Pos2>,
    bounds: Bounds,
    radius: f32,
    color: Color32,
}
fn segment_bounds(bounds: Bounds, a: Pos2, b: Pos2, radius: f32) -> Bounds {
    let rect = Rect::from_two_pos(a, b).expand(radius + 1.0);
    let clip = Rect::from_min_max(
        pos2(bounds.left as f32, bounds.top as f32),
        pos2(bounds.right as f32, bounds.bottom as f32),
    );
    Bounds::new(rect.intersect(clip), [bounds.right, bounds.bottom])
}
fn prepare(
    snapshot: &ScribbleSnapshot,
    layout: &Layout,
    pixels: usize,
) -> Result<Vec<RasterStroke>, ScribbleError> {
    let mut budget = Budget(pixels);
    let mut prepared = Vec::new();
    for stroke in &snapshot.strokes {
        let points: Vec<_> = stroke.points().iter().map(|p| layout.point(*p)).collect();
        let radius = stroke.brush().pen_width().points() * layout.scale / 2.0;
        let bounds = Bounds::new(Rect::from_points(&points).expand(radius + 1.0), layout.size);
        budget.spend(bounds.area().saturating_mul(2))?;
        if points.len() == 1 {
            budget.spend(segment_bounds(bounds, points[0], points[0], radius).area())?;
        } else {
            for pair in points.windows(2) {
                budget.spend(segment_bounds(bounds, pair[0], pair[1], radius).area())?;
            }
        }
        prepared.push(RasterStroke {
            points,
            bounds,
            radius,
            color: stroke.brush().ink(),
        });
    }
    Ok(prepared)
}
