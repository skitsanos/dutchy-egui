use crate::config::Config;

#[test]
fn defaults_are_offline_without_credentials() {
    let c = Config::from_sources(None, |_| None);
    assert_eq!(c.model, "gpt-6-luna");
    assert_eq!(c.effort, "low");
    assert!(!c.key_configured);
    assert!(c.streaming);
    assert!(c.warning.is_none());
}
#[test]
fn process_values_override_file_values_including_empty_credentials() {
    let c = Config::from_sources(
        Some("OPENAI_API_KEY=fixture-value\nMODEL_NAME=file-model\nMODEL_EFFORT=high\n"),
        |key| match key {
            "OPENAI_API_KEY" => Some(String::new()),
            "MODEL_NAME" => Some("process-model".into()),
            _ => None,
        },
    );
    assert_eq!(c.model, "process-model");
    assert_eq!(c.effort, "high");
    assert!(!c.key_configured);
}
#[test]
fn quoted_literals_comments_and_unrelated_variables_are_safe() {
    let c = Config::from_sources(
        Some(
            "\u{feff}# Local settings\nexport MODEL_NAME='demo-model' # comment\nMODEL_EFFORT=medium # comment\nOPENAI_API_KEY=\"fixture=secret#opaque\"\nOTHER=$(false)\n",
        ),
        |_| None,
    );
    assert_eq!(c.model, "demo-model");
    assert_eq!(c.effort, "medium");
    assert!(c.key_configured);
    assert!(!format!("{c:?}").contains("fixture=secret"));
}
#[test]
fn malformed_or_invalid_values_fall_back_without_echoing_input() {
    let c = Config::from_sources(
        Some("MODEL_NAME='unterminated\nMODEL_EFFORT=not-supported\n"),
        |_| None,
    );
    assert_eq!(c.model, "gpt-6-luna");
    assert_eq!(c.effort, "low");
    assert!(!c.warning.unwrap().contains("not-supported"));
    let long = format!("MODEL_NAME={}", "x".repeat(129));
    assert_eq!(
        Config::from_sources(Some(&long), |_| None).model,
        "gpt-6-luna"
    );
}
