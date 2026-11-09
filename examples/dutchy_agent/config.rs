//! Local configuration without process mutation; in-memory credentials redact Debug.
use crate::provider::Mode;
use std::{collections::HashMap, fs, io::ErrorKind};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Credential(String);
impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    pub model: String,
    pub effort: String,
    pub system_prompt: String,
    pub streaming: bool,
    pub key_configured: bool,
    pub provider: Mode,
    credential: Option<Credential>,
    pub warning: Option<&'static str>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gpt-6-luna".into(),
            effort: "low".into(),
            system_prompt: "Give concise, useful answers with Markdown and readable code.".into(),
            streaming: true,
            key_configured: false,
            provider: Mode::Mock,
            credential: None,
            warning: None,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let file = fs::read_to_string(".env");
        let mut config = Self::from_sources(file.as_deref().ok(), |key| std::env::var(key).ok());
        if let Err(error) = file
            && error.kind() != ErrorKind::NotFound
        {
            config.warning = Some("Local configuration could not be read. Defaults apply.");
        }
        config
    }

    /// Injectable sources keep tests independent from machine credentials.
    pub fn from_sources(file: Option<&str>, environment: impl Fn(&str) -> Option<String>) -> Self {
        let values = parse(file.unwrap_or_default());
        let value = |key| environment(key).or_else(|| values.get(key).cloned());
        let mut config = Self::default();
        if let Some(model) = value("MODEL_NAME").filter(|value| !value.trim().is_empty()) {
            if model.chars().count() <= 128 {
                config.model = model.trim().into();
            } else {
                config.warning = Some("The model name is too long. The default is used.");
            }
        }
        if let Some(effort) = value("MODEL_EFFORT").filter(|value| !value.trim().is_empty()) {
            if ["low", "medium", "high"].contains(&effort.trim()) {
                config.effort = effort.trim().into();
            } else {
                config.warning = Some("Unrecognized model effort. Low is used.");
            }
        }
        config.credential = value("OPENAI_API_KEY")
            .filter(|v| !v.trim().is_empty())
            .map(|value| Credential(value.trim().to_owned()));
        config.key_configured = config.credential.is_some();
        if config.key_configured {
            config.provider = Mode::OpenAi;
        }
        config
    }

    pub fn credential(&self) -> Option<&str> {
        self.credential.as_ref().map(|key| key.0.as_str())
    }
}

fn parse(source: &str) -> HashMap<String, String> {
    source
        .trim_start_matches('\u{feff}')
        .lines()
        .filter_map(|line| {
            let line = line.trim().strip_prefix("export ").unwrap_or(line.trim());
            let (key, value) = line.split_once('=')?;
            let key = key.trim();
            if !["OPENAI_API_KEY", "MODEL_NAME", "MODEL_EFFORT"].contains(&key) {
                return None;
            }
            Some((key.to_owned(), literal(value.trim())?.to_owned()))
        })
        .collect()
}

fn literal(value: &str) -> Option<&str> {
    if let Some(quote) = value.chars().next().filter(|c| matches!(c, '\'' | '"')) {
        let rest = &value[1..];
        let end = rest.find(quote)?;
        let tail = rest[end + 1..].trim();
        return (tail.is_empty() || tail.starts_with('#')).then_some(&rest[..end]);
    }
    Some(value.split(" #").next().unwrap_or_default().trim())
}
