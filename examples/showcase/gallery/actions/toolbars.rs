use super::ActionsState;
use crate::gallery::section;
use dutchy_egui::{
    Icon, Text, TextColor, Theme, Toolbar, ToolbarEvent, ToolbarItem, ToolbarOrientation, Weight,
};

#[derive(Clone, Copy, Debug, Hash)]
enum Tool {
    Bold,
    Italic,
    Underline,
    Left,
    Center,
    Right,
    Save,
    Undo,
    Export,
}

pub fn show(ui: &mut egui::Ui, state: &mut ActionsState) {
    section(ui, "01", "Host-owned formatting / one tab stop", |ui| {
        let items = formatting(state);
        let response = Toolbar::new("editor-tools", &items)
            .accessible_name("Document formatting")
            .show(ui);
        if let Some(event) = response.event {
            apply(state, event);
        }
        let theme = Theme::from_context(ui.ctx());
        let text = Text::new("Sharp tools. Your state.")
            .weight(if state.bold {
                Weight::Bold
            } else {
                Weight::Regular
            })
            .italic(state.italic);
        let mut job = text.layout_job(&theme, ui.available_width());
        job.halign = match state.alignment {
            1 => egui::Align::Center,
            2 => egui::Align::RIGHT,
            _ => egui::Align::LEFT,
        };
        if state.underline {
            for section in &mut job.sections {
                section.format.underline =
                    egui::Stroke::new(dutchy_egui::tokens::border::THIN, theme.palette.foreground);
            }
        }
        let alignment = job.halign;
        ui.with_layout(egui::Layout::top_down(alignment), |ui| {
            ui.add(egui::Label::new(job).halign(alignment));
        });
        ui.add(Text::label(format!("Save actions: {}", state.saves)).color(TextColor::Muted));
    });
    section(ui, "02", "Overflow, wrapping and group boundaries", |ui| {
        let items = formatting(state);
        ui.add(Text::label("Scroll / focus reveals the active tool").color(TextColor::Muted));
        if let Some(event) = Toolbar::new("overflow-tools", &items)
            .accessible_name("Compact formatting")
            .width(220.0)
            .show(ui)
            .event
        {
            apply(state, event);
        }
        ui.add(Text::label("Wrap / linear arrow order").color(TextColor::Muted));
        if let Some(event) = Toolbar::new("wrapped-tools", &items)
            .accessible_name("Wrapped formatting")
            .width(220.0)
            .wrap(true)
            .show(ui)
            .event
        {
            apply(state, event);
        }
    });
    section(ui, "03", "Vertical / Up and Down", |ui| {
        let items = [
            ToolbarItem::action(Tool::Save, "Save document").icon(Icon::Save),
            ToolbarItem::action(Tool::Undo, "Undo change")
                .icon(Icon::Undo)
                .enabled(false),
            ToolbarItem::action(Tool::Export, "Exporting document")
                .icon(Icon::Download)
                .loading(true),
        ];
        if let Some(event) = Toolbar::new("vertical-tools", &items)
            .accessible_name("Document actions")
            .orientation(ToolbarOrientation::Vertical)
            .show(ui)
            .event
        {
            apply(state, event);
        }
        ui.add(
            Text::new("Unavailable and busy tools stay visible. Arrow navigation skips them.")
                .color(TextColor::Muted),
        );
    });
}

fn formatting(state: &ActionsState) -> [ToolbarItem<'static, Tool>; 7] {
    [
        ToolbarItem::toggle(Tool::Bold, "Bold", state.bold).icon(Icon::Bold),
        ToolbarItem::toggle(Tool::Italic, "Italic", state.italic).icon(Icon::Italic),
        ToolbarItem::toggle(Tool::Underline, "Underline", state.underline).icon(Icon::Underline),
        ToolbarItem::toggle(Tool::Left, "Align left", state.alignment == 0)
            .icon(Icon::AlignLeft)
            .separator_before(true),
        ToolbarItem::toggle(Tool::Center, "Align center", state.alignment == 1)
            .icon(Icon::AlignCenter),
        ToolbarItem::toggle(Tool::Right, "Align right", state.alignment == 2)
            .icon(Icon::AlignRight),
        ToolbarItem::action(Tool::Save, "Save document")
            .icon(Icon::Save)
            .separator_before(true),
    ]
}

fn apply(state: &mut ActionsState, event: ToolbarEvent<Tool>) {
    match event {
        ToolbarEvent::Activated(Tool::Save) => state.saves += 1,
        ToolbarEvent::Toggled { value, pressed } => match value {
            Tool::Bold => state.bold = pressed,
            Tool::Italic => state.italic = pressed,
            Tool::Underline => state.underline = pressed,
            Tool::Left => state.alignment = 0,
            Tool::Center => state.alignment = 1,
            Tool::Right => state.alignment = 2,
            _ => {}
        },
        _ => {}
    }
}
