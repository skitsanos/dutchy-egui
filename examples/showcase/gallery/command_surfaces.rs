use dutchy_egui::{
    Icon, MenuBar, MenuBarItem, MenuBarState, MenuEvent, MenuItem, Switch, Text, TextColor,
    TextSize, Theme, Toolbar, ToolbarEvent, ToolbarItem,
    tokens::{border, space},
};
use egui::{Stroke, Ui, pos2, vec2};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Command {
    Save,
    Undo,
    Copy,
    Grid,
    Rulers,
    Export,
    Image,
    Png,
    Svg,
}

pub struct CommandState {
    pub bar: MenuBarState,
    pub grid: bool,
    pub rulers: bool,
    accept: bool,
    saves: usize,
    last: String,
}
impl Default for CommandState {
    fn default() -> Self {
        Self {
            bar: MenuBarState::default(),
            grid: true,
            rulers: false,
            accept: true,
            saves: 0,
            last: "Ready to create".into(),
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut CommandState) {
    let formats = [
        MenuItem::new(Command::Png, "PNG image").shortcut("⇧ ⌘ E"),
        MenuItem::new(Command::Svg, "SVG vector"),
    ];
    let exports = [
        MenuItem::new(Command::Image, "Image format").submenu(&formats),
        MenuItem::new(Command::Copy, "Copy SVG source"),
    ];
    let file = [
        MenuItem::new(Command::Save, "Save document").shortcut("⌘ S"),
        MenuItem::new(Command::Export, "Export artwork").submenu(&exports),
    ];
    let edit = [
        MenuItem::new(Command::Undo, "Undo edit").enabled(false),
        MenuItem::new(Command::Copy, "Copy selection"),
    ];
    let view = [
        MenuItem::new(Command::Grid, "Show grid").checked(state.grid),
        MenuItem::new(Command::Rulers, "Show rulers").checked(state.rulers),
    ];
    let menus = [
        MenuBarItem::new("file", "File", &file),
        MenuBarItem::new("edit", "Edit", &edit),
        MenuBarItem::new("view", "View", &view),
    ];
    if let Some(event) = MenuBar::new("studio-menus", &mut state.bar, &menus)
        .accessible_name("Canal Studio menus")
        .show(ui)
        .event
    {
        apply(state, event);
    }
    ui.add_space(space::SM);
    let tools = [
        ToolbarItem::action(Command::Save, "Save artwork").icon(Icon::Save),
        ToolbarItem::action(Command::Undo, "Undo edit")
            .icon(Icon::Undo)
            .enabled(false),
        ToolbarItem::toggle(Command::Grid, "Show grid", state.grid)
            .icon(Icon::Grid)
            .separator_before(true),
        ToolbarItem::action(Command::Export, "Export artwork")
            .icon(Icon::Download)
            .icon_only(false)
            .menu(&exports),
    ];
    if let Some(event) = Toolbar::new("studio-tools", &tools)
        .accessible_name("Canal Studio tools")
        .show(ui)
        .event
    {
        apply(
            state,
            match event {
                ToolbarEvent::Activated(value) => MenuEvent::Activated(value),
                ToolbarEvent::Toggled { value, pressed } => MenuEvent::Checked {
                    value,
                    checked: pressed,
                },
            },
        );
    }
    ui.add_space(space::LG);
    preview(ui, state);
    ui.add_space(space::MD);
    ui.add(
        Text::new(format!("{} · Saved {} times", state.last, state.saves))
            .size(TextSize::Sm)
            .color(TextColor::Muted),
    );
    ui.add(Switch::new(
        "accept-view",
        &mut state.accept,
        "Apply view changes",
    ));
    ui.add(Text::new("File → Export artwork → Image format. The toolbar opens the same export commands. View checkmarks and the grid tool share your canvas settings.").size(TextSize::Sm).color(TextColor::Muted));
}

fn apply(state: &mut CommandState, event: MenuEvent<Command>) {
    match event {
        MenuEvent::Checked { value, checked } => {
            if state.accept {
                match value {
                    Command::Grid => state.grid = checked,
                    Command::Rulers => state.rulers = checked,
                    _ => {}
                }
                state.last = "View settings updated".into();
            } else {
                state.last = "View change declined".into();
            }
        }
        MenuEvent::Activated(command) => {
            state.last = match command {
                Command::Save => {
                    state.saves += 1;
                    "Artwork saved"
                }
                Command::Png => "PNG export requested",
                Command::Svg => "SVG export requested",
                Command::Copy => "Copy requested",
                _ => "Ready to create",
            }
            .into();
        }
    }
}

fn preview(ui: &mut Ui, state: &CommandState) {
    let p = Theme::from_context(ui.ctx()).palette;
    egui::Frame::NONE
        .fill(p.muted)
        .stroke(Stroke::new(border::STANDARD, p.border))
        .inner_margin(egui::Margin::same(space::XL as i8))
        .show(ui, |ui| {
            ui.set_min_size(vec2(ui.available_width(), space::XXL * 5.0));
            let rect = egui::Rect::from_min_size(
                ui.max_rect().min,
                vec2(ui.available_width(), space::XXL * 5.0),
            );
            if state.grid {
                let painter = ui.painter().with_clip_rect(rect);
                for x in (0..(rect.width() / (space::XL * 2.0)) as usize)
                    .map(|x| rect.left() + x as f32 * space::XL * 2.0)
                {
                    painter.line_segment(
                        [pos2(x, rect.top()), pos2(x, rect.top() + space::XXL * 5.0)],
                        Stroke::new(border::THIN, p.border),
                    );
                }
            }
            if state.rulers {
                ui.painter().line_segment(
                    [rect.min, rect.min + vec2(rect.width(), 0.0)],
                    Stroke::new(border::ACCENT, p.primary),
                );
            }
            ui.add_space(space::LG);
            ui.add(Text::label("Canal Studio").color(TextColor::Primary));
            ui.add(Text::heading("Make something sharp.").size(TextSize::Xl3));
            ui.add(Text::new("Your commands. One connected workspace."));
            ui.add_space(space::LG);
        });
}
