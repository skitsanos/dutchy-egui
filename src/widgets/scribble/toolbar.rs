use super::{Scribble, ScribbleAction};
use crate::{Button, ButtonVariant, ControlSize};
use egui::{Id, Response, Ui, WidgetInfo};

pub(super) fn show(
    ui: &mut Ui,
    root: Id,
    config: &Scribble<'_>,
) -> Option<(ScribbleAction, Response)> {
    let mut request = None;
    ui.horizontal_wrapped(|ui| {
        for (action, label, enabled) in actions(config) {
            // Let the wrapping layout allocate the button before its style scope.
            // The enclosing stable widget scope and fixed action order preserve IDs.
            let response = ui.add(
                Button::new(label)
                    .variant(if action == ScribbleAction::Clear {
                        ButtonVariant::Ghost
                    } else {
                        ButtonVariant::Outline
                    })
                    .size(ControlSize::Small)
                    .enabled(enabled),
            );
            response.widget_info(|| {
                WidgetInfo::labeled(
                    egui::accesskit::Role::Button,
                    ui.is_enabled() && enabled,
                    format!("{label} {}", config.label),
                )
            });
            if response.gained_focus() || response.clicked() {
                response.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
            }
            if response.clicked() {
                request = Some((action, response));
            }
        }
    });
    if request.is_some() {
        let key = root.with(("toolbar-frame", ui.ctx().viewport_id()));
        let frame = ui.ctx().cumulative_frame_nr();
        if ui
            .ctx()
            .data(|data| data.get_temp::<u64>(key) == Some(frame))
        {
            return None;
        }
        ui.ctx().data_mut(|data| data.insert_temp(key, frame));
    }
    request
}
fn actions(config: &Scribble<'_>) -> Vec<(ScribbleAction, &'static str, bool)> {
    let edit = !config.read_only;
    let export = config.exports_enabled && !config.drawing.is_empty();
    let actions = vec![
        (
            ScribbleAction::Undo,
            "Undo",
            edit && config.drawing.can_undo(),
        ),
        (
            ScribbleAction::Redo,
            "Redo",
            edit && config.drawing.can_redo(),
        ),
        (
            ScribbleAction::Clear,
            "Clear",
            edit && !config.drawing.is_empty(),
        ),
        (ScribbleAction::Preview, "Preview", export),
        (ScribbleAction::ExportSvg, "Export SVG", export),
    ];
    #[cfg(feature = "png-export")]
    {
        let mut actions = actions;
        actions.push((ScribbleAction::ExportPng, "Export PNG", export));
        actions
    }
    #[cfg(not(feature = "png-export"))]
    actions
}
