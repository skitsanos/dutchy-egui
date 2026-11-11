//! Blocking backdrops dim and intercept the whole window, including its edges.
mod common;
#[path = "common/media.rs"]
#[allow(dead_code)]
mod media;
use dutchy_egui::{
    Accent, CommandPalette, GalleryItem, Lightbox, MenuItem, Modal, ModalWidth, Mode,
    OverlayDismissal, OverlayOptions, SearchState, Theme, tokens::overlay::VIEWPORT_GUTTER,
};
use egui::{Pos2, Rect, Sense, pos2};
use egui_kittest::Harness;
use std::sync::{Arc, Mutex};

const SIZE: [f32; 2] = [800.0, 600.0];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Surface {
    Modal,
    Palette,
    Lightbox,
}
struct State {
    surface: Surface,
    open: bool,
    long: bool,
    options: OverlayOptions,
    outside: usize,
    dismissed: Vec<OverlayDismissal>,
    rect: Option<Rect>,
    backdrop: Option<Rect>,
    search: SearchState,
    selected: Option<u8>,
}
fn harness(surface: Surface, mode: Mode, scale: f32, size: [f32; 2]) -> Harness<'static, State> {
    let theme = Theme::new(Accent::Orange, mode).with_reduced_motion(true);
    let probe = Arc::new(Mutex::new(media::Probe::default()));
    let mut search = SearchState::default();
    search.open = true;
    let state = State {
        surface,
        open: true,
        long: false,
        options: OverlayOptions::default(),
        outside: 0,
        dismissed: Vec::new(),
        rect: None,
        backdrop: None,
        search,
        selected: Some(1),
    };
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(if mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_context_setup(move |ctx| {
            common::install(ctx, theme);
            media::install(ctx, probe.clone());
        })
        .build_ui_state(host, state)
}
fn host(ui: &mut egui::Ui, state: &mut State) {
    let ctx = ui.ctx().clone();
    let window = ctx.content_rect();
    ctx.layer_painter(ui.layer_id()).rect_filled(
        window,
        0,
        Theme::from_context(&ctx).palette.background,
    );
    if ui
        .interact(window, ui.scope_id().with("host"), Sense::click())
        .clicked()
    {
        state.outside += 1;
    }
    match state.surface {
        Surface::Modal => modal(&ctx, state),
        Surface::Palette => {
            let items = [MenuItem::new(1, "Dashboard")];
            if let Some(result) =
                CommandPalette::new("commands", &mut state.search, &items).show(&ctx)
            {
                state.rect = Some(result.response.rect);
            }
            state.open = state.search.open;
        }
        Surface::Lightbox => {
            let items = [GalleryItem::new(1, "fixture://image-1", "Photo 1")];
            if let Some(result) = Lightbox::new("viewer", &items, &mut state.selected)
                .options(state.options.clone())
                .show(&ctx)
            {
                state.rect = Some(result.response.rect);
                state.backdrop = Some(result.backdrop_response.interact_rect);
                state.dismissed.extend(result.dismissed);
            }
            state.open = state.selected.is_some();
        }
    }
}
fn modal(ctx: &egui::Context, state: &mut State) {
    let long = state.long;
    let result = Modal::new("surface", &mut state.open, "Edit record")
        .width(if long {
            ModalWidth::Large
        } else {
            ModalWidth::Small
        })
        .options(state.options.clone())
        .show(ctx, |ui| {
            ui.label("The application owns its values and saving.");
            for n in 0..if long { 40 } else { 0 } {
                ui.label(format!("Supporting detail {n:02}."));
            }
        });
    if let Some(result) = result {
        state.rect = Some(result.response.rect);
        state.backdrop = Some(result.backdrop_response.interact_rect);
        state.dismissed.extend(result.dismissed);
    }
}
/// Corners and edge midpoints, at the outer and inner limits of the gutter.
fn band([width, height]: [f32; 2]) -> Vec<Pos2> {
    let mut points = Vec::new();
    for inset in [2.0, VIEWPORT_GUTTER - 2.0] {
        let (right, bottom) = (width - inset, height - inset);
        let (center, middle) = (width / 2.0, height / 2.0);
        points.extend([
            pos2(inset, inset),
            pos2(right, inset),
            pos2(inset, bottom),
            pos2(right, bottom),
            pos2(center, inset),
            pos2(center, bottom),
            pos2(inset, middle),
            pos2(right, middle),
        ]);
    }
    points
}
fn click_at(h: &mut Harness<'_, State>, at: Pos2) {
    h.hover_at(at);
    h.run_steps(2);
    h.drag_at(at);
    h.run_steps(2);
    h.drop_at(at);
    h.run_steps(4);
}
#[test]
fn backdrops_dim_the_window_edges_like_the_rest_of_the_window() {
    for surface in [Surface::Modal, Surface::Palette, Surface::Lightbox] {
        for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
            let mut h = harness(surface, mode, scale, SIZE);
            h.run_steps(5);
            let image = h.render().unwrap();
            let pixel = |at: Pos2| {
                image
                    .get_pixel((at.x * scale) as u32, (at.y * scale) as u32)
                    .0
            };
            // Inside the gutter and outside every surface.
            let reference = pixel(pos2(20.0, 20.0));
            let plain = Theme::new(Accent::Orange, mode).palette.background;
            assert_ne!(
                reference,
                plain.to_array(),
                "{surface:?} {mode:?}: no backdrop"
            );
            for at in band(SIZE) {
                assert_eq!(
                    pixel(at),
                    reference,
                    "{surface:?} {mode:?} {scale}x: undimmed backdrop at {at:?}"
                );
            }
        }
    }
}
#[test]
fn host_target_receives_edge_clicks_while_nothing_blocks_it() {
    let mut h = harness(Surface::Modal, Mode::Light, 1.0, SIZE);
    h.state_mut().open = false;
    h.run_steps(3);
    click_at(&mut h, pos2(2.0, 2.0));
    click_at(&mut h, pos2(400.0, 598.0));
    assert_eq!(h.state().outside, 2);
}
#[test]
fn modal_backdrop_blocks_and_dismisses_from_the_window_edges() {
    for at in [pos2(2.0, 2.0), pos2(400.0, 598.0), pos2(790.0, 300.0)] {
        let mut h = harness(Surface::Modal, Mode::Light, 1.0, SIZE);
        h.state_mut().options.dismiss_on_backdrop = false;
        h.run_steps(5);
        assert_eq!(
            h.state().backdrop,
            Some(Rect::from_min_size(Pos2::ZERO, SIZE.into()))
        );
        click_at(&mut h, at);
        assert!(h.state().open, "{at:?}: locked backdrop dismissed");
        assert_eq!(h.state().outside, 0, "{at:?}: click reached the host");
        h.state_mut().options.dismiss_on_backdrop = true;
        h.run_steps(2);
        click_at(&mut h, at);
        assert!(!h.state().open, "{at:?}: backdrop click ignored");
        assert_eq!(h.state().dismissed, [OverlayDismissal::Backdrop]);
        assert_eq!(h.state().outside, 0, "{at:?}: click reached the host");
    }
}
#[test]
fn palette_and_lightbox_backdrops_dismiss_from_the_window_edges() {
    for surface in [Surface::Palette, Surface::Lightbox] {
        for at in [pos2(2.0, 2.0), pos2(400.0, 598.0)] {
            let mut h = harness(surface, Mode::Light, 1.0, SIZE);
            h.run_steps(5);
            click_at(&mut h, at);
            assert!(
                !h.state().open,
                "{surface:?} {at:?}: backdrop click ignored"
            );
            assert_eq!(
                h.state().outside,
                0,
                "{surface:?} {at:?}: click reached host"
            );
        }
    }
}
#[test]
fn modal_surface_keeps_the_gutter_and_height_limit() {
    for (size, long, scale) in [
        (SIZE, false, 1.0),
        (SIZE, true, 2.0),
        ([280.0, 360.0], true, 1.0),
        ([280.0, 360.0], true, 2.0),
    ] {
        let mut h = harness(Surface::Modal, Mode::Light, scale, size);
        h.state_mut().long = long;
        h.run_steps(6);
        let rect = h.state().rect.unwrap();
        let limit = Rect::from_min_size(Pos2::ZERO, size.into()).shrink(VIEWPORT_GUTTER);
        assert!(
            limit.expand(0.5).contains_rect(rect),
            "{size:?} {scale}x: {rect:?} leaves {limit:?}"
        );
        if long {
            assert!(
                (rect.height() - limit.height()).abs() <= 0.5,
                "{size:?} {scale}x: height {}",
                rect.height()
            );
        }
    }
}
