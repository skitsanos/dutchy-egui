mod pages;
mod paint;
use crate::tokens::navigation as geometry;
use egui::{IdSalt, Response, Ui, UiBuilder, WidgetInfo};
use std::borrow::Cow;
/// Pagination presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationVariant {
    /// Bordered page numbers, decorative ellipses, and previous/next arrows.
    #[default]
    Standard,
    /// Textual previous/next controls and a page-count summary.
    Compact,
}
/// Effective range and a host-owned navigation request.
pub struct PaginationResponse {
    /// Named native navigation region.
    pub response: Response,
    /// Requested one-based page. No request is emitted for the current page.
    pub requested_page: Option<usize>,
    /// View-clamped current page, or None when there are no pages.
    pub current_page: Option<usize>,
    /// Controls in visual order, including disabled directions and current page.
    pub controls: Vec<Response>,
}
/// Bounded page navigation. The host owns the current page, data, and loading.
#[must_use = "Call show to render pagination"]
pub struct Pagination<'a> {
    id: IdSalt,
    current: usize,
    total: usize,
    variant: PaginationVariant,
    name: Cow<'a, str>,
    enabled: bool,
}
impl<'a> Pagination<'a> {
    /// One-based current page and page count. Invalid input is clamped for display
    /// only; no navigation or host-state mutation occurs without activation.
    pub fn new(id: impl egui::AsIdSalt, current: usize, total: usize) -> Self {
        Self {
            id: IdSalt::new(id),
            current,
            total,
            variant: PaginationVariant::default(),
            name: Cow::Borrowed("Pagination"),
            enabled: true,
        }
    }
    /// Numbered or compact presentation.
    pub fn variant(mut self, variant: PaginationVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Name multiple pagers distinctly.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable navigation, for example while the host loads data.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Native Tab/Shift+Tab and Enter/Space behavior. Numbered controls wrap at
    /// narrow widths and retain their keyed identities as the page window changes.
    pub fn show(self, ui: &mut Ui) -> PaginationResponse {
        let root = ui.make_persistent_id(("dutchy-pagination", self.id));
        let current = (self.total > 0).then(|| self.current.clamp(1, self.total));
        let mut request = None;
        let mut controls = Vec::new();
        let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            let compact = self.variant == PaginationVariant::Compact;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::splat(if compact {
                    geometry::COMPACT_GAP
                } else {
                    geometry::PAGE_GAP
                });
                let previous = paint::control(
                    ui,
                    root.with("previous"),
                    paint::Kind::Direction {
                        next: false,
                        compact,
                    },
                    current.is_some_and(|page| page > 1),
                );
                if previous.clicked() {
                    request = current.map(|page| page - 1);
                    previous.request_focus();
                }
                controls.push(previous);
                if compact || current.is_none() {
                    paint::summary(
                        ui,
                        &current.map_or_else(
                            || "No pages".into(),
                            |page| format!("Page {page} of {}", self.total),
                        ),
                    );
                } else if let Some(current) = current {
                    for entry in pages::entries(current, self.total) {
                        match entry {
                            pages::Entry::Ellipsis => paint::ellipsis(ui),
                            pages::Entry::Page(number) => {
                                let response = paint::control(
                                    ui,
                                    root.with(("page", number)),
                                    paint::Kind::Page {
                                        number,
                                        current: number == current,
                                    },
                                    true,
                                );
                                if response.clicked() {
                                    response.request_focus();
                                    if number != current {
                                        request = Some(number);
                                    }
                                }
                                controls.push(response);
                            }
                        }
                    }
                }
                let next = paint::control(
                    ui,
                    root.with("next"),
                    paint::Kind::Direction {
                        next: true,
                        compact,
                    },
                    current.is_some_and(|page| page < self.total),
                );
                if next.clicked() {
                    request = current.map(|page| page + 1);
                    next.request_focus();
                }
                controls.push(next);
            });
        });
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Navigation,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        ui.ctx().accesskit_node_builder(result.response.id, |node| {
            node.set_description(current.map_or_else(
                || "No pages".into(),
                |page| format!("Page {page} of {}", self.total),
            ))
        });
        PaginationResponse {
            response: result.response,
            requested_page: request,
            current_page: current,
            controls,
        }
    }
}
