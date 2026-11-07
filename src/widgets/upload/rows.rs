use super::{FileItem, FileStatus, file_size, icon};
use crate::{
    Button, ButtonVariant, ControlSize, Family, Progress, ProgressSize, ProgressVariant, Text,
    TextColor, TextSize, Theme, Weight,
    tokens::{border, space, upload as tokens},
};
use egui::{Id, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo, vec2};
pub(super) fn show<K>(
    ui: &mut Ui,
    id: Id,
    item: &FileItem<'_, K>,
    anchor: egui::Pos2,
) -> (Option<Response>, bool) {
    let p = Theme::from_context(ui.ctx()).palette;
    let mut action = None;
    let mut retry = false;
    let row = ui
        .scope_builder(UiBuilder::new().id_salt(id), |ui| {
            if !item.enabled {
                ui.disable();
            }
            egui::Frame::NONE
                .stroke(Stroke::new(border::STANDARD, p.border))
                .inner_margin(tokens::ROW_PADDING)
                .show(ui, |ui| {
                    let width = ui.available_width();
                    ui.set_width(width);
                    let stacked = width
                        < tokens::MIN_TEXT_WIDTH + tokens::ACTION_SIZE + space::SM + space::LG;
                    let actions_width = if item.removable && !stacked {
                        tokens::ACTION_SIZE + space::SM
                    } else {
                        0.0
                    };
                    let show_icon = width
                        >= actions_width
                            + tokens::MIN_TEXT_WIDTH
                            + tokens::FILE_ICON_SIZE
                            + space::MD;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = space::SM;
                        if show_icon {
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(tokens::FILE_ICON_SIZE, tokens::ACTION_SIZE),
                                Sense::hover(),
                            );
                            icon::file(
                                ui,
                                egui::Rect::from_center_size(
                                    rect.center(),
                                    vec2(tokens::FILE_ICON_SIZE, tokens::FILE_ICON_SIZE),
                                ),
                                p.muted_foreground,
                            );
                        }
                        let content_width = (ui.available_width() - actions_width).max(1.0);
                        ui.vertical(|ui| {
                            ui.set_width(content_width);
                            let mut job = Text::new(item.name.as_ref())
                                .size(TextSize::Sm)
                                .weight(Weight::Medium)
                                .layout_job(&Theme::from_context(ui.ctx()), content_width);
                            job.wrap.break_anywhere = true;
                            ui.add(egui::Label::new(job).wrap());
                            ui.add(
                                Text::new(file_size(item.bytes))
                                    .family(Family::Mono)
                                    .size(TextSize::Xs)
                                    .color(TextColor::Muted),
                            );
                            status(ui, item);
                            if matches!(item.status, FileStatus::Failed(_)) {
                                let response = ui.add(
                                    Button::new("Retry")
                                        .variant(ButtonVariant::Outline)
                                        .size(ControlSize::Small)
                                        .full_width(content_width < tokens::RETRY_WIDTH),
                                );
                                response.widget_info(|| {
                                    WidgetInfo::labeled(
                                        egui::accesskit::Role::Button,
                                        ui.is_enabled(),
                                        format!("Retry upload {}", item.name),
                                    )
                                });
                                retry = response.clicked();
                            }
                        });
                        if item.removable && !stacked {
                            action = Some(action_control(ui, id, item, width, anchor));
                        }
                    });
                    if item.removable && stacked {
                        action = Some(action_control(ui, id, item, width, anchor));
                    }
                });
        })
        .response;
    row.widget_info(|| {
        WidgetInfo::labeled(
            egui::accesskit::Role::ListItem,
            ui.is_enabled() && item.enabled,
            item.name.as_ref(),
        )
    });
    (action, retry)
}
fn status<K>(ui: &mut Ui, item: &FileItem<'_, K>) {
    match &item.status {
        FileStatus::Uploading(value) => {
            ui.add(
                value
                    .map_or_else(Progress::indeterminate, Progress::new)
                    .id_salt("transfer")
                    .size(ProgressSize::Medium)
                    .show_value(true)
                    .accessible_name(format!("Uploading {}", item.name)),
            );
        }
        FileStatus::Complete => {
            ui.add(
                Progress::new(100.0)
                    .id_salt("transfer")
                    .size(ProgressSize::Medium)
                    .variant(ProgressVariant::Success)
                    .label("Complete")
                    .accessible_name(format!("Upload complete {}", item.name)),
            );
        }
        FileStatus::Failed(error) => {
            ui.add(
                Text::new(error)
                    .size(TextSize::Sm)
                    .color(TextColor::Destructive),
            );
        }
        FileStatus::Ready => {
            ui.add(
                Text::new("Ready to upload")
                    .size(TextSize::Sm)
                    .color(TextColor::Muted),
            );
        }
    }
}

fn action_control<K>(
    ui: &mut Ui,
    id: Id,
    item: &FileItem<'_, K>,
    width: f32,
    anchor: egui::Pos2,
) -> Response {
    let p = Theme::from_context(ui.ctx()).palette;
    let (_, rect) = ui.allocate_space(vec2(tokens::ACTION_SIZE.min(width), tokens::ACTION_SIZE));
    let response = ui.interact(rect, id.with("action"), Sense::click());
    let label = format!(
        "{} {}",
        if matches!(item.status, FileStatus::Uploading(_)) {
            "Cancel upload"
        } else {
            "Remove file"
        },
        item.name
    );
    response.widget_info(|| {
        WidgetInfo::labeled(
            egui::accesskit::Role::Button,
            ui.is_enabled(),
            label.as_str(),
        )
    });
    super::super::close::paint(
        ui,
        &response,
        tokens::CLOSE_ICON_SIZE,
        super::super::close::Colors {
            idle: p.muted_foreground,
            foreground: p.destructive,
            focus: p.primary,
        },
    );
    crate::interaction::reveal_focused(ui, &response, anchor);

    response
}
