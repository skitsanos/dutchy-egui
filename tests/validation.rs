//! Rule semantics and stale asynchronous result protection.

use dutchy_egui::validation::{FieldValidation, Rule, ValidationState, validate};

#[test]
fn required_and_optional_blank_values_follow_the_reference() {
    assert!(
        validate(
            " \n ",
            &[Rule::email("Email"), Rule::min_length(8, "Length")]
        )
        .is_ok()
    );
    assert_eq!(
        validate(" \n ", &[Rule::required("Required"), Rule::email("Email")]),
        Err("Required".into())
    );
}

#[test]
fn rules_return_the_first_error_and_count_unicode_scalars() {
    let rules = [
        Rule::min_length(2, "Too short"),
        Rule::max_length(3, "Too long"),
    ];
    assert!(validate("é🦊", &rules).is_ok());
    assert_eq!(validate("a", &rules), Err("Too short".into()));
    assert_eq!(validate("é🦊ab", &rules), Err("Too long".into()));
    assert!(validate("e\u{301}", &[Rule::max_length(2, "Length")]).is_ok());
}

#[test]
fn email_and_decimal_rules_reject_invalid_values() {
    for value in ["a@b", "a@@b.test", "a b@c.test", "@c.test", "a@.test"] {
        assert!(validate(value, &[Rule::email("Email")]).is_err(), "{value}");
    }
    assert!(validate("name+tag@example.test", &[Rule::email("Email")]).is_ok());
    for value in ["NaN", "inf", "-inf", "12x", "0xff"] {
        assert!(
            validate(value, &[Rule::number("Number")]).is_err(),
            "{value}"
        );
    }
    assert!(validate(" -1.5e2 ", &[Rule::number("Number")]).is_ok());
}

#[test]
fn custom_rules_can_borrow_application_data() {
    let existing = ["one", "two"];
    let rules = [Rule::custom(
        |value| !existing.contains(&value),
        "Already used",
    )];
    assert_eq!(validate("one", &rules), Err("Already used".into()));
    assert!(validate("three", &rules).is_ok());
}

#[test]
fn async_results_are_current_single_use_and_bound_to_the_field() {
    let mut first = FieldValidation::default();
    let mut second = FieldValidation::default();
    let old = first.begin_async();
    let current = first.begin_async();
    assert!(!first.finish_async(old, Err("Old error".into())));
    assert!(!second.finish_async(current, Ok(())));
    assert_eq!(first.state(), &ValidationState::Pending);
    assert!(first.finish_async(current, Ok(())));
    assert!(first.is_valid());
    assert!(!first.finish_async(current, Err("Repeated result".into())));
}

#[test]
fn reset_and_sync_validation_invalidate_pending_results() {
    let mut state = FieldValidation::default();
    let ticket = state.begin_async();
    state.clear();
    assert!(!state.touched());
    assert!(!state.finish_async(ticket, Ok(())));
    let ticket = state.begin_async();
    assert!(!state.validate("", &[Rule::required("Required")]));
    assert!(!state.finish_async(ticket, Ok(())));
    state.set_error("Server error");
    assert_eq!(
        state.state(),
        &ValidationState::Invalid("Server error".into())
    );
}
