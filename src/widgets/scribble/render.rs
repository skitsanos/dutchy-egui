use super::{Scribble, ScribbleResponse, capture, keyboard, paint, toolbar};
use crate::{Text, TextColor, TextSize, tokens::space};
use egui::{Response, Sense, Ui, UiBuilder, WidgetInfo};

pub(super) fn show(config: Scribble<'_>, ui: &mut Ui) -> ScribbleResponse {
    let root = ui.make_persistent_id(("dutchy-scribble", config.id));
    let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
        if !config.enabled {
            ui.disable();
        }
        ui.add(Text::label(config.label.as_ref()));
        ui.add_space(space::SM);
        let (outer, rect) = paint::allocate(ui, config.drawing, config.width);
        let editable = ui.is_enabled() && !config.read_only;
        let mut canvas = ui.interact(
            rect,
            root.with("canvas"),
            if editable {
                Sense::click_and_drag()
            } else {
                Sense::focusable_noninteractive()
            },
        );
        // Retain drawing arrows before egui's next-pass spatial focus navigation.
        // Tab and Escape keep their native focus behavior; read-only releases arrows.
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                canvas.id,
                egui::EventFilter {
                    horizontal_arrows: editable,
                    vertical_arrows: editable,
                    ..Default::default()
                },
            );
        });
        let key = root.with("gesture");
        let mut state = ui
            .ctx()
            .data(|data| data.get_temp::<capture::State>(key))
            .unwrap_or_default();
        let outcome = capture::update(
            ui,
            &canvas,
            config.drawing,
            &mut state,
            capture::Pen {
                brush: config.brush,
                smoothing: config.smoothing,
            },
            editable,
        );
        if outcome.strokes > 0 {
            canvas.mark_changed();
        }
        metadata(
            &canvas,
            config.label.as_ref(),
            config.drawing.strokes().len(),
            config.drawing.point_count(),
            config.read_only,
        );
        paint::show(
            ui,
            outer,
            &canvas,
            config.drawing,
            &state,
            config
                .placeholder
                .as_deref()
                .unwrap_or(if config.read_only {
                    "No drawing"
                } else {
                    "Draw here"
                }),
            editable,
        );
        if editable && canvas.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        let reveal = (canvas.gained_focus() && !state.is_pointer()) || outcome.keyboard_used;
        if reveal && !ui.clip_rect().contains_rect(canvas.rect) {
            canvas.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
        }
        ui.ctx().data_mut(|data| data.insert_temp(key, state));
        let mut actions = outcome.actions;
        let mut action_response = None;
        if config.controls {
            ui.add_space(space::MD);
            if let Some((action, response)) = toolbar::show(ui, root, &config) {
                actions.push(action);
                action_response = Some(response);
            }
        }
        if config.instructions {
            ui.add_space(space::SM);
            ui.add(
                Text::new(if config.read_only {
                    "Read-only drawing. Preview and export remain available."
                } else {
                    keyboard::HELP
                })
                .size(TextSize::Xs)
                .color(TextColor::Muted),
            );
        }
        (
            canvas,
            outcome.strokes,
            actions,
            action_response,
            outcome.error,
        )
    });
    let mut response = result.response;
    response.widget_info(|| {
        WidgetInfo::labeled(
            egui::accesskit::Role::Group,
            ui.is_enabled() && config.enabled,
            config.label.as_ref(),
        )
    });
    if result.inner.1 > 0 {
        response.mark_changed();
    }
    ScribbleResponse {
        response,
        canvas: result.inner.0,
        strokes_added: result.inner.1,
        actions: result.inner.2,
        action_response: result.inner.3,
        error: result.inner.4,
    }
}
fn metadata(canvas: &Response, label: &str, strokes: usize, points: usize, read_only: bool) {
    canvas.widget_info(|| {
        WidgetInfo::labeled(egui::accesskit::Role::Canvas, canvas.enabled(), label)
    });
    canvas.ctx.accesskit_node_builder(canvas.id, |node| {
        node.set_value(format!("{strokes} strokes, {points} points"));
        node.set_description(if read_only {
            "Read-only drawing"
        } else {
            keyboard::HELP
        });
        if read_only {
            node.set_read_only();
        }
    });
}
