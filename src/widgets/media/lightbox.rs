use super::{GalleryItem, ImageResponse, viewer};
use crate::{
    OverlayDismissal, OverlayOptions, Theme,
    tokens::image as geometry,
    widgets::overlay::{dismissal, memory::Memory},
};
use egui::{Context, Id, IdSalt, LayerId, Order, Response};
use std::borrow::Cow;

/// Rendered viewer state; the host owns the bound selected key and loader policy.
pub struct LightboxResponse<K> {
    /// Dialog surface.
    pub response: Response,
    /// Full-viewport input-blocking backdrop.
    pub backdrop_response: Response,
    /// Key actually painted this pass, or None when it is missing/disabled.
    pub key: Option<K>,
    /// Image/placeholder and optional retry control.
    pub image: Option<ImageResponse>,
    /// Close button, unless explicitly hidden through OverlayOptions.
    pub close: Option<Response>,
    /// Previous-image button, when a sequence is available.
    pub previous: Option<Response>,
    /// Next-image button, when a sequence is available.
    pub next: Option<Response>,
    /// Reason the bound selection was cleared this pass.
    pub dismissed: Option<OverlayDismissal>,
}
/// Constrained fullscreen image viewer with native modality and keyed host state.
/// Call every frame, including while selection is None, for deferred focus return.
#[must_use = "Call show to render the lightbox"]
pub struct Lightbox<'a, K> {
    id: IdSalt,
    items: &'a [GalleryItem<'a, K>],
    selected: &'a mut Option<K>,
    name: Cow<'a, str>,
    options: OverlayOptions,
    wrap: bool,
}
impl<'a, K: Clone + PartialEq> Lightbox<'a, K> {
    /// Bind items and a selected key; None closes the viewer. Missing/disabled
    /// keys display closable unavailable guidance rather than choosing another item.
    pub fn new(
        id: impl egui::AsIdSalt,
        items: &'a [GalleryItem<'a, K>],
        selected: &'a mut Option<K>,
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            selected,
            name: "Image viewer".into(),
            options: OverlayOptions::default(),
            wrap: false,
        }
    }
    /// Visible/accessible viewer title.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Common dismissal and initial/return focus policy.
    pub fn options(mut self, options: OverlayOptions) -> Self {
        self.options = options;
        self
    }
    /// Wrap previous/next at collection ends. Disabled by default.
    pub fn wrap_navigation(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    /// Show the selected item, or finish focus restoration while closed.
    pub fn show(self, ctx: &Context) -> Option<LightboxResponse<K>> {
        let root = Id::unique(("dutchy-lightbox", self.id));
        let layer = LayerId::new(Order::Foreground, root);
        let mut memory = Memory::load(ctx, root);
        if self.selected.is_none() {
            memory.closed(ctx, layer);
            memory.store(ctx, root);
            return None;
        }
        memory.enter(ctx, &self.options);
        let index = self
            .items
            .iter()
            .position(|item| Some(&item.key) == self.selected.as_ref() && item.enabled);
        let previous = index.and_then(|index| neighbor(self.items, index, false, self.wrap));
        let next = index.and_then(|index| neighbor(self.items, index, true, self.wrap));
        let viewport = ctx.content_rect();
        let gutter = geometry::VIEWPORT_GUTTER
            .min(viewport.width() / 4.0)
            .min(viewport.height() / 4.0);
        let size = (viewport.size() - egui::Vec2::splat(gutter * 2.0)).max(egui::Vec2::ZERO);
        let area = egui::Modal::default_area(root)
            .accessible_name(self.name.to_string())
            .role(egui::accesskit::Role::Dialog)
            .fade_in(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(viewport.center())
            .constrain_to(viewport);
        let p = viewer::palette(Theme::from_context(ctx));
        let modal = egui::Modal::new(root)
            .area(area)
            .frame(egui::Frame::NONE.fill(p.background))
            .backdrop_color(geometry::VIEWER_BACKGROUND.gamma_multiply(geometry::BACKDROP_OPACITY))
            .show(ctx, |ui| {
                viewer::show(
                    ui,
                    root,
                    viewer::View {
                        name: self.name.as_ref(),
                        items: self.items,
                        index,
                        previous,
                        next,
                        size,
                        options: &self.options,
                        palette: p,
                    },
                )
            });
        if !modal.inner.sizing {
            memory.focus(ctx, layer, modal.inner.focus, &self.options);
        }
        let close = modal.inner.close.as_ref().is_some_and(Response::clicked);
        let dismissed = dismissal(&modal, close, &self.options);
        let requested = if dismissed.is_none() {
            navigation(ctx, &modal, self.items, index, previous, next)
        } else {
            None
        };
        if dismissed.is_some() {
            *self.selected = None;
            ctx.request_repaint();
        } else if let Some(index) = requested {
            *self.selected = Some(self.items[index].key.clone());
            ctx.request_repaint();
        }
        ctx.accesskit_node_builder(root, |node| {
            node.set_modal();
            let r = modal.response.rect;
            node.set_bounds(egui::accesskit::Rect {
                x0: r.left().into(),
                y0: r.top().into(),
                x1: r.right().into(),
                y1: r.bottom().into(),
            });
        });
        memory.store(ctx, root);
        Some(LightboxResponse {
            response: modal.response,
            backdrop_response: modal.backdrop_response,
            key: index.map(|index| self.items[index].key.clone()),
            image: modal.inner.image,
            close: modal.inner.close,
            previous: modal.inner.previous,
            next: modal.inner.next,
            dismissed,
        })
    }
}
fn navigation<K>(
    ctx: &Context,
    modal: &egui::ModalResponse<viewer::Contents>,
    items: &[GalleryItem<'_, K>],
    index: Option<usize>,
    previous: Option<usize>,
    next: Option<usize>,
) -> Option<usize> {
    if modal.inner.previous.as_ref().is_some_and(Response::clicked) {
        return previous;
    }
    if modal.inner.next.as_ref().is_some_and(Response::clicked) {
        return next;
    }
    if !modal.is_top_modal || modal.any_popup_open || index.is_none() {
        return None;
    }
    ctx.input_mut(|input| {
        if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft) {
            previous
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight) {
            next
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
            items.iter().position(|item| item.enabled)
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
            items.iter().rposition(|item| item.enabled)
        } else {
            None
        }
    })
    .filter(|requested| Some(*requested) != index)
}
fn neighbor<K>(
    items: &[GalleryItem<'_, K>],
    index: usize,
    next: bool,
    wrap: bool,
) -> Option<usize> {
    let result = if next {
        items
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, item)| item.enabled)
            .map(|(index, _)| index)
    } else {
        items[..index].iter().rposition(|item| item.enabled)
    };
    result
        .or_else(|| {
            if !wrap {
                return None;
            }
            if next {
                items.iter().position(|item| item.enabled)
            } else {
                items.iter().rposition(|item| item.enabled)
            }
        })
        .filter(|candidate| *candidate != index)
}
