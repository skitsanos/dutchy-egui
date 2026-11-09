use super::Action;
use dutchy_egui::{Command, CommandRegistry, Icon, MenuItem};
use egui::{Context, Key, KeyboardShortcut, Modifiers};
pub(super) fn registry() -> CommandRegistry<Action> {
    let chord = |key| KeyboardShortcut::new(Modifiers::COMMAND, key);
    CommandRegistry::new(
        "canal-studio",
        [
            Command::new(Action::Save, "Save document")
                .shortcut(chord(Key::S))
                .allow_in_text(true)
                .icon(Icon::Save),
            Command::new(Action::Undo, "Undo edit")
                .enabled(false)
                .icon(Icon::Undo),
            Command::new(Action::Copy, "Copy selection").icon(Icon::Copy),
            Command::new(Action::CopySvg, "Copy SVG source"),
            Command::new(Action::Grid, "Show grid")
                .checked(true)
                .shortcut(KeyboardShortcut::new(
                    Modifiers::COMMAND | Modifiers::SHIFT,
                    Key::G,
                ))
                .icon(Icon::Grid),
            Command::new(Action::Rulers, "Show rulers").checked(false),
            Command::new(Action::Png, "PNG image")
                .shortcut(KeyboardShortcut::new(
                    Modifiers::COMMAND | Modifiers::SHIFT,
                    Key::E,
                ))
                .icon(Icon::Download),
            Command::new(Action::Svg, "SVG vector"),
            Command::new(Action::Palette, "Command palette")
                .shortcut(chord(Key::K))
                .allow_in_text(true)
                .allow_in_overlays(true)
                .icon(Icon::Search),
        ],
    )
}
pub(super) struct Tree<'a> {
    pub file: &'a [MenuItem<'a, Action>],
    pub edit: &'a [MenuItem<'a, Action>],
    pub view: &'a [MenuItem<'a, Action>],
    pub exports: &'a [MenuItem<'a, Action>],
    pub palette: &'a [MenuItem<'a, Action>],
}
pub(super) fn with_tree<R>(
    registry: &CommandRegistry<Action>,
    ctx: &Context,
    render: impl FnOnce(Tree<'_>) -> R,
) -> R {
    let item = |key| {
        registry
            .menu_item(ctx, &key)
            .expect("catalog action exists")
    };
    let formats = [item(Action::Png), item(Action::Svg)];
    let exports = [
        MenuItem::new(Action::Image, "Image format").submenu(&formats),
        item(Action::CopySvg),
    ];
    let file = [
        item(Action::Save),
        MenuItem::new(Action::Export, "Export artwork").submenu(&exports),
    ];
    let edit = [item(Action::Undo), item(Action::Copy)];
    let view = [item(Action::Grid), item(Action::Rulers)];
    let roots = [
        MenuItem::new(Action::File, "File").submenu(&file),
        MenuItem::new(Action::Edit, "Edit").submenu(&edit),
        MenuItem::new(Action::View, "View").submenu(&view),
    ];
    render(Tree {
        file: &file,
        edit: &edit,
        view: &view,
        exports: &exports,
        palette: &roots,
    })
}
