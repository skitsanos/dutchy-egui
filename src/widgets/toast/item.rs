use super::{ToastAppearance, ToastVariant};
use crate::tokens::toast as tokens;
use std::time::Duration;
/// Owned notification content and behavior. Store it in an application-owned ToastQueue.
#[derive(Clone, Debug)]
pub struct Toast<T = ()> {
    pub(super) message: String,
    pub(super) title: Option<String>,
    pub(super) variant: ToastVariant,
    pub(super) appearance: ToastAppearance,
    pub(super) action: Option<(String, T)>,
    pub(super) timeout: Option<Duration>,
    pub(super) dismissible: bool,
    pub(super) enabled: bool,
    pub(super) announce: bool,
}
impl<T> Toast<T> {
    /// Solid information notification with manual dismissal and a five-second lifetime.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            title: None,
            variant: ToastVariant::Info,
            appearance: ToastAppearance::Solid,
            action: None,
            timeout: Some(tokens::DEFAULT_DURATION),
            dismissible: true,
            enabled: true,
            announce: true,
        }
    }
    /// Optional visible heading.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// Severity treatment and accessible name.
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Solid live-web or documented outline treatment.
    pub fn appearance(mut self, appearance: ToastAppearance) -> Self {
        self.appearance = appearance;
        self
    }
    /// Return this payload once when the native action button is activated.
    pub fn action(mut self, label: impl Into<String>, action: T) -> Self {
        self.action = Some((label.into(), action));
        self
    }
    /// Visible lifetime. None persists until dismissed. Some durations are clamped
    /// to at least four seconds, including zero. Queued/paused time does not count.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout.map(|value| value.max(tokens::MIN_DURATION));
        self
    }
    /// Show a close button; defaults to true.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
    /// Enable controls; disabled entries may still expire.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Disable live announcements for static/history presentations.
    pub fn announce(mut self, announce: bool) -> Self {
        self.announce = announce;
        self
    }
    pub(super) fn name(&self) -> String {
        self.title.as_ref().map_or_else(
            || self.variant.name().to_owned(),
            |title| format!("{}: {title}", self.variant.name()),
        )
    }
}
