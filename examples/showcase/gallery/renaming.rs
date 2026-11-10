use dutchy_egui::{
    Button, ButtonVariant, Card, CardPadding, CardVariant, ControlSize, InlineEdit,
    InlineEditEvent, InlineEditState, Switch, Text, TextColor, TextSize, tokens::space,
};
use egui::{
    Ui,
    accesskit::{Live, Role},
};

struct Name {
    id: u64,
    value: String,
    edit: InlineEditState,
    error: bool,
}
impl Name {
    fn new(id: u64, value: &str) -> Self {
        Self {
            id,
            value: value.into(),
            edit: InlineEditState::default(),
            error: false,
        }
    }
    fn show(
        &mut self,
        ui: &mut Ui,
        name: &str,
        size: ControlSize,
        locked: bool,
        disabled: bool,
    ) -> Option<String> {
        let result = InlineEdit::new(self.id, &self.value, &mut self.edit)
            .accessible_name(name)
            .size(size)
            .char_limit(60)
            .read_only(locked)
            .enabled(!disabled)
            .show(ui);
        if result.response.changed() {
            self.error = false;
        }
        let status = match result.event {
            Some(InlineEditEvent::Commit(value)) if value.trim().is_empty() => {
                self.edit.begin(value);
                self.error = true;
                None
            }
            Some(InlineEditEvent::Commit(value)) => {
                self.error = false;
                self.value = value;
                Some(format!("Renamed to {}", self.value))
            }
            Some(InlineEditEvent::Cancelled) => {
                self.error = false;
                Some("Name unchanged.".into())
            }
            _ => None,
        };
        if self.error {
            let message = ui.add(
                Text::new("Use at least one visible character.")
                    .size(TextSize::Sm)
                    .color(TextColor::Destructive),
            );
            message.ctx.accesskit_node_builder(message.id, |node| {
                node.set_role(Role::Alert);
                node.set_live(Live::Assertive);
                node.set_label("Use at least one visible character.");
            });
        }
        status
    }
}

pub struct RenamingState {
    notebook: Name,
    documents: [Name; 3],
    locked: bool,
    disabled: bool,
    status: String,
}
impl Default for RenamingState {
    fn default() -> Self {
        Self {
            notebook: Name::new(0, "Canal Studio"),
            documents: [
                Name::new(1, "Design notes.md"),
                Name::new(2, "Release checklist.md"),
                Name::new(3, "Ideas.md"),
            ],
            locked: false,
            disabled: false,
            status: "Click a name to make it yours.".into(),
        }
    }
}
pub fn show(ui: &mut Ui, state: &mut RenamingState) {
    ui.add(Text::heading("Make it yours.").size(TextSize::Xl5));
    ui.add(Text::new("A clearer name for the next big idea.").color(TextColor::Muted));
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        ui.add(Switch::new("rename-lock", &mut state.locked, "Lock names"));
        ui.add(Switch::new(
            "rename-disabled",
            &mut state.disabled,
            "Disabled",
        ));
    });
    ui.add_space(space::LG);
    Card::new()
        .variant(CardVariant::Accent)
        .padding(CardPadding::Small)
        .show(ui, |ui| {
            ui.add(Text::label("01 / Workspace").color(TextColor::Primary));
            if let Some(status) = state.notebook.show(
                ui,
                "Workspace name",
                ControlSize::Large,
                state.locked,
                state.disabled,
            ) {
                state.status = status;
            }
            ui.add(
                Text::new("A place for small ideas and ambitious work.").color(TextColor::Muted),
            );
        });
    ui.add_space(space::LG);
    Card::new()
        .variant(CardVariant::Bordered)
        .padding(CardPadding::Small)
        .show(ui, |ui| {
            ui.add(Text::label("02 / Notebook"));
            for (index, document) in state.documents.iter_mut().enumerate() {
                if index > 0 {
                    ui.add_space(space::SM);
                }
                if let Some(status) = document.show(
                    ui,
                    &format!("Document {} name", document.id),
                    ControlSize::Medium,
                    state.locked,
                    state.disabled,
                ) {
                    state.status = status;
                }
            }
            ui.add_space(space::LG);
            if ui
                .add(
                    Button::new("Reverse order")
                        .size(ControlSize::Small)
                        .variant(ButtonVariant::Outline),
                )
                .clicked()
            {
                state.documents.reverse();
            }
        });
    ui.add_space(space::LG);
    let status = ui.add(
        Text::new(&state.status)
            .size(TextSize::Sm)
            .color(TextColor::Muted),
    );
    status.ctx.accesskit_node_builder(status.id, |node| {
        node.set_role(Role::Status);
        node.set_live(Live::Polite);
    });
    ui.add(
        Text::new("Enter saves. Escape cancels. You can move away and return to your draft.")
            .size(TextSize::Sm)
            .color(TextColor::Muted),
    );
}
