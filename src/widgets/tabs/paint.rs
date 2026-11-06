use super::{Tab, TabsVariant, focus};
use crate::{
    Text, TextColor, TextSize, Theme, Tracking, Weight, tokens::disclosure as geometry,
    widgets::close,
};
use egui::{Id, Rect, Response, Sense, Ui, WidgetInfo, pos2, vec2};

fn label(ui: &Ui, text: &str, color: TextColor) -> std::sync::Arc<egui::Galley> {
    let job = Text::new(text.replace('\n', " "))
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .tracking(Tracking::Wide)
        .uppercase(true)
        .color(color)
        .layout_job(&Theme::from_context(ui.ctx()), f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

pub(super) fn width<T>(ui: &Ui, tab: &Tab<'_, T>) -> f32 {
    label(ui, &tab.label, TextColor::Foreground).size().x
        + geometry::PADDING_X * 2.0
        + if tab.closable {
            geometry::CLOSE_SIZE + geometry::CLOSE_GAP
        } else {
            0.0
        }
}

pub(super) fn tab<T>(
    ui: &mut Ui,
    root: Id,
    item: &Tab<'_, T>,
    selected: bool,
    tab_stop: bool,
    width: f32,
    variant: TabsVariant,
) -> (Response, Option<Response>) {
    let p = Theme::from_context(ui.ctx()).palette;
    let height = if variant == TabsVariant::Underline {
        geometry::TAB_HEIGHT
    } else {
        geometry::SEGMENT_HEIGHT
    };
    let (_, rect) = ui.allocate_space(vec2(width, height));
    let text_width = label(ui, &item.label, TextColor::Foreground).size().x;
    let content_width = text_width
        + if item.closable {
            geometry::CLOSE_SIZE + geometry::CLOSE_GAP
        } else {
            0.0
        };
    let close_rect = Rect::from_center_size(
        pos2(
            rect.center().x + content_width / 2.0 - geometry::CLOSE_SIZE / 2.0,
            rect.center().y,
        ),
        vec2(geometry::CLOSE_SIZE, geometry::CLOSE_SIZE),
    );
    let sense = if tab_stop {
        Sense::click()
    } else {
        Sense::CLICK
    };
    let response = ui.interact(rect, focus::trigger(root, item.id), sense);
    response.widget_info(|| {
        WidgetInfo::selected(
            egui::accesskit::Role::Tab,
            ui.is_enabled(),
            selected,
            item.label.to_uppercase(),
        )
    });
    let close = item.closable.then(|| {
        let response = ui.interact(close_rect, focus::close(root, item.id), sense);
        response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Button,
                ui.is_enabled(),
                format!("Close {}", item.label),
            )
        });
        response
    });
    if !ui.is_rect_visible(rect) {
        return (response, close);
    }
    if variant == TabsVariant::Segmented && selected {
        ui.painter().rect_filled(rect, 0, p.background);
    }
    let color = if selected || response.hovered() {
        TextColor::Foreground
    } else {
        TextColor::Muted
    };
    let galley = label(ui, &item.label, color);
    let content_width = galley.size().x
        + if item.closable {
            geometry::CLOSE_SIZE + geometry::CLOSE_GAP
        } else {
            0.0
        };
    ui.painter().galley(
        pos2(
            rect.center().x - content_width / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        p.foreground,
    );
    if variant == TabsVariant::Underline && selected {
        let mut line = rect;
        line.min.y = line.max.y - geometry::TAB_UNDERLINE;
        ui.painter().rect_filled(line, 0, p.primary);
    }
    if let Some(close) = &close {
        close::paint(
            ui,
            close,
            geometry::CLOSE_SIZE - geometry::CLOSE_INSET * 2.0,
            close::Colors {
                idle: p.muted_foreground,
                foreground: p.foreground,
                focus: p.primary,
            },
        );
    }
    (response, close)
}
