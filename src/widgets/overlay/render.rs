use super::{DrawerSide, OverlayOptions, OverlayResponse, content, memory::Memory};
use crate::{
    Theme,
    tokens::{
        border, motion, overlay as geometry,
        overlay::{DrawerWidth, ModalWidth},
    },
};
use egui::{Align2, Context, Id, IdSalt, LayerId, Order, Ui, vec2};
use std::borrow::Cow;
#[derive(Clone, Copy)]
pub(super) enum Placement {
    Modal(ModalWidth),
    Drawer(DrawerSide, DrawerWidth),
}
impl Placement {
    pub fn is_drawer(self) -> bool {
        matches!(self, Self::Drawer(..))
    }
    fn width(self, ctx: &Context) -> f32 {
        match self {
            Self::Modal(width) => width
                .points()
                .min((ctx.content_rect().width() - geometry::VIEWPORT_GUTTER * 2.0).max(0.0)),
            Self::Drawer(_, width) => width.points().min(ctx.content_rect().width()),
        }
    }
}
pub(super) struct Config<'a> {
    pub id: IdSalt,
    pub open: &'a mut bool,
    pub title: Cow<'a, str>,
    pub placement: Placement,
    pub options: OverlayOptions,
}
pub(super) fn show<R, F>(
    config: Config<'_>,
    ctx: &Context,
    body: impl FnOnce(&mut Ui) -> R,
    footer: Option<impl FnOnce(&mut Ui) -> F>,
) -> Option<OverlayResponse<(R, Option<F>)>> {
    let root = Id::unique(("dutchy-overlay", config.id));
    let layer = LayerId::new(Order::Foreground, root);
    let mut memory = Memory::load(ctx, root);
    if !*config.open {
        memory.closed(ctx, layer);
        memory.store(ctx, root);
        return None;
    }
    let opening = memory.enter(ctx, &config.options);
    let theme = Theme::from_context(ctx);
    let width = config.placement.width(ctx);
    let drawer = config.placement.is_drawer();
    let frame = egui::Frame::NONE
        .fill(theme.palette.background)
        .stroke(if drawer {
            egui::Stroke::NONE
        } else {
            egui::Stroke::new(border::ACCENT, theme.palette.border)
        });
    let inset = if drawer { 0.0 } else { border::ACCENT * 2.0 };
    let height = (ctx.content_rect().height()
        - inset
        - if drawer {
            0.0
        } else {
            geometry::VIEWPORT_GUTTER * 2.0
        })
    .max(0.0);
    let area = area(ctx, root, &config, width, opening);
    let modal = egui::Modal::new(root)
        .area(area)
        .frame(frame)
        .backdrop_color(
            theme
                .palette
                .foreground
                .gamma_multiply(geometry::BACKDROP_OPACITY),
        )
        .show(ctx, |ui| {
            content::show(
                ui,
                root,
                &config,
                (width - inset).max(0.0),
                height,
                body,
                footer,
            )
        });
    if !modal.inner.sizing {
        memory.focus(ctx, layer, modal.inner.focus, &config.options);
    }
    let dismissed = super::dismissal(&modal, modal.inner.close, &config.options);
    if dismissed.is_some() {
        *config.open = false;
        ctx.request_repaint();
    }
    if !modal.inner.sizing
        && let Placement::Drawer(side, _) = config.placement
    {
        edge(ctx, layer, modal.response.rect, side);
    }
    ctx.accesskit_node_builder(root, |node| {
        node.set_modal();
        let rect = modal.response.rect;
        node.set_bounds(egui::accesskit::Rect {
            x0: rect.left().into(),
            y0: rect.top().into(),
            x1: rect.right().into(),
            y1: rect.bottom().into(),
        });
    });
    memory.store(ctx, root);
    Some(OverlayResponse {
        response: modal.response,
        backdrop_response: modal.backdrop_response,
        inner: modal.inner.inner,
        dismissed,
    })
}
fn area(ctx: &Context, root: Id, config: &Config<'_>, width: f32, opening: bool) -> egui::Area {
    let theme = Theme::from_context(ctx);
    let area = egui::Modal::default_area(root)
        .accessible_name(config.title.to_string())
        .role(egui::accesskit::Role::Dialog)
        .fade_in(false);
    match config.placement {
        Placement::Modal(_) => {
            area.constrain_to(ctx.content_rect().shrink(geometry::VIEWPORT_GUTTER))
        }
        Placement::Drawer(side, _) => {
            let animation = root.with("entrance");
            if opening {
                ctx.animate_value_with_time(animation, 0.0, 0.0);
            }
            let duration = if theme.reduced_motion {
                0.0
            } else {
                motion::NORMAL
            };
            let amount = ctx.animate_value_with_time(animation, 1.0, duration);
            let offset = width * (1.0 - amount);
            let (anchor, offset) = match side {
                DrawerSide::Left => (Align2::LEFT_TOP, -offset),
                DrawerSide::Right => (Align2::RIGHT_TOP, offset),
            };
            area.default_size(vec2(width, ctx.content_rect().height()))
                .anchor(anchor, vec2(offset, 0.0))
                .constrain(false)
        }
    }
}

fn edge(ctx: &Context, layer: LayerId, rect: egui::Rect, side: DrawerSide) {
    let x = match side {
        DrawerSide::Left => rect.right() - border::ACCENT / 2.0,
        DrawerSide::Right => rect.left() + border::ACCENT / 2.0,
    };
    ctx.layer_painter(layer).line_segment(
        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
        egui::Stroke::new(border::ACCENT, Theme::from_context(ctx).palette.border),
    );
}
