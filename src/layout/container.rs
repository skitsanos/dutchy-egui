use crate::{
    ContainerWidth, Spacing,
    tokens::{layout::WIDE_GUTTER_BREAKPOINT, space},
};
use egui::{Align, Frame, IdSalt, InnerResponse, Layout, Margin, Ui, UiBuilder};

/// Centered content with a maximum outer width and horizontal gutters.
#[derive(Clone, Debug)]
#[must_use = "Call show to lay out content"]
pub struct Container {
    id: IdSalt,
    max_width: ContainerWidth,
    padding: Option<Spacing>,
}

impl Container {
    /// 1280-point maximum, with 16/24-point viewport-dependent gutters.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: IdSalt::new(id),
            max_width: ContainerWidth::default(),
            padding: None,
        }
    }
    /// Maximum outer width, including gutters.
    pub fn max_width(mut self, width: ContainerWidth) -> Self {
        self.max_width = width;
        self
    }
    /// Override the responsive horizontal gutter with a shared spacing token.
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = Some(padding);
        self
    }
    /// Render a centered container. Children must respect their available width.
    pub fn show<R>(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let available = ui.available_width().max(0.0);
        let width = self
            .max_width
            .points()
            .map_or(available, |max| available.min(max));
        let default_padding = if ui.ctx().content_rect().width() >= WIDE_GUTTER_BREAKPOINT {
            space::XXL
        } else {
            space::LG
        };
        let padding = self
            .padding
            .map_or(default_padding, Spacing::points)
            .min(width / 2.0);
        let id = ui.make_persistent_id(("dutchy-container", self.id));
        ui.scope_builder(
            UiBuilder::new()
                .scope_id(id)
                .layout(Layout::top_down(Align::Center)),
            |ui| {
                ui.set_min_width(available);
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        Frame::NONE
                            .inner_margin(Margin::symmetric(padding as i8, 0))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                contents(ui)
                            })
                    },
                )
                .inner
            },
        )
        .inner
    }
}
