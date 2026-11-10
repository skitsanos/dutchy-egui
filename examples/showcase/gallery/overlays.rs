use super::section;
use dutchy_egui::{
    Button, ButtonVariant, Checkbox, Drawer, DrawerSide, DrawerWidth, Field, Input, Modal,
    ModalWidth, OverlayOptions, Text, TextColor,
    tokens::space,
    validation::{FieldValidation, Rule},
};
use egui::{Context, Id, Ui};
pub struct OverlayState {
    pub confirm_open: bool,
    pub edit_open: bool,
    pub drawer_open: bool,
    pub side: DrawerSide,
    pub draft_name: String,
    pub saved_name: String,
    pub archives: usize,
    pub preference_saves: usize,
    pub draft_workspace: String,
    pub notifications: bool,
    pub backdrop_dismissal: bool,
    name_check: FieldValidation,
    focus_invalid: bool,
    modal_opener: Option<Id>,
    drawer_opener: Option<Id>,
}
impl Default for OverlayState {
    fn default() -> Self {
        Self {
            confirm_open: false,
            edit_open: false,
            drawer_open: false,
            side: DrawerSide::Right,
            draft_name: "Ada Lovelace".into(),
            saved_name: "Ada Lovelace".into(),
            archives: 0,
            preference_saves: 0,
            draft_workspace: "Design studio".into(),
            notifications: true,
            backdrop_dismissal: true,
            name_check: FieldValidation::default(),
            focus_invalid: false,
            modal_opener: None,
            drawer_opener: None,
        }
    }
}
impl OverlayState {
    pub fn modals(&mut self, ui: &mut Ui) {
        ui.add(Text::heading("One task. Clear focus."));
        ui.add(Text::new(
            "Blocking dialogs keep decisions and editable data in the application.",
        ));
        section(ui, "01", "Confirmation / explicit host action", |ui| {
            ui.add(Text::new("Archive a sample project after confirmation. Dismissing the dialog performs no action."));
            ui.add_space(space::MD);
            let trigger = ui.add(Button::new("Open confirmation").variant(ButtonVariant::Outline));
            if trigger.clicked() {
                self.confirm_open = true;
                self.modal_opener = Some(trigger.id);
            }
            ui.add(
                Text::label(format!("Host archive actions: {}", self.archives))
                    .color(TextColor::Muted),
            );
        });
        section(ui, "02", "Content / host validation", |ui| {
            ui.add(Text::new(format!("Saved profile: {}", self.saved_name)));
            ui.add_space(space::MD);
            let trigger = ui.add(Button::new("Edit profile"));
            if trigger.clicked() {
                self.edit_open = true;
                self.modal_opener = Some(trigger.id);
            }
            ui.add_space(space::SM);
            ui.add(Text::new("Drafts survive dismissal. Save validates the field before updating the host's profile.").color(TextColor::Muted));
        });
    }
    pub fn drawers(&mut self, ui: &mut Ui) {
        ui.add(Text::heading("Context at the edge."));
        ui.add(Text::new(
            "Full-height overlays preserve the workspace beneath a blocking backdrop.",
        ));
        section(ui, "01", "Left and right / retained preferences", |ui| {
            ui.horizontal_wrapped(|ui| {
                for (label, side) in [
                    ("Open left drawer", DrawerSide::Left),
                    ("Open right drawer", DrawerSide::Right),
                ] {
                    let trigger = ui.add(Button::new(label).variant(ButtonVariant::Outline));
                    if trigger.clicked() {
                        self.drawer_open = true;
                        self.side = side;
                        self.drawer_opener = Some(trigger.id);
                    }
                }
            });
            ui.add_space(space::MD);
            ui.add(Checkbox::new(
                "dismiss-backdrop",
                &mut self.backdrop_dismissal,
                "Dismiss drawer on backdrop",
            ));
            ui.add(
                Text::label(format!("Host preference saves: {}", self.preference_saves))
                    .color(TextColor::Muted),
            );
        });
        section(ui, "02", "Native input and reduced motion", |ui| {
            ui.add(Text::new("Tab remains within the active overlay. Escape closes a child popup first, then the overlay. Closing restores the opener's focus."));
            ui.add_space(space::SM);
            ui.add(Text::new("Drawers slide in using the shared motion token. Reduce motion makes placement immediate. The header remains visible while long content and its footer scroll.").color(TextColor::Muted));
        });
    }
    pub fn render(&mut self, ctx: &Context) {
        self.confirmation(ctx);
        self.profile(ctx);
        self.preferences(ctx);
    }
    fn confirmation(&mut self, ctx: &Context) {
        let result = Modal::new(
            "archive-confirmation",
            &mut self.confirm_open,
            "Archive project?",
        )
        .width(ModalWidth::Medium)
        .options(OverlayOptions::default().return_focus(self.modal_opener))
        .show_with_footer(
            ctx,
            |ui| {
                ui.add(Text::new(
                    "Archive the sample project? The showcase only increments its local counter.",
                ));
            },
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(Button::new("Cancel archive").variant(ButtonVariant::Outline))
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.add(Button::new("Archive project").variant(ButtonVariant::Destructive))
                        .clicked()
                })
                .inner
            },
        );
        if result.is_some_and(|result| result.inner.1) {
            self.archives += 1;
            self.confirm_open = false;
        }
    }
    fn profile(&mut self, ctx: &Context) {
        let rules = [Rule::required("Enter a profile name before saving.")];
        let result=Modal::new("edit-profile",&mut self.edit_open,"Edit profile")
            .width(ModalWidth::Medium)
            .options(OverlayOptions::default().return_focus(self.modal_opener))
            .show_with_footer(ctx,|ui| {
                let response=Field::new("Profile name").required(true).validation(self.name_check.state())
                    .show(ui,|ui|ui.add(Input::new("profile-name",&mut self.draft_name)));
                if self.focus_invalid {response.request_focus();self.focus_invalid=false;}
                ui.add_space(space::MD);
                ui.add(Text::new("The application validates and saves. Closing keeps the draft available for the next edit.").color(TextColor::Muted));
            },|ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.add(Button::new("Cancel edit").variant(ButtonVariant::Outline)).clicked() {ui.close();}
                    ui.add(Button::new("Save profile")).clicked()
                }).inner
            });
        if result.is_some_and(|result| result.inner.1) {
            if self.name_check.validate(&self.draft_name, &rules) {
                self.saved_name.clone_from(&self.draft_name);
                self.edit_open = false;
            } else {
                self.focus_invalid = true;
                ctx.request_repaint();
            }
        }
    }
    fn preferences(&mut self, ctx: &Context) {
        let result = Drawer::new(
            "workspace-preferences",
            &mut self.drawer_open,
            "Workspace preferences",
        )
        .side(self.side)
        .width(DrawerWidth::Medium)
        .options(
            OverlayOptions::default()
                .dismiss_on_backdrop(self.backdrop_dismissal)
                .return_focus(self.drawer_opener),
        )
        .show_with_footer(
            ctx,
            |ui| {
                Field::new("Workspace name").show(ui, |ui| {
                    ui.add(Input::new("workspace-name", &mut self.draft_workspace))
                });
                ui.add(Checkbox::new(
                    "notifications",
                    &mut self.notifications,
                    "Email notifications",
                ));
                ui.add_space(space::LG);
                ui.add(Text::label("Workspace activity"));
                for index in 1..=12 {
                    ui.add_space(space::SM);
                    ui.add(Text::new(format!(
                        "Activity {index:02} · Sample workspace update"
                    )));
                }
            },
            |ui| {
                ui.add(Button::new("Apply preferences").full_width(true))
                    .clicked()
            },
        );
        if result.is_some_and(|result| result.inner.1) {
            self.preference_saves += 1;
            self.drawer_open = false;
        }
    }
}
