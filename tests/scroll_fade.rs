//! Scroll areas keep flat fills: the theme disables egui's edge fade gradient.
mod common;

use std::sync::{Arc, Mutex};

use dutchy_egui::{Accent, Mode, Theme};
use egui::{Color32, Rect, ScrollArea, Sense, vec2};
use egui_kittest::Harness;

#[test]
fn every_theme_style_disables_the_scroll_fade() {
    for &accent in Accent::ALL {
        for mode in [Mode::Light, Mode::Dark] {
            for reduced in [false, true] {
                let style = Theme::new(accent, mode)
                    .with_reduced_motion(reduced)
                    .style();
                assert_eq!(
                    style.spacing.scroll.fade.strength, 0.0,
                    "{accent:?} {mode:?} reduced motion {reduced}"
                );
            }
        }
    }
}

#[test]
fn applying_a_theme_disables_the_fade_for_both_egui_themes() {
    let ctx = egui::Context::default();
    Theme::new(Accent::Blue, Mode::Dark).apply(&ctx);
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        assert_eq!(
            ctx.style_of(theme).spacing.scroll.fade.strength,
            0.0,
            "{theme:?}"
        );
    }
}

/// Taller than the harness so the content overflows at both edges once the
/// view is scrolled, which is where egui would paint its fade gradients.
const CONTENT_HEIGHT: f32 = 400.0;
const SCROLL_OFFSET: f32 = 100.0;

type Viewport = Arc<Mutex<Option<Rect>>>;

fn harness(theme: Theme, scale: f32, viewport: Viewport) -> Harness<'static> {
    let fill = theme.palette.primary;
    Harness::builder()
        .with_size([200.0, 120.0])
        .with_render_cursor(false)
        .with_theme(match theme.mode {
            Mode::Light => egui::Theme::Light,
            Mode::Dark => egui::Theme::Dark,
        })
        .with_pixels_per_point(scale)
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui(move |ui| {
            let output = ScrollArea::vertical()
                .vertical_scroll_offset(SCROLL_OFFSET)
                .show(ui, |ui| {
                    let size = vec2(ui.available_width(), CONTENT_HEIGHT);
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    ui.painter().rect_filled(rect, 0, fill);
                });
            *viewport.lock().unwrap() = Some(output.inner_rect);
        })
}

fn matches(pixel: [u8; 4], color: Color32) -> bool {
    pixel
        .iter()
        .zip(color.to_array())
        .all(|(actual, expected)| actual.abs_diff(expected) <= 2)
}

#[test]
fn clipped_scroll_content_keeps_its_flat_fill_at_both_edges() {
    for (theme, scale) in [
        (Theme::new(Accent::Orange, Mode::Light), 1.0),
        (Theme::new(Accent::Blue, Mode::Dark), 2.0),
    ] {
        let viewport: Viewport = Arc::default();
        let mut h = harness(theme, scale, viewport.clone());
        h.run_steps(2);
        let image = h.render().unwrap();
        let inner = viewport.lock().unwrap().expect("the scroll area ran");
        let fill = theme.palette.primary;
        let x = ((inner.left() + 4.0) * scale).round() as u32;
        let top = ((inner.top() + 1.0) * scale).ceil() as u32;
        let bottom = ((inner.bottom() - 1.0) * scale).floor() as u32;
        assert!(
            bottom > top + 40,
            "viewport {inner:?} at {scale}x is too small to test"
        );
        for y in top..bottom {
            let pixel = image.get_pixel(x, y).0;
            assert!(
                matches(pixel, fill),
                "{:?} at {scale}x: pixel ({x}, {y}) is {pixel:?}, expected {:?}",
                theme.mode,
                fill.to_array()
            );
        }
    }
}
