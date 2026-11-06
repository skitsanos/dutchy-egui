use super::{Skeleton, SkeletonGroup, SkeletonVariant};
use crate::{
    Theme,
    tokens::{border, loading, space},
};
use egui::{Frame, Margin, Response, Stroke, Ui};

impl SkeletonGroup<'_> {
    /// Bordered card with title, description lines, and action placeholder.
    pub fn show_card(self, ui: &mut Ui) -> Response {
        self.show(ui, |ui| {
            frame(ui, border::ACCENT, space::XXL).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = space::LG;
                let width = ui.available_width();
                ui.add(Skeleton::new().width(width * loading::TITLE_FRACTION));
                ui.add(Skeleton::new().height(space::MD));
                ui.add(Skeleton::new().height(space::MD).width(width * (5.0 / 6.0)));
                ui.add_space(space::LG);
                ui.add(
                    Skeleton::new()
                        .width(loading::ACTION_WIDTH)
                        .height(space::LARGE),
                );
            });
        })
        .response
    }
    /// Bordered rows with avatar, two text lines, and a trailing action.
    pub fn show_list(self, ui: &mut Ui, rows: usize) -> Response {
        self.show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space::LG;
            for _ in 0..rows {
                frame(ui, border::STANDARD, space::LG).show(ui, |ui| {
                    let width = ui.available_width();
                    let avatar = space::LARGE.min(width / 4.0);
                    let action = space::PAGE.min(width / 4.0);
                    let gap = space::LG.min(width / 12.0);
                    let text = (width - avatar - action - 2.0 * gap).max(0.0);
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.horizontal(|ui| {
                        ui.add(Skeleton::new().width(avatar).height(avatar));
                        ui.vertical(|ui| {
                            ui.set_width(text);
                            ui.add(Skeleton::new().width(text * 0.5));
                            ui.add(Skeleton::new().width(text).height(space::MD));
                        });
                        ui.add(Skeleton::new().width(action).height(space::SECTION));
                    });
                });
            }
        })
        .response
    }
    /// Article/content layout with title, lines, and a media block.
    pub fn show_content(self, ui: &mut Ui) -> Response {
        self.show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space::XXL;
            ui.add(
                Skeleton::new()
                    .variant(SkeletonVariant::Title)
                    .width(ui.available_width() * 0.5),
            );
            ui.add(Skeleton::new().height(space::MD).count(3));
            ui.add(Skeleton::new().variant(SkeletonVariant::Card));
            ui.add(Skeleton::new().height(space::MD).count(2));
        })
        .response
    }
    /// Bordered four-column table, including a placeholder header.
    pub fn show_table(self, ui: &mut Ui, rows: usize) -> Response {
        self.show(ui, |ui| {
            frame(ui, border::STANDARD, space::LG).show(ui, |ui| {
                ui.add(Skeleton::new().variant(SkeletonVariant::TableRow));
                ui.separator();
                for _ in 0..rows {
                    ui.add(
                        Skeleton::new()
                            .variant(SkeletonVariant::TableRow)
                            .height(space::MD),
                    );
                    ui.add_space(space::LG);
                }
            });
        })
        .response
    }
}

fn frame(ui: &Ui, width: f32, padding: f32) -> Frame {
    Frame::NONE
        .stroke(Stroke::new(
            width,
            Theme::from_context(ui.ctx()).palette.border,
        ))
        .inner_margin(Margin::same(padding as i8))
}
