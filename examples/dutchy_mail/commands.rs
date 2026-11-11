//! Shared command descriptions and adapters for DutchyMail's surfaces.

use crate::{
    context_menu::ContextMenu,
    icons,
    mailbox::{ARCHIVE, FolderId, MessageId, TRASH},
    phosphor,
};
use dutchy_egui::{
    ButtonVariant, Command, CommandRegistry, ControlSize, Icon, MenuBar, MenuBarItem, MenuBarState,
    MenuItem, MenuItemVariant, Toolbar, ToolbarEvent,
};
use egui::{Context, Key, KeyboardShortcut, Modifiers, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Compose,
    Archive,
    Delete,
    ToggleRead,
    Sync,
    FocusSearch,
}

impl Action {
    pub fn needs_messages(self) -> bool {
        MESSAGE_ACTIONS.contains(&self)
    }
}

/// Global surfaces act on the selection. A context menu captures its target.
pub enum Request {
    Selection(Action),
    Context {
        action: Action,
        folder: FolderId,
        messages: Vec<MessageId>,
    },
}

const MESSAGE_ACTIONS: [Action; 3] = [Action::Archive, Action::Delete, Action::ToggleRead];

#[derive(Clone, Copy)]
pub struct Availability {
    pub folder: FolderId,
    pub has_messages: bool,
    pub syncing: bool,
}

impl Availability {
    fn allows(self, action: Action) -> bool {
        match action {
            Action::Archive => self.has_messages && self.folder != ARCHIVE,
            Action::Delete => self.has_messages && self.folder != TRASH,
            Action::ToggleRead => self.has_messages,
            Action::Sync => !self.syncing,
            Action::Compose | Action::FocusSearch => true,
        }
    }
}

pub struct Commands {
    pub registry: CommandRegistry<Action>,
    pub context: ContextMenu,
    bar: MenuBarState,
}

impl Default for Commands {
    fn default() -> Self {
        let chord = |key| KeyboardShortcut::new(Modifiers::COMMAND, key);
        Self {
            registry: CommandRegistry::new(
                "dutchy-mail",
                [
                    Command::new(Action::Compose, "Compose")
                        .shortcut(chord(Key::N))
                        .allow_in_text(true)
                        .icon(Icon::Plus),
                    Command::new(Action::Archive, "Archive")
                        .shortcut(chord(Key::E))
                        .allow_in_text(true)
                        .icon(Icon::Custom(icons::archive)),
                    Command::new(Action::Delete, "Delete")
                        .shortcut(KeyboardShortcut::new(Modifiers::NONE, Key::Delete))
                        .icon(Icon::Trash),
                    Command::new(Action::ToggleRead, "Read / unread")
                        .icon(Icon::Custom(icons::envelope)),
                    Command::new(Action::Sync, "Sync"),
                    Command::new(Action::FocusSearch, "Search")
                        .shortcut(chord(Key::F))
                        .allow_in_text(true)
                        .icon(Icon::Search),
                ],
            ),
            context: ContextMenu::default(),
            bar: MenuBarState::default(),
        }
    }
}

impl Commands {
    pub fn use_phosphor_icons(&mut self, ctx: &Context) {
        phosphor::install(ctx);
        for (action, icon) in [
            (Action::Compose, Icon::Custom(phosphor::plus)),
            (Action::Archive, Icon::Custom(phosphor::archive)),
            (Action::Delete, Icon::Custom(phosphor::trash)),
            (Action::ToggleRead, Icon::Custom(phosphor::envelope)),
        ] {
            self.registry
                .get_mut(&action)
                .expect("mail command is registered")
                .icon = Some(icon);
        }
    }

    pub fn update(&mut self, availability: Availability) {
        for action in MESSAGE_ACTIONS.into_iter().chain([Action::Sync]) {
            self.registry
                .get_mut(&action)
                .expect("mail command is registered")
                .enabled = availability.allows(action);
        }
    }

    pub fn shortcuts(&self, ctx: &Context) -> Vec<Request> {
        self.registry
            .dispatch(ctx)
            .into_iter()
            .map(|request| Request::Selection(request.value))
            .collect()
    }

    pub fn menu_bar(&mut self, ui: &mut Ui) -> Option<Request> {
        let mailbox = [Action::Compose, Action::Sync, Action::FocusSearch]
            .map(|action| menu_item(&self.registry, ui.ctx(), action));
        let messages = MESSAGE_ACTIONS.map(|action| menu_item(&self.registry, ui.ctx(), action));
        let items = [
            MenuBarItem::new("mailbox", "Mailbox", &mailbox),
            MenuBarItem::new("message", "Message", &messages),
        ];
        MenuBar::new("mail-menus", &mut self.bar, &items)
            .accessible_name("DutchyMail menus")
            .show(ui)
            .action
            .map(Request::Selection)
    }

    pub fn toolbar(&self, ui: &mut Ui) -> Option<Request> {
        let items = [
            Action::Compose,
            Action::Archive,
            Action::Delete,
            Action::ToggleRead,
        ]
        .map(|action| {
            let item = self
                .registry
                .toolbar_item(ui.ctx(), &action)
                .expect("mail command is registered");
            match action {
                Action::Compose => item.variant(ButtonVariant::Primary),
                Action::Archive => item.separator_before(true),
                _ => item,
            }
        });
        let result = Toolbar::new("mail-actions", &items)
            .accessible_name("Message actions")
            .size(ControlSize::Small)
            .wrap(true)
            .show(ui);
        match result.event {
            Some(ToolbarEvent::Activated(action)) => Some(Request::Selection(action)),
            _ => None,
        }
    }
}

pub fn context_items(
    registry: &CommandRegistry<Action>,
    ctx: &Context,
    folder: FolderId,
) -> [MenuItem<'static, Action>; 3] {
    let availability = Availability {
        folder,
        has_messages: true,
        syncing: false,
    };
    MESSAGE_ACTIONS
        .map(|action| menu_item(registry, ctx, action).enabled(availability.allows(action)))
}

fn menu_item(
    registry: &CommandRegistry<Action>,
    ctx: &Context,
    action: Action,
) -> MenuItem<'static, Action> {
    let item = registry
        .menu_item(ctx, &action)
        .expect("mail command is registered");
    if action == Action::Delete {
        item.variant(MenuItemVariant::Destructive)
    } else {
        item
    }
}
