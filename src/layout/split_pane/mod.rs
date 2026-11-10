mod geometry;
mod input;
mod paint;
mod pointer;

use crate::tokens::split_pane as tokens;
use egui::{Align, Id, IdSalt, InnerResponse, Layout, Rect, Response, Sense, Ui, UiBuilder, vec2};
use geometry::{Axis, Bounds, dimension};
use std::borrow::Cow;

/// Two clipped, resizable panes with an application-owned first-pane size.
#[must_use = "Call show to lay out the panes"]
pub struct SplitPane<'a> {
    id: IdSalt,
    first_size: &'a mut f32,
    axis: Axis,
    min_sizes: [f32; 2],
    height: f32,
    name: Cow<'a, str>,
    enabled: bool,
    read_only: bool,
}
/// Child results and the fixed outer/divider responses.
#[non_exhaustive]
pub struct SplitPaneResponse<A, B> {
    /// Allocated outer rectangle. Changed when the first-pane size changes.
    pub response: Response,
    /// Focusable resizing control, with the same changed flag.
    pub divider: Response,
    /// First-pane result; its response rectangle is the allocated pane slot.
    pub first: InnerResponse<A>,
    /// Second-pane result; its response rectangle is the allocated pane slot.
    pub second: InnerResponse<B>,
}
impl<'a> SplitPane<'a> {
    /// Left/right panes, binding the left pane's width in logical points.
    pub fn horizontal(id: impl egui::AsIdSalt, first_size: &'a mut f32) -> Self {
        Self {
            id: IdSalt::new(id),
            first_size,
            axis: Axis::Horizontal,
            min_sizes: [tokens::MIN_SIZE; 2],
            height: tokens::HEIGHT,
            name: Cow::Borrowed("Primary pane"),
            enabled: true,
            read_only: false,
        }
    }
    /// Top/bottom panes, binding the top pane's height in logical points.
    pub fn vertical(id: impl egui::AsIdSalt, first_size: &'a mut f32) -> Self {
        Self {
            axis: Axis::Vertical,
            ..Self::horizontal(id, first_size)
        }
    }
    /// Desired minimum sizes. If they cannot fit, both shrink proportionally.
    pub fn min_sizes(mut self, first: f32, second: f32) -> Self {
        self.min_sizes = [first, second];
        self
    }
    /// Allocated height in logical points; defaults to 320 points. It is not
    /// capped by available height, so panes keep it on scrolling pages.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    /// Accessible name shared by the divider and its controlled primary pane.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable the complete composition, respecting disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Disable resizing while keeping both panes' controls usable.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Call each closure once in a stable, clipped scope. The first size is
    /// normalized/clamped even when resizing is disabled or read-only.
    pub fn show<A, B>(
        self,
        ui: &mut Ui,
        first: impl FnOnce(&mut Ui) -> A,
        second: impl FnOnce(&mut Ui) -> B,
    ) -> SplitPaneResponse<A, B> {
        // A vertical ScrollArea reports only the viewport left below the cursor
        // as available height, so a requested height is never capped by it.
        let size = vec2(
            dimension(ui.available_width(), tokens::WIDTH),
            dimension(self.height, tokens::HEIGHT),
        );
        let (_, rect) = ui.allocate_space(size);
        let id = ui.make_persistent_id(("dutchy-split-pane", self.id));
        let divider_id = id.with("divider");
        let mut root = ui.new_child(UiBuilder::new().scope_id(id).max_rect(rect));
        root.set_min_size(size);
        root.set_clip_rect(ui.clip_rect().intersect(rect));
        if !rect.is_positive() {
            root.set_invisible();
        }
        if !self.enabled {
            root.disable();
        }
        let bounds = Bounds::new(self.axis.length(rect), self.min_sizes[0], self.min_sizes[1]);
        let before = *self.first_size;
        *self.first_size = input::read(
            &root,
            divider_id,
            rect,
            self.axis,
            bounds,
            bounds.clamp(before),
            self.read_only,
        );
        let [first_rect, divider_rect, second_rect] = self.axis.rects(rect, *self.first_size);
        let first = pane(
            &mut root,
            id.with("first"),
            first_rect,
            Some(&self.name),
            first,
        );
        let mut divider = root.interact(divider_rect, divider_id, Sense::drag());
        if divider.clicked() || divider.is_pointer_button_down_on() {
            divider.request_focus();
        }
        if before != *self.first_size {
            divider.mark_changed();
        }
        paint::metadata(
            &root,
            &divider,
            paint::Info {
                name: &self.name,
                first: first.response.id,
                axis: self.axis,
                bounds,
                value: *self.first_size,
                read_only: self.read_only,
            },
        );
        paint::divider(&root, &divider, self.axis, self.read_only);
        let second = pane(&mut root, id.with("second"), second_rect, None, second);
        let mut response = root.response();
        response.rect = rect;
        if divider.changed() {
            response.mark_changed();
        }
        SplitPaneResponse {
            response,
            divider,
            first,
            second,
        }
    }
}
fn pane<R>(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    name: Option<&str>,
    contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let mut child = ui.new_child(
        UiBuilder::new()
            .scope_id(id)
            .max_rect(rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_min_size(rect.size());
    child.set_clip_rect(ui.clip_rect().intersect(rect));
    if !rect.is_positive() {
        child.set_invisible();
    }
    let inner = contents(&mut child);
    let mut response = child.response();
    response.rect = rect;
    response.widget_info(|| egui::WidgetInfo::new(egui::accesskit::Role::Group));
    if let Some(name) = name {
        child
            .ctx()
            .accesskit_node_builder(response.id, |node| node.set_label(name));
    }
    InnerResponse { inner, response }
}
