use crate::Icon;
use std::borrow::Cow;

/// Color of a segment's square marker, or of its icon. Text keeps the
/// foreground color, so the text itself must state the status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum StatusTone {
    /// No marker; an icon uses the foreground color.
    #[default]
    Default,
    /// Brand accent.
    Primary,
    /// Successful or healthy state.
    Success,
    /// Needs attention.
    Warning,
    /// Failed or offline state.
    Destructive,
    /// Informational state.
    Info,
}

impl StatusTone {
    /// Every tone in display order.
    pub const ALL: &[Self] = &[
        Self::Default,
        Self::Primary,
        Self::Success,
        Self::Warning,
        Self::Destructive,
        Self::Info,
    ];
}

/// Which end of the bar a segment belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum StatusAlign {
    /// Packed from the leading (left) edge, in slice order.
    #[default]
    Start,
    /// Packed against the trailing (right) edge, keeping slice order.
    End,
}

#[derive(Clone, Debug)]
pub(super) enum Content<'a> {
    Text,
    Labeled(Cow<'a, str>),
    Hint(Cow<'a, str>),
}

/// One keyed segment of a [`StatusBar`](super::StatusBar).
///
/// The key identifies the segment across frames, reordering and hiding.
/// Values are host-formatted text; the bar never parses or counts them.
#[derive(Clone, Debug)]
#[must_use = "Pass the item to a StatusBar"]
pub struct StatusItem<'a, K> {
    pub(super) key: K,
    pub(super) content: Content<'a>,
    pub(super) text: Cow<'a, str>,
    pub(super) icon: Option<Icon>,
    pub(super) tone: StatusTone,
    pub(super) progress: Option<Option<f64>>,
    pub(super) priority: u8,
    pub(super) align: StatusAlign,
    pub(super) clickable: bool,
    pub(super) enabled: bool,
    pub(super) live: bool,
    pub(super) tooltip: Option<Cow<'a, str>>,
    pub(super) name: Option<Cow<'a, str>>,
}

impl<'a, K> StatusItem<'a, K> {
    fn with(key: K, content: Content<'a>, text: Cow<'a, str>) -> Self {
        Self {
            key,
            content,
            text,
            icon: None,
            tone: StatusTone::Default,
            progress: None,
            priority: 0,
            align: StatusAlign::Start,
            clickable: false,
            enabled: true,
            live: false,
            tooltip: None,
            name: None,
        }
    }
    /// A plain value, such as `Up to date`. It is also the accessible name.
    pub fn text(key: K, text: impl Into<Cow<'a, str>>) -> Self {
        Self::with(key, Content::Text, text.into())
    }
    /// A muted label followed by its value, such as `Unread 42`. Assistive
    /// technology reads `Unread: 42`; a clickable or progress segment keeps
    /// the label as its name and the value as its accessible value.
    pub fn labeled(key: K, label: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self::with(key, Content::Labeled(label.into()), value.into())
    }
    /// A keyboard hint: a key chip and what it does, such as `Ctrl+K Commands`.
    /// The action is the accessible name and the chord its keyboard shortcut.
    pub fn hint(key: K, chord: impl Into<Cow<'a, str>>, action: impl Into<Cow<'a, str>>) -> Self {
        Self::with(key, Content::Hint(chord.into()), action.into())
    }
    /// Leading icon, colored by the tone.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Marker or icon color.
    pub fn tone(mut self, tone: StatusTone) -> Self {
        self.tone = tone;
        self
    }
    /// Trailing determinate progress in 0..=100 units, also exposed as a
    /// progress indicator. NaN becomes zero; other values are clamped.
    pub fn progress(mut self, percent: f64) -> Self {
        self.progress = Some(Some(if percent.is_nan() {
            0.0
        } else {
            percent.clamp(0.0, 100.0)
        }));
        self
    }
    /// Trailing indeterminate progress, exposed as a busy progress indicator.
    pub fn busy(mut self) -> Self {
        self.progress = Some(None);
        self
    }
    /// Overflow priority. When the bar is too narrow, the lowest priority is
    /// hidden first and, among equal priorities, the later item. Default 0.
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    /// Which end of the bar this segment belongs to.
    pub fn align(mut self, align: StatusAlign) -> Self {
        self.align = align;
        self
    }
    /// Make the segment a focusable button whose activation the bar returns.
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }
    /// Disable this segment, respecting a disabled bar or parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Announce changes to this segment politely through a live status node.
    /// Keep it for state changes such as sync results, not per-frame counters.
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
    /// Hover/focus hint, also the accessible description.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    /// Accessible name, overriding the label, text or action.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Full accessible text of a passive segment, such as `Unread: 42`.
    pub(super) fn spoken(&self) -> String {
        match (&self.name, &self.content) {
            (Some(name), _) => name.to_string(),
            (None, Content::Labeled(_)) => self.full_text().into_owned(),
            (None, _) => self.text.to_string(),
        }
    }

    /// Complete visible wording, independent of an accessible-name override.
    pub(super) fn full_text(&self) -> Cow<'_, str> {
        match &self.content {
            Content::Labeled(label) => format!("{label}: {}", self.text).into(),
            Content::Hint(chord) => format!("{chord} {}", self.text).into(),
            Content::Text => Cow::Borrowed(&self.text),
        }
    }

    pub(super) fn name(&self) -> &str {
        match (&self.name, &self.content) {
            (Some(name), _) => name,
            (None, Content::Labeled(label)) => label,
            (None, _) => &self.text,
        }
    }
}
