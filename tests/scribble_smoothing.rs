//! Pointer smoothing: stabilized, fitted and simplified strokes; exact keyboard ink.
mod common;
use dutchy_egui::{
    Scribble, ScribbleBrush, ScribbleDrawing, ScribbleSmoothing, ScribbleStroke, Theme,
};
use egui::{
    Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, emath::RectTransform, pos2,
    vec2,
};
use egui_kittest::{Harness, kittest::Queryable};

const LEVELS: [ScribbleSmoothing; 3] = [
    ScribbleSmoothing::Light,
    ScribbleSmoothing::Regular,
    ScribbleSmoothing::Strong,
];

/// Deterministic hand tremor in `[-amplitude, amplitude]`.
fn tremor(seed: &mut u32, amplitude: f32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((*seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0) * amplitude
}
/// A shaky horizontal line along y = 100, sampled every two drawing units.
fn shaky_line() -> Vec<Pos2> {
    let mut seed = 7;
    (0..=250)
        .map(|step| pos2(50.0 + step as f32 * 2.0, 100.0 + tremor(&mut seed, 1.5)))
        .collect()
}
/// A shaky half circle of radius 80 around (300, 100).
fn shaky_arc() -> Vec<Pos2> {
    let mut seed = 11;
    (0..=200)
        .map(|step| {
            let angle = std::f32::consts::PI * step as f32 / 200.0;
            let radius = 80.0 + tremor(&mut seed, 1.5);
            pos2(300.0 + radius * angle.cos(), 100.0 + radius * angle.sin())
        })
        .collect()
}
/// Largest deviation from y = 100 away from the anchored ends.
fn wobble(points: &[Pos2]) -> f32 {
    points
        .iter()
        .filter(|point| (100.0..=500.0).contains(&point.x))
        .map(|point| (point.y - 100.0).abs())
        .fold(0.0, f32::max)
}
/// Sharpest direction change between consecutive segments, in degrees.
fn sharpest_turn(points: &[Pos2]) -> f32 {
    points
        .windows(3)
        .map(|w| (w[1] - w[0]).angle_to(w[2] - w[1]).abs().to_degrees())
        .fold(0.0, f32::max)
}
trait AngleTo {
    fn angle_to(self, other: egui::Vec2) -> f32;
}
impl AngleTo for egui::Vec2 {
    fn angle_to(self, other: egui::Vec2) -> f32 {
        (self.x * other.y - self.y * other.x).atan2(self.dot(other))
    }
}

#[test]
fn off_and_degenerate_input_pass_through_unchanged() {
    let line = shaky_line();
    assert_eq!(ScribbleSmoothing::Off.smooth(&line), line);
    assert_eq!(ScribbleSmoothing::default(), ScribbleSmoothing::Off);
    assert!(!ScribbleSmoothing::Off.is_enabled());
    for level in LEVELS {
        assert!(level.is_enabled());
        assert!(level.smooth(&[]).is_empty());
        assert_eq!(level.smooth(&[pos2(4.0, 5.0)]), [pos2(4.0, 5.0)]);
        assert_eq!(
            level.smooth(&[pos2(4.0, 5.0), pos2(4.2, 5.0)]),
            [pos2(4.0, 5.0)]
        );
    }
}
#[test]
fn tremor_is_removed_while_the_stroke_stays_anchored_and_compact() {
    let line = shaky_line();
    let raw = wobble(&line);
    let mut previous = raw;
    for level in LEVELS {
        let smoothed = level.smooth(&line);
        let wobble = wobble(&smoothed);
        println!(
            "{level:?}: {wobble:.3} of {raw:.3}, {} of {} points",
            smoothed.len(),
            line.len()
        );
        assert_eq!(smoothed.first(), line.first());
        assert_eq!(smoothed.last(), line.last());
        assert!(wobble < previous, "{level:?} wobbles {wobble} ≥ {previous}");
        assert!(
            smoothed.len() < line.len() / 2,
            "{level:?}: {}",
            smoothed.len()
        );
        previous = wobble;
    }
}
#[test]
fn curves_turn_gradually_instead_of_at_every_sample() {
    let arc = shaky_arc();
    let raw = sharpest_turn(&arc);
    for level in LEVELS {
        let turn = sharpest_turn(&level.smooth(&arc));
        println!("{level:?}: sharpest turn {turn:.2}° of {raw:.2}°");
        assert!(
            turn < raw / 2.0,
            "{level:?} turns {turn}° against raw {raw}°"
        );
    }
}
#[test]
fn smoothed_ink_hugging_the_edges_stays_inside_the_drawing() {
    let mut seed = 3;
    let mut point = pos2(0.0, 0.0);
    let walk: Vec<_> = (0..2000)
        .map(|_| {
            let step = vec2(tremor(&mut seed, 12.0), tremor(&mut seed, 12.0));
            point = (point + step).clamp(Pos2::ZERO, pos2(600.0, 200.0));
            point
        })
        .collect();
    let bounds = Rect::from_points(&walk);
    for level in LEVELS {
        let smoothed = level.smooth(&walk);
        assert!(smoothed.iter().all(|point| bounds.contains(*point)));
        let mut drawing = ScribbleDrawing::default();
        let stroke = ScribbleStroke::new(smoothed, ScribbleBrush::default()).unwrap();
        assert_eq!(drawing.push(stroke), Ok(()));
    }
}

struct State {
    drawing: ScribbleDrawing,
    smoothing: ScribbleSmoothing,
    rect: Rect,
}
fn harness(smoothing: ScribbleSmoothing) -> Harness<'static, State> {
    harness_at(smoothing, 1.0)
}
fn harness_at(smoothing: ScribbleSmoothing, scale: f32) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([740.0, 650.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                let result = Scribble::new("pad", &mut s.drawing)
                    .label("Drawing")
                    .smoothing(s.smoothing)
                    .show(ui);
                s.rect = result.canvas.rect;
            },
            State {
                drawing: ScribbleDrawing::default(),
                smoothing,
                rect: Rect::NOTHING,
            },
        );
    h.run_steps(3);
    h
}
fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn to_screen(h: &Harness<'_, State>) -> RectTransform {
    let drawing = Rect::from_min_size(Pos2::ZERO, h.state().drawing.size());
    RectTransform::from_to(drawing, h.state().rect)
}
fn to_drawing(h: &Harness<'_, State>) -> RectTransform {
    to_screen(h).inverse()
}
/// Press, move through, and release over `points` given in drawing coordinates;
/// returns the drawing-coordinate samples the widget received.
fn gesture(h: &mut Harness<'_, State>, points: &[Pos2], frames: usize) -> Vec<Pos2> {
    let screen: Vec<_> = points
        .iter()
        .map(|p| to_screen(h).transform_pos(*p))
        .collect();
    let last = screen.len() - 1;
    let mut events = vec![Event::PointerMoved(screen[0]), button(screen[0], true)];
    events.extend(screen[1..last].iter().map(|p| Event::PointerMoved(*p)));
    events.push(button(screen[last], false));
    let chunk = events.len().div_ceil(frames);
    for frame in events.chunks(chunk) {
        h.input_mut().events.extend(frame.iter().cloned());
        h.run_steps(1);
    }
    let back = to_drawing(h);
    let mut samples = vec![back.transform_pos(screen[0])];
    samples.extend(screen[1..].iter().map(|p| back.transform_pos(*p)));
    samples
}
fn assert_close(actual: &[Pos2], expected: &[Pos2]) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!(a.distance(*e) < 1e-3, "{a:?} != {e:?}");
    }
}

#[test]
fn pointer_gestures_commit_the_shared_smoothing_pipeline() {
    for level in LEVELS {
        let mut h = harness(level);
        let samples = gesture(&mut h, &shaky_line(), 1);
        let committed = h.state().drawing.strokes()[0].points();
        assert_close(committed, &level.smooth(&samples));
        assert!(committed.len() < samples.len() / 2);
    }
}
#[test]
fn smoothing_is_captured_when_a_gesture_starts() {
    let mut h = harness(ScribbleSmoothing::Strong);
    let screen = to_screen(&h);
    let start = screen.transform_pos(pos2(50.0, 100.0));
    h.input_mut()
        .events
        .extend([Event::PointerMoved(start), button(start, true)]);
    h.run_steps(1);
    h.state_mut().smoothing = ScribbleSmoothing::Off;
    let line = shaky_line();
    let samples = gesture_tail(&mut h, &line[1..]);
    let mut all = vec![to_drawing(&h).transform_pos(start)];
    all.extend(samples);
    assert_close(
        h.state().drawing.strokes()[0].points(),
        &ScribbleSmoothing::Strong.smooth(&all),
    );
    // The next gesture uses the newly selected level.
    let raw = gesture(&mut h, &line, 4);
    assert_close(h.state().drawing.strokes()[1].points(), &raw);
}
/// Continue an already pressed gesture through `points` and release at the last.
fn gesture_tail(h: &mut Harness<'_, State>, points: &[Pos2]) -> Vec<Pos2> {
    let screen: Vec<_> = points
        .iter()
        .map(|p| to_screen(h).transform_pos(*p))
        .collect();
    let last = screen.len() - 1;
    for chunk in screen[..last].chunks(64) {
        h.input_mut()
            .events
            .extend(chunk.iter().map(|p| Event::PointerMoved(*p)));
        h.run_steps(1);
    }
    h.input_mut().events.push(button(screen[last], false));
    h.run_steps(1);
    let back = to_drawing(h);
    screen.iter().map(|p| back.transform_pos(*p)).collect()
}
#[test]
fn clicks_still_make_dots() {
    let mut h = harness(ScribbleSmoothing::Strong);
    let point = to_screen(&h).transform_pos(pos2(120.0, 60.0));
    h.input_mut().events.extend([
        Event::PointerMoved(point),
        button(point, true),
        button(point, false),
    ]);
    h.run_steps(1);
    let stroke = &h.state().drawing.strokes()[0];
    assert_eq!(stroke.points().len(), 1);
    assert!(stroke.points()[0].distance(pos2(120.0, 60.0)) < 1e-3);
}
#[test]
fn keyboard_strokes_keep_exact_positions_while_pointer_smoothing_is_on() {
    let mut h = harness(ScribbleSmoothing::Strong);
    h.get_by_role_and_label(Role::Canvas, "Drawing").focus();
    h.run_steps(2);
    for key in [
        Key::Space,
        Key::ArrowRight,
        Key::ArrowRight,
        Key::ArrowDown,
        Key::ArrowRight,
        Key::Space,
    ] {
        h.key_press(key);
        h.run_steps(1);
    }
    assert_eq!(
        h.state().drawing.strokes()[0].points(),
        [
            pos2(300.0, 100.0),
            pos2(304.0, 100.0),
            pos2(308.0, 100.0),
            pos2(308.0, 104.0),
            pos2(312.0, 104.0),
        ]
    );
}

/// Dark ink pixels of the canvas, inside its border, at the given display scale.
struct Ink {
    width: usize,
    dark: Vec<bool>,
}
impl Ink {
    fn capture(h: &mut Harness<'_, State>, scale: f32) -> Self {
        let rect = h.state().rect.shrink(2.0);
        let image = h.render().unwrap();
        let columns = (rect.left() * scale).ceil() as u32..(rect.right() * scale).floor() as u32;
        let mut dark = Vec::new();
        for y in (rect.top() * scale).ceil() as u32..(rect.bottom() * scale).floor() as u32 {
            for x in columns.clone() {
                dark.push(image.get_pixel(x, y).0[..3].iter().all(|c| *c < 128));
            }
        }
        Self {
            width: columns.len(),
            dark,
        }
    }
    fn count(&self) -> usize {
        self.dark.iter().filter(|dark| **dark).count()
    }
    /// Pixels that are ink in exactly one capture.
    fn changed(&self, other: &Self) -> usize {
        self.dark
            .iter()
            .zip(&other.dark)
            .filter(|(a, b)| a != b)
            .count()
    }
    /// Ink pixels with no ink in `other` within `reach` pixels: visible geometry
    /// changes, ignoring sub-pixel antialiasing shifts.
    fn unmatched(&self, other: &Self, reach: usize) -> usize {
        let height = self.dark.len() / self.width;
        let near = |x: usize, y: usize| {
            (y.saturating_sub(reach)..(y + reach + 1).min(height)).any(|y| {
                (x.saturating_sub(reach)..(x + reach + 1).min(self.width))
                    .any(|x| other.dark[y * self.width + x])
            })
        };
        (0..self.dark.len())
            .filter(|index| self.dark[*index] && !near(index % self.width, index / self.width))
            .count()
    }
}
#[test]
fn live_smoothed_ink_matches_the_committed_stroke_across_scales() {
    let arc = shaky_arc();
    for scale in [1.0, 2.0] {
        let mut h = harness_at(ScribbleSmoothing::Regular, scale);
        let screen: Vec<_> = arc
            .iter()
            .map(|p| to_screen(&h).transform_pos(*p))
            .collect();
        h.input_mut()
            .events
            .extend([Event::PointerMoved(screen[0]), button(screen[0], true)]);
        h.input_mut()
            .events
            .extend(screen[1..].iter().map(|p| Event::PointerMoved(*p)));
        h.run_steps(1);
        let live = Ink::capture(&mut h, scale);
        h.input_mut()
            .events
            .push(button(screen[screen.len() - 1], false));
        h.run_steps(1);
        assert_eq!(h.state().drawing.strokes().len(), 1);
        let committed = Ink::capture(&mut h, scale);
        let mut raw = harness_at(ScribbleSmoothing::Off, scale);
        gesture(&mut raw, &arc, 1);
        let unsmoothed = Ink::capture(&mut raw, scale);
        // Simplification moves ink by at most 0.25 drawing units (under a pixel here).
        let reach = scale.ceil() as usize;
        let popped = live.unmatched(&committed, reach) + committed.unmatched(&live, reach);
        let ink = committed.count();
        // The tolerant comparison still sees tremor-sized displacement.
        let displaced =
            committed.unmatched(&unsmoothed, reach) + unsmoothed.unmatched(&committed, reach);
        let changed = committed.changed(&unsmoothed);
        println!("{scale}x: {ink} ink, {popped} popped, {displaced} displaced, {changed} changed");
        assert!(ink > 0);
        assert_eq!(popped, 0, "{scale}x: ink visibly changed on release");
        assert!(displaced > 0, "{scale}x: the pop check cannot see tremor");
        assert!(
            changed * 5 > ink,
            "{scale}x: smoothing barely changed ink ({changed} of {ink})"
        );
    }
}
