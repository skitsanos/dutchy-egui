//! Pointer-stroke smoothing: a pulled-string stabilizer, a midpoint quadratic
//! B-spline fit and chunked Ramer–Douglas–Peucker simplification.
//! Pure and I/O-free; the widget and [`ScribbleSmoothing::smooth`] share every step.
use super::raster::distance;
use crate::tokens::scribble as tokens;
use egui::{Pos2, Rect};

/// Upper bound on flattened segments per spline piece, bounding fit output.
const MAX_PIECE_SEGMENTS: usize = 16;
/// Simplification window; bounds the worst-case cost of one commit.
const SIMPLIFY_CHUNK: usize = 256;

/// Tokenized pointer-stroke smoothing. Keyboard strokes are never smoothed.
///
/// Smoothing is applied once, when a gesture is committed, so stored strokes,
/// history and exports remain ordinary polylines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScribbleSmoothing {
    /// Store pointer samples as captured.
    #[default]
    Off,
    /// Short stabilizer string; removes fine tremor with little lag.
    Light,
    /// Balanced stabilization for handwriting and signatures.
    Regular,
    /// Long stabilizer string for slow, deliberate curves.
    Strong,
}
impl ScribbleSmoothing {
    /// Stabilizer string length in drawing units; zero when off.
    pub const fn radius(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Light => tokens::SMOOTH_LIGHT,
            Self::Regular => tokens::SMOOTH_REGULAR,
            Self::Strong => tokens::SMOOTH_STRONG,
        }
    }
    /// Whether this level changes captured samples.
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::Off)
    }
    /// Smooth one gesture's drawing-coordinate samples exactly as the widget
    /// commits a pointer stroke: stabilize, end at the last sample, fit a curve
    /// and simplify. `Off` returns the samples unchanged.
    ///
    /// The first and last samples are kept, and every output point stays inside
    /// the samples' bounding box, so in-bounds input remains in bounds. The
    /// widget's per-stroke point caps are not applied here.
    pub fn smooth(self, samples: &[Pos2]) -> Vec<Pos2> {
        let Some((&first, rest)) = samples.split_first() else {
            return Vec::new();
        };
        let Some(mut stabilizer) = Stabilizer::new(self, first) else {
            return samples.to_vec();
        };
        let mut points = vec![first];
        for &sample in rest {
            if let Some(pen) = stabilizer.advance(sample, points[points.len() - 1]) {
                points.push(pen);
            }
        }
        simplify(&fit(&stabilizer.trail(&points)))
    }
}

/// Pulled-string stabilizer: the pen trails the pointer on a fixed-length string
/// and moves only when the pointer pulls it taut. Distance-based, so results do
/// not depend on event rate or frame timing.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Stabilizer {
    radius: f32,
    pen: Pos2,
    tip: Pos2,
}
impl Stabilizer {
    /// A stabilizer for an enabled level, starting at the gesture's first sample.
    pub(crate) fn new(smoothing: ScribbleSmoothing, start: Pos2) -> Option<Self> {
        smoothing.is_enabled().then_some(Self {
            radius: smoothing.radius(),
            pen: start,
            tip: start,
        })
    }
    /// Record a raw sample. Returns the pen position to store once it is at least
    /// one sample distance from `last`, the most recently stored point.
    pub(crate) fn advance(&mut self, sample: Pos2, last: Pos2) -> Option<Pos2> {
        self.tip = sample;
        let delta = sample - self.pen;
        let length = delta.length();
        if length > self.radius {
            self.pen += delta * ((length - self.radius) / length);
        }
        (last.distance(self.pen) >= tokens::SAMPLE_DISTANCE).then_some(self.pen)
    }
    /// Stored points followed by the latest raw sample, so ink ends under the pointer.
    pub(crate) fn trail(&self, points: &[Pos2]) -> Vec<Pos2> {
        let mut trail = points.to_vec();
        if trail
            .last()
            .is_none_or(|last| last.distance(self.tip) >= tokens::SAMPLE_DISTANCE)
        {
            trail.push(self.tip);
        }
        trail
    }
}

/// Fit a midpoint quadratic B-spline through the sample polygon and flatten it
/// within the shared tolerance. Endpoints are preserved; the curve never
/// overshoots its control polygon.
pub(crate) fn fit(points: &[Pos2]) -> Vec<Pos2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let bounds = Rect::from_points(points);
    let last = points.len() - 1;
    let mut fitted = vec![points[0]];
    let mut start = points[0];
    for index in 1..last {
        let control = points[index];
        let end = if index + 1 == last {
            points[last]
        } else {
            control.lerp(points[index + 1], 0.5)
        };
        flatten(&mut fitted, [start, control, end], bounds);
        start = end;
    }
    fitted
}
/// Append one quadratic piece, excluding its start, as uniform chords. The chord
/// error of `n` pieces is |p0 - 2p1 + p2| / (4n²).
fn flatten(out: &mut Vec<Pos2>, [start, control, end]: [Pos2; 3], bounds: Rect) {
    let bend = (start.to_vec2() - 2.0 * control.to_vec2() + end.to_vec2()).length();
    let segments = ((bend / (4.0 * tokens::SMOOTH_TOLERANCE)).sqrt().ceil() as usize)
        .clamp(1, MAX_PIECE_SEGMENTS);
    for step in 1..segments {
        let t = step as f32 / segments as f32;
        let point = start.lerp(control, t).lerp(control.lerp(end, t), t);
        // Rounding must never move ink outside the samples' bounds.
        out.push(bounds.clamp(point));
    }
    out.push(end);
}

/// Drop points within the shared tolerance of the simplified polyline, in
/// bounded windows so a commit's cost stays linear in the point count.
pub(crate) fn simplify(points: &[Pos2]) -> Vec<Pos2> {
    let Some(&first) = points.first() else {
        return Vec::new();
    };
    let mut simplified = vec![first];
    let mut start = 0;
    while start + 1 < points.len() {
        let end = (start + SIMPLIFY_CHUNK).min(points.len() - 1);
        reduce(&points[start..=end], &mut simplified);
        start = end;
    }
    simplified
}
/// Ramer–Douglas–Peucker on one window; appends kept points after the first.
fn reduce(points: &[Pos2], out: &mut Vec<Pos2>) {
    let last = points.len() - 1;
    let mut keep = vec![false; points.len()];
    keep[last] = true;
    let mut spans = vec![(0, last)];
    while let Some((first, last)) = spans.pop() {
        let farthest = (first + 1..last)
            .map(|index| {
                let offset = distance(points[index], points[first], points[last]);
                (index, offset)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((index, offset)) = farthest
            && offset > tokens::SMOOTH_TOLERANCE
        {
            keep[index] = true;
            spans.push((first, index));
            spans.push((index, last));
        }
    }
    out.extend(
        points
            .iter()
            .zip(keep)
            .skip(1)
            .filter_map(|(point, keep)| keep.then_some(*point)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn the_pen_moves_only_when_the_string_is_taut() {
        let mut stabilizer = Stabilizer::new(ScribbleSmoothing::Regular, Pos2::ZERO).unwrap();
        let radius = ScribbleSmoothing::Regular.radius();
        assert_eq!(stabilizer.advance(pos2(radius, 0.0), Pos2::ZERO), None);
        let pen = stabilizer
            .advance(pos2(radius + 3.0, 0.0), Pos2::ZERO)
            .unwrap();
        assert!(pen.distance(pos2(3.0, 0.0)) < 1e-5, "{pen:?}");
        assert!(Stabilizer::new(ScribbleSmoothing::Off, Pos2::ZERO).is_none());
    }
    #[test]
    fn fitting_keeps_endpoints_and_flattens_within_tolerance() {
        let corner = [pos2(0.0, 0.0), pos2(40.0, 0.0), pos2(40.0, 40.0)];
        let fitted = fit(&corner);
        assert_eq!(fitted.first(), Some(&corner[0]));
        assert_eq!(fitted.last(), Some(&corner[2]));
        // The quadratic's midpoint is (30, 10); its nearest chord stays within tolerance.
        let middle = pos2(30.0, 10.0);
        let nearest = fitted
            .windows(2)
            .map(|pair| distance(middle, pair[0], pair[1]))
            .fold(f32::INFINITY, f32::min);
        assert!(nearest <= tokens::SMOOTH_TOLERANCE, "{nearest}");
    }
    #[test]
    fn simplification_collapses_straight_runs_and_keeps_corners() {
        let mut line: Vec<_> = (0..=600).map(|x| pos2(x as f32, 5.0)).collect();
        line.push(pos2(600.0, 100.0));
        assert_eq!(
            simplify(&line),
            [
                pos2(0.0, 5.0),
                pos2(256.0, 5.0),
                pos2(512.0, 5.0),
                pos2(600.0, 5.0),
                pos2(600.0, 100.0)
            ]
        );
    }
}
