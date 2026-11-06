//! Pure rules and application-owned validation state. No async runtime or I/O.

mod rules;
mod state;

pub use rules::{Rule, validate};
pub use state::{FieldValidation, ValidationState, ValidationTicket, ValidationTrigger};
