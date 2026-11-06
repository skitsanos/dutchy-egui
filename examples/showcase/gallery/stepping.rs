use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, Field, Input, Step, Stepper, StepperState,
    StepperVariant, Text, TextColor,
    tokens::space,
    validation::{FieldValidation, Rule, ValidationTrigger},
};
use egui::Ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    Account,
    Workspace,
    Review,
}
pub struct SteppingState {
    pub workflow: StepperState<Stage>,
    pub email: String,
    pub workspace: String,
    pub submissions: usize,
    email_check: FieldValidation,
    workspace_check: FieldValidation,
    focus_pending: bool,
}
impl Default for SteppingState {
    fn default() -> Self {
        Self {
            workflow: StepperState::Current(Stage::Account),
            email: String::new(),
            workspace: String::new(),
            submissions: 0,
            email_check: FieldValidation::default(),
            workspace_check: FieldValidation::default(),
            focus_pending: false,
        }
    }
}
impl SteppingState {
    pub fn position(&self) -> usize {
        match self.workflow {
            StepperState::Current(Stage::Account) => 1,
            StepperState::Current(Stage::Workspace) => 2,
            StepperState::Current(Stage::Review) => 3,
            _ => 0,
        }
    }
    fn move_to(&mut self, stage: Stage) {
        self.workflow = StepperState::Current(stage);
        self.focus_pending = true;
    }
    fn advance(&mut self) {
        match self.workflow {
            StepperState::Current(Stage::Account) => {
                if self.email_check.validate(&self.email, &email_rules()) {
                    self.move_to(Stage::Workspace);
                } else {
                    self.focus_pending = true;
                }
            }
            StepperState::Current(Stage::Workspace) => {
                if self
                    .workspace_check
                    .validate(&self.workspace, &workspace_rules())
                {
                    self.move_to(Stage::Review);
                } else {
                    self.focus_pending = true;
                }
            }
            StepperState::Current(Stage::Review) => {
                self.workflow = StepperState::Complete;
                self.submissions += 1;
                self.focus_pending = true;
            }
            _ => {}
        }
    }
    fn focus(&mut self, response: &egui::Response) {
        if self.focus_pending {
            response.request_focus();
            self.focus_pending = false;
        }
    }
}
fn email_rules() -> [Rule<'static>; 2] {
    [
        Rule::required("Enter an email before continuing."),
        Rule::email("Enter a valid email address."),
    ]
}
fn workspace_rules() -> [Rule<'static>; 1] {
    [Rule::required("Enter a workspace name before continuing.")]
}

pub fn show(ui: &mut Ui, state: &mut SteppingState) {
    ui.add(Text::heading("A clear path forward."));
    ui.add(Text::new(
        "Track progress. Keep decisions and data in the application.",
    ));
    section(ui, "01", "Host-owned setup workflow", |ui| {
        wizard(ui, state)
    });
    section(ui, "02", "Vertical / supporting descriptions", |ui| {
        let steps = reference_steps();
        Stepper::new("vertical-reference", &StepperState::Current(2), &steps)
            .variant(StepperVariant::Vertical)
            .accessible_name("Detailed setup")
            .show(ui);
    });
    section(ui, "03", "Compact / scrollable sequence", |ui| {
        let steps = reference_steps();
        Stepper::new("compact-reference", &StepperState::Current(3), &steps)
            .variant(StepperVariant::Compact)
            .accessible_name("Compact setup")
            .show(ui);
        ui.add_space(space::SM);
        ui.add(
            Text::new(
                "Hover a number for its label and status. Narrow sequences scroll horizontally.",
            )
            .color(TextColor::Muted),
        );
    });
    section(ui, "04", "Position progress / explicit completion", |ui| {
        let steps = reference_steps();
        Stepper::new("progress-reference", &StepperState::Current(2), &steps)
            .variant(StepperVariant::Progress)
            .accessible_name("Setup position")
            .show(ui);
        ui.add_space(space::LG);
        Stepper::new("complete-reference", &StepperState::Complete, &steps)
            .variant(StepperVariant::Progress)
            .accessible_name("Completed setup")
            .show(ui);
        ui.add_space(space::SM);
        ui.add(Text::new("The percentage measures position in the sequence. Only the host marks the workflow complete.").color(TextColor::Muted));
    });
}
fn wizard(ui: &mut Ui, state: &mut SteppingState) {
    let position = state.position();
    let steps = [
        Step::new(Stage::Account, "Account").navigable(position > 1),
        Step::new(Stage::Workspace, "Workspace").navigable(position > 2),
        Step::new(Stage::Review, "Review"),
    ];
    let result = Stepper::new("setup-wizard", &state.workflow, &steps)
        .accessible_name("Workspace setup")
        .show(ui);
    if let Some(stage) = result.requested_step {
        state.move_to(stage);
    }
    ui.add_space(space::LG);
    match state.workflow {
        StepperState::Current(Stage::Account) => {
            let response = Field::new("Setup email")
                .required(true)
                .validation(state.email_check.state())
                .show(ui, |ui| {
                    ui.add(
                        Input::new("setup-email", &mut state.email).placeholder("you@example.test"),
                    )
                });
            state.email_check.update(
                &response,
                &state.email,
                ValidationTrigger::OnSubmit,
                &email_rules(),
            );
            state.focus(&response);
        }
        StepperState::Current(Stage::Workspace) => {
            let response = Field::new("Setup workspace name")
                .required(true)
                .validation(state.workspace_check.state())
                .show(ui, |ui| {
                    ui.add(
                        Input::new("setup-workspace", &mut state.workspace)
                            .placeholder("Design studio"),
                    )
                });
            state.workspace_check.update(
                &response,
                &state.workspace,
                ValidationTrigger::OnSubmit,
                &workspace_rules(),
            );
            state.focus(&response);
        }
        StepperState::Current(Stage::Review) => {
            ui.add(Text::label("Review workspace"));
            ui.add(Text::new(format!("{} · {}", state.workspace, state.email)));
        }
        StepperState::Complete => {
            Alert::new(
                "setup-complete",
                "The host applied the completion state. No external service was called.",
            )
            .variant(AlertVariant::Success)
            .title("Workspace setup complete")
            .show(ui);
            let restart = ui.add(Button::new("Start setup again").variant(ButtonVariant::Outline));
            state.focus(&restart);
            if restart.clicked() {
                state.email_check.clear();
                state.workspace_check.clear();
                state.move_to(Stage::Account);
            }
        }
        _ => {}
    }
    if matches!(state.workflow, StepperState::Current(_)) {
        let position = state.position();
        ui.add_space(space::MD);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(
                    Button::new("Back")
                        .variant(ButtonVariant::Ghost)
                        .enabled(position > 1),
                )
                .clicked()
            {
                state.move_to(if position == 3 {
                    Stage::Workspace
                } else {
                    Stage::Account
                });
            }
            let next = ui.add(Button::new(
                if state.workflow == StepperState::Current(Stage::Review) {
                    "Create workspace"
                } else {
                    "Continue setup"
                },
            ));
            if state.workflow == StepperState::Current(Stage::Review) {
                state.focus(&next);
            }
            if next.clicked() {
                state.advance();
                ui.ctx().request_repaint();
            }
        });
    }
    ui.add_space(space::SM);
    ui.add(Text::label(format!("Host submissions: {}", state.submissions)).color(TextColor::Muted));
}
fn reference_steps() -> [Step<'static, u8>; 4] {
    [
        Step::new(1, "Account")
            .description("Choose your account and retain its details in the host."),
        Step::new(2, "Workspace details")
            .description("Enter a name, then review the information before continuing."),
        Step::new(3, "Review")
            .description("The host validates the form and decides whether navigation is allowed."),
        Step::new(4, "Confirmation").description(
            "Completion is explicit. Merely viewing the last step does not submit work.",
        ),
    ]
}
