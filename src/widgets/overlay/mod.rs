mod content;
pub(super) mod memory;
mod render;
mod types;
use crate::tokens::overlay::{DrawerWidth, ModalWidth};
use egui::{Context, IdSalt, Ui};
use render::{Config, Placement};
use std::borrow::Cow;
pub use types::{DrawerSide, OverlayDismissal, OverlayOptions, OverlayResponse};
/// Centered blocking dialog with native egui modality.
/// Call every frame, including while closed, to finish focus restoration.
#[must_use = "Call show or show_with_footer"]
pub struct Modal<'a> {
    config: Config<'a>,
}
impl<'a> Modal<'a> {
    /// Bind a unique ID, host-owned visibility, and visible/accessible title.
    pub fn new(
        id: impl egui::AsIdSalt,
        open: &'a mut bool,
        title: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            config: Config {
                id: IdSalt::new(id),
                open,
                title: title.into(),
                placement: Placement::Modal(ModalWidth::default()),
                options: OverlayOptions::default(),
            },
        }
    }
    /// Preferred token width; always constrained by the viewport.
    pub fn width(mut self, width: ModalWidth) -> Self {
        self.config.placement = Placement::Modal(width);
        self
    }
    /// Dismissal and initial/return focus policy.
    pub fn options(mut self, options: OverlayOptions) -> Self {
        self.config.options = options;
        self
    }
    /// Show a header and scrollable content. Returns None while closed.
    pub fn show<R>(
        self,
        ctx: &Context,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<OverlayResponse<R>> {
        render::show(self.config, ctx, body, None::<fn(&mut Ui)>).map(|r| r.map(|(body, _)| body))
    }
    /// Append a divided footer to the scrollable content, in native Tab order.
    pub fn show_with_footer<R, F>(
        self,
        ctx: &Context,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
    ) -> Option<OverlayResponse<(R, F)>> {
        render::show(self.config, ctx, body, Some(footer))
            .map(|r| r.map(|(body, footer)| (body, footer.unwrap())))
    }
}
/// Full-height blocking side panel. This is an overlay, not a docked SidePanel.
/// Call every frame, including while closed, to finish focus restoration.
#[must_use = "Call show or show_with_footer"]
pub struct Drawer<'a> {
    config: Config<'a>,
}
impl<'a> Drawer<'a> {
    /// Bind a unique ID, host-owned visibility, and visible/accessible title.
    pub fn new(
        id: impl egui::AsIdSalt,
        open: &'a mut bool,
        title: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            config: Config {
                id: IdSalt::new(id),
                open,
                title: title.into(),
                placement: Placement::Drawer(DrawerSide::default(), DrawerWidth::default()),
                options: OverlayOptions::default(),
            },
        }
    }
    /// Attach to the left or right viewport edge.
    pub fn side(mut self, side: DrawerSide) -> Self {
        if let Placement::Drawer(current, _) = &mut self.config.placement {
            *current = side;
        }
        self
    }
    /// Preferred token width; never wider than the viewport.
    pub fn width(mut self, width: DrawerWidth) -> Self {
        if let Placement::Drawer(_, current) = &mut self.config.placement {
            *current = width;
        }
        self
    }
    /// Dismissal and initial/return focus policy.
    pub fn options(mut self, options: OverlayOptions) -> Self {
        self.config.options = options;
        self
    }
    /// Show the header and scrollable content. Returns None while closed.
    pub fn show<R>(
        self,
        ctx: &Context,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<OverlayResponse<R>> {
        render::show(self.config, ctx, body, None::<fn(&mut Ui)>).map(|r| r.map(|(body, _)| body))
    }
    /// Append a divided footer to the scrollable content, in native Tab order.
    pub fn show_with_footer<R, F>(
        self,
        ctx: &Context,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
    ) -> Option<OverlayResponse<(R, F)>> {
        render::show(self.config, ctx, body, Some(footer))
            .map(|r| r.map(|(body, footer)| (body, footer.unwrap())))
    }
}

pub(in crate::widgets) use types::dismissal;
