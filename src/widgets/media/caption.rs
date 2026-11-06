use super::geometry as layout;
use crate::{Family, Text, TextSize, Theme, Tracking, tokens::image as geometry};
use egui::{Ui, UiBuilder, vec2};

pub(super) fn show(ui: &mut Ui, caption: Option<&str>, source: Option<&str>, color: egui::Color32) {
    if caption.is_none() && source.is_none() {
        return;
    }
    ui.add_space(geometry::CAPTION_GAP);
    let result = ui.scope_builder(UiBuilder::new().id_salt("caption"), |ui| {
        ui.spacing_mut().interact_size.y = 0.0;
        ui.spacing_mut().item_spacing.y = crate::tokens::space::XS;
        let credit = source.map(|source| {
            Text::new(source)
                .family(Family::Mono)
                .size(TextSize::Xs)
                .tracking(Tracking::Wide)
                .uppercase(true)
                .color(color)
        });
        let credit_width = credit.as_ref().map_or(0.0, |credit| {
            let job = credit
                .clone()
                .layout_job(&Theme::from_context(ui.ctx()), f32::INFINITY);
            ui.fonts_mut(|fonts| fonts.layout_job(job).size().x)
        });
        let width = ui.available_width();
        if caption.is_some()
            && credit.is_some()
            && credit_width + geometry::CAPTION_GAP < width / 2.0
        {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = geometry::CAPTION_GAP;
                ui.allocate_ui_with_layout(
                    vec2((width - credit_width - geometry::CAPTION_GAP).max(0.0), 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.add(Text::new(caption.unwrap()).size(TextSize::Sm).color(color));
                    },
                );
                ui.add(credit.unwrap());
            });
        } else {
            if let Some(caption) = caption {
                ui.add(Text::new(caption).size(TextSize::Sm).color(color));
            }
            if let Some(credit) = credit {
                ui.add(credit);
            }
        }
    });
    layout::bounds(&result.response);
    ui.ctx().accesskit_node_builder(result.response.id, |node| {
        node.set_role(egui::accesskit::Role::FigureCaption)
    });
}
