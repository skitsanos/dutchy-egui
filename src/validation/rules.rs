use std::borrow::Cow;

enum Check<'a> {
    Required,
    Email,
    Number,
    MinLength(usize),
    MaxLength(usize),
    Custom(Box<dyn Fn(&str) -> bool + 'a>),
}

/// A synchronous rule with a user-facing error message.
///
/// Non-required rules skip blank values, matching the web system. String lengths
/// count Unicode scalar values, consistently with egui's character limit.
pub struct Rule<'a> {
    check: Check<'a>,
    message: Cow<'a, str>,
}

impl<'a> Rule<'a> {
    /// Reject empty or whitespace-only input.
    pub fn required(message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            check: Check::Required,
            message: message.into(),
        }
    }

    /// Basic email shape validation, matching the web reference's simple rule.
    /// This does not verify delivery or implement the complete email RFC grammar.
    pub fn email(message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            check: Check::Email,
            message: message.into(),
        }
    }

    /// Accept a finite decimal number, including decimal exponent notation.
    pub fn number(message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            check: Check::Number,
            message: message.into(),
        }
    }

    /// Require at least this many Unicode scalar values.
    pub fn min_length(min: usize, message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            check: Check::MinLength(min),
            message: message.into(),
        }
    }

    /// Allow at most this many Unicode scalar values.
    pub fn max_length(max: usize, message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            check: Check::MaxLength(max),
            message: message.into(),
        }
    }

    /// Supply an application predicate, for patterns, URLs, or domain rules.
    pub fn custom(check: impl Fn(&str) -> bool + 'a, message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            check: Check::Custom(Box::new(check)),
            message: message.into(),
        }
    }

    fn passes(&self, value: &str) -> bool {
        if value.trim().is_empty() {
            return !matches!(self.check, Check::Required);
        }
        match &self.check {
            Check::Required => true,
            Check::Email => email_shape(value),
            Check::Number => value.trim().parse::<f64>().is_ok_and(f64::is_finite),
            Check::MinLength(min) => value.chars().count() >= *min,
            Check::MaxLength(max) => value.chars().count() <= *max,
            Check::Custom(check) => check(value),
        }
    }
}

/// Return the first failing rule's message, or `Ok(())`.
pub fn validate(value: &str, rules: &[Rule<'_>]) -> Result<(), String> {
    rules
        .iter()
        .find(|rule| !rule.passes(value))
        .map_or(Ok(()), |rule| Err(rule.message.to_string()))
}

fn email_shape(value: &str) -> bool {
    if value.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.contains('@') {
        return false;
    }
    domain
        .rsplit_once('.')
        .is_some_and(|(host, suffix)| !host.is_empty() && !suffix.is_empty())
}
