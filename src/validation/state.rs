use std::sync::atomic::{AtomicU64, Ordering};

use super::{Rule, validate};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// Presentation of a field's current validation result.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ValidationState {
    /// No current result, including after an edit/reset.
    #[default]
    Unchecked,
    /// All rules passed for the last validated value.
    Valid,
    /// A user-facing error.
    Invalid(String),
    /// The application is executing an asynchronous check.
    Pending,
}

/// When the field should execute synchronous rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ValidationTrigger {
    /// Validate after any edit.
    OnChange,
    /// Validate when focus leaves the control.
    #[default]
    OnBlur,
    /// Validate only when the application calls `FieldValidation::validate`.
    OnSubmit,
}

/// Opaque request identity. Tickets cannot be used across fields or after edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationTicket {
    owner: u64,
    revision: u64,
}

/// Application-owned field state. Values and async execution stay with the host.
#[derive(Debug)]
pub struct FieldValidation {
    state: ValidationState,
    touched: bool,
    owner: u64,
    revision: u64,
}

impl Default for FieldValidation {
    fn default() -> Self {
        Self {
            state: ValidationState::Unchecked,
            touched: false,
            owner: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
            revision: 0,
        }
    }
}

impl FieldValidation {
    /// Current presentation state.
    pub fn state(&self) -> &ValidationState {
        &self.state
    }

    /// Whether focus has left this control or validation was explicitly requested.
    pub fn touched(&self) -> bool {
        self.touched
    }

    /// Whether the current result is valid. Unchecked/pending fields are not valid.
    pub fn is_valid(&self) -> bool {
        self.state == ValidationState::Valid
    }

    /// Observe an egui control response once per frame. Rules only run on events.
    ///
    /// Edits invalidate old results/tickets, even with OnSubmit. Call `clear` when
    /// changing the value outside this control (or before applying a server value).
    pub fn update(
        &mut self,
        response: &egui::Response,
        value: &str,
        trigger: ValidationTrigger,
        rules: &[Rule<'_>],
    ) {
        let before = self.state.clone();
        if response.changed() {
            self.invalidate();
        }
        if response.lost_focus() {
            self.touched = true;
        }
        let should_validate = match trigger {
            ValidationTrigger::OnChange => response.changed(),
            ValidationTrigger::OnBlur => response.lost_focus(),
            ValidationTrigger::OnSubmit => false,
        };
        if should_validate && response.enabled() {
            self.validate(value, rules);
        }
        if self.state != before {
            response.ctx.request_repaint();
        }
    }

    /// Execute rules now (e.g. submit), cancel pending checks, and store the result.
    pub fn validate(&mut self, value: &str, rules: &[Rule<'_>]) -> bool {
        self.invalidate();
        self.touched = true;
        self.state = match validate(value, rules) {
            Ok(()) => ValidationState::Valid,
            Err(message) => ValidationState::Invalid(message),
        };
        self.is_valid()
    }

    /// Display a host/server error and cancel pending work for this field.
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.invalidate();
        self.touched = true;
        self.state = ValidationState::Invalid(message.into());
    }

    /// Reset result/touched state, invalidating outstanding request tickets.
    pub fn clear(&mut self) {
        self.invalidate();
        self.touched = false;
    }

    /// Mark an application-run async check pending and return its identity.
    /// The host must request repaint after accepting an async completion.
    pub fn begin_async(&mut self) -> ValidationTicket {
        self.invalidate();
        self.touched = true;
        self.state = ValidationState::Pending;
        ValidationTicket {
            owner: self.owner,
            revision: self.revision,
        }
    }

    /// Accept a current async result. Return false for stale or cross-field tickets.
    pub fn finish_async(&mut self, ticket: ValidationTicket, result: Result<(), String>) -> bool {
        if ticket.owner != self.owner
            || ticket.revision != self.revision
            || self.state != ValidationState::Pending
        {
            return false;
        }
        self.state = match result {
            Ok(()) => ValidationState::Valid,
            Err(message) => ValidationState::Invalid(message),
        };
        true
    }

    fn invalidate(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.state = ValidationState::Unchecked;
    }
}
