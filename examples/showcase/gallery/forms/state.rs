use dutchy_egui::validation::{FieldValidation, Rule, ValidationTicket, ValidationTrigger};

pub struct FormState {
    pub name: String,
    pub email: String,
    pub password: String,
    pub notes: String,
    pub role: Option<&'static str>,
    pub digest: Option<u8>,
    pub updates: bool,
    pub reveal_password: bool,
    pub trigger: Option<ValidationTrigger>,
    pub name_check: FieldValidation,
    pub email_check: FieldValidation,
    pub password_check: FieldValidation,
    pub notes_check: FieldValidation,
    pub role_check: FieldValidation,
    pub submitted: bool,
    pub reference: String,
    pending: Option<DemoCheck>,
}

struct DemoCheck {
    ticket: ValidationTicket,
    completes_at: f64,
    email: String,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            password: String::new(),
            notes: String::new(),
            role: None,
            digest: Some(0),
            updates: false,
            reveal_password: false,
            trigger: Some(ValidationTrigger::OnBlur),
            name_check: FieldValidation::default(),
            email_check: FieldValidation::default(),
            password_check: FieldValidation::default(),
            notes_check: FieldValidation::default(),
            role_check: FieldValidation::default(),
            submitted: false,
            reference: "WORKSPACE-001".into(),
            pending: None,
        }
    }
}

impl FormState {
    pub fn is_valid(&self) -> bool {
        [
            &self.name_check,
            &self.email_check,
            &self.password_check,
            &self.notes_check,
            &self.role_check,
        ]
        .into_iter()
        .all(FieldValidation::is_valid)
    }

    pub fn validate(&mut self) -> bool {
        self.submitted = [
            self.name_check.validate(&self.name, &name_rules()),
            self.email_check.validate(&self.email, &email_rules()),
            self.password_check
                .validate(&self.password, &password_rules()),
            self.notes_check.validate(&self.notes, &notes_rules()),
            self.role_check.validate(
                if self.role.is_some() { "chosen" } else { "" },
                &role_rules(),
            ),
        ]
        .into_iter()
        .all(|valid| valid);
        self.submitted
    }

    pub fn begin_demo_check(&mut self, ctx: &egui::Context) {
        if !self.email_check.validate(&self.email, &email_rules()) {
            return;
        }
        self.pending = Some(DemoCheck {
            ticket: self.email_check.begin_async(),
            completes_at: ctx.input(|input| input.time) + 1.0,
            email: self.email.clone(),
        });
        ctx.request_repaint();
    }

    pub fn tick(&mut self, ctx: &egui::Context) {
        let Some(check) = &self.pending else {
            return;
        };
        let remaining = check.completes_at - ctx.input(|input| input.time);
        if remaining > 0.0 {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(remaining));
            return;
        }
        let result = if check.email.starts_with("taken@") {
            Err("This demo address is already taken.".into())
        } else {
            Ok(())
        };
        self.email_check.finish_async(check.ticket, result);
        self.pending = None;
        ctx.request_repaint();
    }
}

pub(super) fn name_rules() -> [Rule<'static>; 2] {
    [
        Rule::required("Enter your name."),
        Rule::min_length(2, "Use at least two characters."),
    ]
}
pub(super) fn email_rules() -> [Rule<'static>; 2] {
    [
        Rule::required("Enter an email address."),
        Rule::email("Enter a valid email address."),
    ]
}
pub(super) fn password_rules() -> [Rule<'static>; 1] {
    [Rule::min_length(
        8,
        "Use at least eight characters, or leave this blank.",
    )]
}
pub(super) fn notes_rules() -> [Rule<'static>; 1] {
    [Rule::max_length(280, "Keep notes within 280 characters.")]
}
pub(super) fn role_rules() -> [Rule<'static>; 1] {
    [Rule::required("Choose a workspace role.")]
}
