//! Host model, immutable snapshots, bounded history and geometry/alpha exports.
use dutchy_egui::{
    ScribbleBrush as Brush, ScribbleDrawing as Drawing, ScribbleError as Error,
    ScribbleExportOptions as Options, ScribbleStroke as Stroke, ScribbleWidth as Width,
};
use egui::{Color32, pos2};
fn dot(x: f32, y: f32) -> Stroke {
    Stroke::new(vec![pos2(x, y)], Brush::new()).unwrap()
}
#[test]
fn invalid_input_is_rejected_without_mutating_data_or_redo_history() {
    for size in [(0, 200), (600, 0), (4097, 200)] {
        assert_eq!(
            Drawing::new(size.0, size.1).unwrap_err(),
            Error::InvalidDimensions
        );
    }
    assert_eq!(
        Stroke::new(vec![], Brush::new()).unwrap_err(),
        Error::EmptyStroke
    );
    assert_eq!(
        Stroke::new(vec![pos2(f32::NAN, 0.0)], Brush::new()).unwrap_err(),
        Error::InvalidPoint
    );
    let mut drawing = Drawing::default();
    drawing.push(dot(10.0, 10.0)).unwrap();
    drawing.undo();
    let revision = drawing.revision();
    assert_eq!(drawing.push(dot(-1.0, 10.0)), Err(Error::OutOfBounds));
    assert_eq!(drawing.revision(), revision);
    assert!(drawing.can_redo());
    assert!(drawing.is_empty());
}
#[test]
fn undo_redo_clear_and_branching_are_host_owned_and_snapshots_remain_immutable() {
    let mut drawing = Drawing::default();
    drawing.push(dot(10.0, 10.0)).unwrap();
    drawing.push(dot(20.0, 20.0)).unwrap();
    let snapshot = drawing.snapshot();
    assert!(drawing.clear());
    assert!(drawing.is_empty());
    assert!(drawing.undo());
    assert_eq!(drawing.strokes(), snapshot.strokes());
    assert!(drawing.redo());
    assert!(drawing.is_empty());
    assert!(drawing.undo());
    assert!(drawing.undo());
    assert_eq!(drawing.strokes().len(), 1);
    drawing.push(dot(30.0, 30.0)).unwrap();
    assert!(!drawing.can_redo());
    assert_eq!(snapshot.strokes()[1].points(), [pos2(20.0, 20.0)]);
    assert_ne!(snapshot.revision(), drawing.revision());
}
#[test]
fn history_and_stroke_point_capacities_are_bounded() {
    use dutchy_egui::scribble::{HISTORY_LIMIT, MAX_POINTS, MAX_STROKE_POINTS, MAX_STROKES};
    let mut drawing = Drawing::default();
    for _ in 0..HISTORY_LIMIT + 4 {
        drawing.push(dot(10.0, 10.0)).unwrap();
    }
    for _ in 0..HISTORY_LIMIT {
        assert!(drawing.undo());
    }
    assert!(!drawing.undo());
    assert_eq!(drawing.strokes().len(), 4);
    let mut drawing = Drawing::default();
    for _ in 0..MAX_STROKES {
        drawing.push(dot(10.0, 10.0)).unwrap();
    }
    assert!(matches!(
        drawing.push(dot(10.0, 10.0)),
        Err(Error::Capacity(_))
    ));
    assert!(Stroke::new(vec![pos2(0.0, 0.0); MAX_STROKE_POINTS + 1], Brush::new()).is_err());
    let mut drawing = Drawing::default();
    for _ in 0..MAX_POINTS / MAX_STROKE_POINTS {
        drawing
            .push(Stroke::new(vec![pos2(0.0, 0.0); MAX_STROKE_POINTS], Brush::new()).unwrap())
            .unwrap();
    }
    assert_eq!(drawing.point_count(), MAX_POINTS);
    assert!(drawing.push(dot(0.0, 0.0)).is_err());
}
#[test]
fn svg_preserves_vector_geometry_dots_colors_and_optional_background() {
    let mut drawing = Drawing::new(64, 32).unwrap();
    drawing
        .push(
            Stroke::new(
                vec![pos2(2.5, 4.5), pos2(40.5, 4.5)],
                Brush::new()
                    .color(Color32::from_rgb(40, 80, 120))
                    .width(Width::Bold),
            )
            .unwrap(),
        )
        .unwrap();
    drawing.push(dot(12.5, 12.5)).unwrap();
    let svg = drawing
        .snapshot()
        .to_svg(Options {
            scale: 2.0,
            ..Default::default()
        })
        .unwrap();
    assert!(svg.contains("width=\"128\" height=\"64\" viewBox=\"0 0 128 64\""));
    assert!(svg.contains("5.000,9.000 81.000,9.000"));
    assert!(svg.contains("stroke=\"#285078\""));
    assert!(svg.contains("stroke-width=\"8\""));
    assert!(svg.contains("<circle"));
    assert!(!svg.contains("<rect"));
    let paper = drawing
        .snapshot()
        .to_svg(Options {
            background: Some(Color32::WHITE),
            ..Default::default()
        })
        .unwrap();
    assert!(paper.contains("<rect"));
    assert!(!paper.contains("<script"));
    assert!(!paper.contains("href="));
}
#[test]
fn raster_contains_ink_only_and_composites_connected_alpha_once_per_stroke() {
    let mut drawing = Drawing::new(32, 24).unwrap();
    let ink = Color32::from_rgba_unmultiplied(0, 0, 0, 128);
    drawing
        .push(
            Stroke::new(
                vec![pos2(4.5, 4.5), pos2(20.5, 4.5), pos2(20.5, 18.5)],
                Brush::new().color(ink).width(Width::Bold),
            )
            .unwrap(),
        )
        .unwrap();
    let snapshot = drawing.snapshot();
    let image = snapshot.rasterize(Default::default()).unwrap();
    assert_eq!(image.size, [32, 24]);
    assert_eq!(image.pixels[0], Color32::TRANSPARENT);
    assert_eq!(image.pixels[4 * 32 + 20].a(), 128);
    assert_eq!(image.pixels[4 * 32 + 10].a(), 128);
    let paper = snapshot
        .rasterize(Options {
            background: Some(Color32::WHITE),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(paper.pixels[0], Color32::WHITE);
    assert_eq!(paper.pixels[4 * 32 + 10].a(), 255);
    assert!((125..=129).contains(&paper.pixels[4 * 32 + 10].r()));
}
#[test]
fn export_size_pixel_and_work_limits_reject_before_unbounded_allocations() {
    let snapshot = Drawing::default().snapshot();
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY, 1000.0] {
        assert_eq!(
            snapshot.to_svg(Options {
                scale,
                ..Default::default()
            }),
            Err(Error::InvalidExportSize)
        );
    }
    let mut large = Drawing::new(4096, 4096).unwrap();
    assert_eq!(
        large
            .snapshot()
            .rasterize(Options {
                scale: 2.0,
                ..Default::default()
            })
            .unwrap_err(),
        Error::RasterTooLarge
    );
    assert!(
        large
            .snapshot()
            .to_svg(Options {
                scale: 2.0,
                ..Default::default()
            })
            .is_ok()
    );
    for _ in 0..4 {
        large
            .push(Stroke::new(vec![pos2(0.0, 0.0), pos2(4096.0, 4096.0)], Brush::new()).unwrap())
            .unwrap();
    }
    assert_eq!(
        large.snapshot().rasterize(Default::default()).unwrap_err(),
        Error::RasterTooComplex
    );
}
#[cfg(feature = "png-export")]
#[test]
fn png_decodes_to_the_same_rgba_pixels_and_dimensions_as_the_public_raster() {
    let mut drawing = Drawing::new(40, 20).unwrap();
    drawing.push(dot(10.5, 10.5)).unwrap();
    let snapshot = drawing.snapshot();
    let options = Options {
        scale: 2.0,
        ..Default::default()
    };
    let rgba = snapshot.rasterize(options).unwrap();
    let png = snapshot.to_png(options).unwrap();
    let decoded = image::load_from_memory(&png).unwrap().into_rgba8();
    assert_eq!(decoded.dimensions(), (80, 40));
    let expected: Vec<_> = rgba
        .pixels
        .iter()
        .flat_map(Color32::to_srgba_unmultiplied)
        .collect();
    assert_eq!(decoded.as_raw(), &expected);
}

#[test]
fn fractional_export_rounding_uses_uniform_scale_and_background_covers_every_pixel() {
    let mut drawing = Drawing::new(3, 2).unwrap();
    drawing.push(dot(1.5, 1.0)).unwrap();
    let options = Options {
        scale: 1.5,
        background: Some(Color32::WHITE),
    };
    let snapshot = drawing.snapshot();
    let svg = snapshot.to_svg(options).unwrap();
    assert!(svg.contains("viewBox=\"0 0 5 3\""));
    assert!(svg.contains("<rect width=\"5\" height=\"3\""));
    assert!(svg.contains("cx=\"2.500\" cy=\"1.500\" r=\"1.500\""));
    let raster = snapshot.rasterize(options).unwrap();
    assert_eq!(raster.size, [5, 3]);
    assert!(raster.pixels.iter().all(|pixel| pixel.a() == 255));
}
