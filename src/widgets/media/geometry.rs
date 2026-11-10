use super::{ImageAspect, ImageFit};
use crate::tokens::image as tokens;
use egui::{Rect, Vec2, pos2, vec2};

pub(super) fn dimension(value: f32) -> Option<f32> {
    value
        .is_finite()
        .then(|| value.clamp(1.0, tokens::MAX_DIMENSION))
}
pub(super) fn width(available: f32, preferred: Option<f32>) -> f32 {
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        tokens::DEFAULT_WIDTH
    };
    preferred
        .unwrap_or(available)
        .min(available)
        .min(tokens::MAX_DIMENSION)
}
pub(super) fn aspect(aspect: ImageAspect, intrinsic: Option<Vec2>) -> f32 {
    match aspect {
        ImageAspect::Square => 1.0,
        ImageAspect::Video => 16.0 / 9.0,
        ImageAspect::Standard => 4.0 / 3.0,
        ImageAspect::Widescreen => 16.0 / 10.0,
        ImageAspect::Original => intrinsic
            .filter(|size| valid(*size))
            .map_or(tokens::FALLBACK_ASPECT, |size| size.x / size.y),
    }
}
pub(in crate::widgets) fn valid(size: Vec2) -> bool {
    size.is_finite() && size.x > 0.0 && size.y > 0.0
}
pub(super) fn bounds(response: &egui::Response) {
    let r = response.rect;
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_bounds(egui::accesskit::Rect {
            x0: r.left().into(),
            y0: r.top().into(),
            x1: r.right().into(),
            y1: r.bottom().into(),
        });
    });
}
/// Return a destination rectangle and normalized source UVs.
pub(in crate::widgets) fn placement(bounds: Rect, size: Vec2, fit: ImageFit) -> (Rect, Rect) {
    let unit = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    if !valid(size) || bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return (bounds, unit);
    }
    let scale = bounds.size() / size;
    if fit == ImageFit::Contain {
        return (
            Rect::from_center_size(bounds.center(), size * scale.x.min(scale.y)),
            unit,
        );
    }
    let fraction = bounds.size() / (size * scale.x.max(scale.y));
    (
        bounds,
        Rect::from_center_size(pos2(0.5, 0.5), vec2(fraction.x, fraction.y)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contain_preserves_the_whole_source_and_cover_crops_symmetrically() {
        let bounds = Rect::from_min_size(pos2(10.0, 20.0), vec2(100.0, 100.0));
        let (destination, uv) = placement(bounds, vec2(400.0, 100.0), ImageFit::Cover);
        assert_eq!(destination, bounds);
        assert_eq!(uv, Rect::from_min_max(pos2(0.375, 0.0), pos2(0.625, 1.0)));
        let (destination, uv) = placement(bounds, vec2(100.0, 400.0), ImageFit::Contain);
        assert_eq!(
            destination,
            Rect::from_center_size(bounds.center(), vec2(25.0, 100.0))
        );
        assert_eq!(uv, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)));
    }
    #[test]
    fn invalid_dimensions_and_unconstrained_layout_have_finite_defaults() {
        assert_eq!(dimension(f32::NAN), None);
        assert_eq!(dimension(f32::INFINITY), None);
        assert_eq!(dimension(-20.0), Some(1.0));
        assert_eq!(dimension(100000.0), Some(tokens::MAX_DIMENSION));
        assert_eq!(width(f32::INFINITY, None), tokens::DEFAULT_WIDTH);
        assert_eq!(width(0.0, Some(200.0)), 0.0);
        assert_eq!(
            aspect(ImageAspect::Original, Some(vec2(1.0, 0.0))),
            tokens::FALLBACK_ASPECT
        );
    }
}
