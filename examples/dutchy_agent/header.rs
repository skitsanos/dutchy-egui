//! A compact identity line and right-aligned, keyboard-named application actions.
use crate::{app::App, support::shell};
use dutchy_egui::{
    Badge, BadgeVariant, Button, ButtonVariant, ControlSize, Icon, IconButton, Text, TextSize,
    tokens::{border, space},
};
use egui::{Align, Color32, Layout, Painter, Rect, Stroke, Ui, pos2, vec2};

pub(crate) fn show(ui: &mut Ui, app: &mut App) {
    shell::header(ui, "agent-header", |ui| {
        let compact = ui.available_width() < 640.0;
        let size = ControlSize::Small;
        let new_width = if compact {
            size.height()
        } else {
            shell::button_width(ui, "New chat", size)
        };
        let actions_width = new_width + space::SM + size.height();
        let brand_width = (ui.available_width() - actions_width - space::LG).max(0.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::LG;
            ui.allocate_ui_with_layout(
                vec2(brand_width, size.height()),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_min_width(brand_width);
                    ui.spacing_mut().item_spacing.x = space::MD;
                    ui.add(Text::heading("DutchyAgent").size(if compact {
                        TextSize::Xl
                    } else {
                        TextSize::Xl2
                    }));
                    ui.add(Badge::new(app.config.provider.label()).variant(BadgeVariant::Muted));
                },
            );
            ui.allocate_ui_with_layout(
                vec2(actions_width, size.height()),
                Layout::left_to_right(Align::Center),
                |ui| actions(ui, app, compact),
            );
        });
    });
}

fn actions(ui: &mut Ui, app: &mut App, compact: bool) {
    ui.spacing_mut().item_spacing.x = space::SM;
    let size = ControlSize::Small;
    let new_chat = if compact {
        ui.add(
            IconButton::new("new-chat", "Start new conversation", Icon::Plus)
                .variant(ButtonVariant::Primary)
                .size(size),
        )
    } else {
        ui.add(Button::new("New chat").size(size))
            .accessible_name("Start new conversation")
    };
    if new_chat.clicked() {
        app.new_chat();
    }
    let settings =
        ui.add(IconButton::new("chat-settings", "Chat settings", Icon::Custom(sliders)).size(size));
    if settings.clicked() {
        app.settings_open = true;
        app.settings_return = Some(settings.id);
    }
}

fn sliders(painter: &Painter, rect: Rect, ink: Color32) {
    let stroke = Stroke::new(border::STANDARD, ink);
    for (y, knob) in [(0.22, 0.32), (0.5, 0.68), (0.78, 0.42)] {
        let point = |x, y| {
            pos2(
                rect.left() + x * rect.width(),
                rect.top() + y * rect.height(),
            )
        };
        painter.line_segment([point(0.08, y), point(0.92, y)], stroke);
        painter.line_segment([point(knob, y - 0.12), point(knob, y + 0.12)], stroke);
    }
}
