mod item;
mod paint;
use crate::{
    Dropdown, MenuItem, MenuItemVariant, MenuWidth, Theme,
    tokens::{border, navigation as geometry},
};
use egui::{Frame, Id, IdSalt, Margin, Response, Ui, UiBuilder, WidgetInfo, vec2};
pub use item::{BreadcrumbIconPainter, BreadcrumbItem};
use std::borrow::Cow;

/// Decorative hierarchy separator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BreadcrumbSeparator {
    /// Diagonal slash.
    Slash,
    /// Sharp right chevron.
    #[default]
    Chevron,
}
/// Navigation request and visible destination responses.
pub struct BreadcrumbsResponse<T> {
    /// Named navigation region.
    pub response: Response,
    /// Destination chosen by the user; the host performs navigation.
    pub navigate: Option<T>,
    /// Visible destinations in path order, including the current page.
    pub items: Vec<Response>,
    /// Collapsed-ancestor trigger, when shown.
    pub collapsed: Option<Response>,
}
/// Wrapping hierarchy with optional host-controlled collapsed ancestors.
#[must_use = "Call show to render the breadcrumbs"]
pub struct Breadcrumbs<'a, T> {
    id: IdSalt,
    items: &'a [BreadcrumbItem<'a, T>],
    separator: BreadcrumbSeparator,
    name: Cow<'a, str>,
    enabled: bool,
    bar: bool,
    collapse: Option<(usize, &'a mut bool)>,
}
impl<'a, T: Clone> Breadcrumbs<'a, T> {
    /// Bind stable destinations. The last item is current and never activates.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [BreadcrumbItem<'a, T>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            separator: BreadcrumbSeparator::default(),
            name: Cow::Borrowed("Breadcrumb"),
            enabled: true,
            bar: false,
            collapse: None,
        }
    }
    /// Separator style.
    pub fn separator(mut self, separator: BreadcrumbSeparator) -> Self {
        self.separator = separator;
        self
    }
    /// Navigation-region name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable ancestor and collapsed-menu actions.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Muted bar with token padding and a bottom border.
    pub fn bar(mut self, bar: bool) -> Self {
        self.bar = bar;
        self
    }
    /// Keep the root plus the trailing destinations, with hidden ancestors in a menu.
    /// At least two destinations are kept. The host owns the popup's open flag.
    pub fn collapse(mut self, visible: usize, open: &'a mut bool) -> Self {
        self.collapse = Some((visible.max(2), open));
        self
    }
    /// Return a navigation request without changing the supplied path or opening a URL.
    pub fn show(mut self, ui: &mut Ui) -> BreadcrumbsResponse<T> {
        let root = ui.make_persistent_id(("dutchy-breadcrumbs", self.id));
        let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            let p = Theme::from_context(ui.ctx()).palette;
            let frame = if self.bar {
                Frame::NONE.fill(p.muted).inner_margin(Margin::symmetric(
                    geometry::BAR_PADDING_X,
                    geometry::BAR_PADDING_Y,
                ))
            } else {
                Frame::NONE
            };
            let framed = frame.show(ui, |ui| {
                if self.bar {
                    ui.set_width(ui.available_width());
                }
                self.contents(ui, root)
            });
            if self.bar {
                let mut line = framed.response.rect;
                line.min.y = line.max.y - border::STANDARD;
                ui.painter().rect_filled(line, 0, p.border);
            }
            framed.inner
        });
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Navigation,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        BreadcrumbsResponse {
            response: result.response,
            navigate: result.inner.0,
            items: result.inner.1,
            collapsed: result.inner.2,
        }
    }
    fn contents(&mut self, ui: &mut Ui, root: Id) -> (Option<T>, Vec<Response>, Option<Response>) {
        let keep = self
            .collapse
            .as_ref()
            .map_or(self.items.len(), |(count, _)| *count);
        let start = self
            .items
            .len()
            .saturating_sub(keep.saturating_sub(1))
            .max(1);
        let hidden = self.items.len() > keep;
        if !hidden && let Some((_, open)) = &mut self.collapse {
            **open = false;
        }
        let mut navigate = None;
        let mut items = Vec::new();
        let mut collapsed = None;
        let list = ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing =
                vec2(geometry::BREADCRUMB_GAP, geometry::BREADCRUMB_GAP);
            for index in 0..self.items.len() {
                if hidden && index == 1 {
                    let actions: Vec<_> = self.items[1..start]
                        .iter()
                        .map(|item| MenuItem {
                            value: item.value.clone(),
                            label: item.label.clone(),
                            enabled: item.enabled,
                            icon: item.icon,
                            id: item.id,
                            group: None,
                            shortcut: None,
                            variant: MenuItemVariant::Default,
                            separator_before: false,
                        })
                        .collect();
                    if let Some((_, open)) = &mut self.collapse {
                        let trigger = chunk(
                            ui,
                            root.with("collapsed"),
                            true,
                            geometry::BREADCRUMB_HEIGHT,
                            self.separator,
                            |ui| {
                                let result = Dropdown::new("ancestors", open, &actions)
                                    .width(MenuWidth::Small)
                                    .accessible_name("Collapsed ancestors")
                                    .show(ui, |ui| {
                                        paint::collapsed(ui, root.with("collapsed-button"))
                                    });
                                navigate = result.action;
                                result.trigger
                            },
                        );
                        collapsed = Some(trigger);
                    }
                }
                if hidden && index > 0 && index < start {
                    continue;
                }
                let item = &self.items[index];
                let current = index + 1 == self.items.len();
                let width = paint::width(ui, item);
                let response = chunk(
                    ui,
                    root.with(("item", item.id)),
                    index > 0,
                    width,
                    self.separator,
                    |ui| {
                        ui.add_enabled_ui(item.enabled, |ui| {
                            paint::item(ui, root.with(item.id), item, current, ui.available_width())
                        })
                        .inner
                    },
                );
                if response.clicked() && !current {
                    navigate = Some(item.value.clone());
                    response.request_focus();
                }
                items.push(response);
            }
        });
        list.response.widget_info(|| {
            WidgetInfo::labeled(egui::accesskit::Role::List, ui.is_enabled(), "Path")
        });
        (navigate, items, collapsed)
    }
}
fn chunk(
    ui: &mut Ui,
    id: Id,
    has_separator: bool,
    width: f32,
    separator: BreadcrumbSeparator,
    content: impl FnOnce(&mut Ui) -> Response,
) -> Response {
    let prefix = if has_separator {
        geometry::ICON_SIZE + geometry::BREADCRUMB_GAP
    } else {
        0.0
    };
    let width = (prefix + width).min(ui.max_rect().width());
    ui.allocate_ui_with_layout(
        vec2(width, geometry::BREADCRUMB_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
                if has_separator {
                    paint::separator(ui, separator);
                }
                content(ui)
            })
            .inner
        },
    )
    .inner
}
