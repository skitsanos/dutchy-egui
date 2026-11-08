//! Small original vector icons. Labels and interaction belong to their controls.
mod paint;
use egui::{Color32, Painter, Rect};

/// Host-painted decorative icon. Painting is clipped to the supplied bounds.
pub type IconPainter = fn(&Painter, Rect, Color32);

/// Original geometric icons plus a host painter, without an icon/font dependency.
#[derive(Clone, Copy, Debug)]
pub enum Icon {
    /// Add/create.
    Plus,
    /// Remove/decrease.
    Minus,
    /// Dismiss.
    Close,
    /// Confirm.
    Check,
    /// Find.
    Search,
    /// Copy/duplicate.
    Copy,
    /// Save to storage.
    Save,
    /// Download/export.
    Download,
    /// Delete.
    Trash,
    /// Bold text.
    Bold,
    /// Italic text.
    Italic,
    /// Underlined text.
    Underline,
    /// Align left.
    AlignLeft,
    /// Align center.
    AlignCenter,
    /// Align right.
    AlignRight,
    /// Grid view.
    Grid,
    /// List view.
    List,
    /// Undo.
    Undo,
    /// Redo.
    Redo,
    /// Start/resume.
    Play,
    /// Pause.
    Pause,
    /// Additional actions.
    More,
    /// Application-supplied painter receiving the current semantic ink color.
    Custom(IconPainter),
}

impl Icon {
    /// All built-in glyphs; custom painters are application-owned.
    pub const ALL: [Self; 22] = [
        Self::Plus,
        Self::Minus,
        Self::Close,
        Self::Check,
        Self::Search,
        Self::Copy,
        Self::Save,
        Self::Download,
        Self::Trash,
        Self::Bold,
        Self::Italic,
        Self::Underline,
        Self::AlignLeft,
        Self::AlignCenter,
        Self::AlignRight,
        Self::Grid,
        Self::List,
        Self::Undo,
        Self::Redo,
        Self::Play,
        Self::Pause,
        Self::More,
    ];
    /// Human-readable glyph name for galleries; controls need an action name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Plus => "Plus",
            Self::Minus => "Minus",
            Self::Close => "Close",
            Self::Check => "Check",
            Self::Search => "Search",
            Self::Copy => "Copy",
            Self::Save => "Save",
            Self::Download => "Download",
            Self::Trash => "Trash",
            Self::Bold => "Bold",
            Self::Italic => "Italic",
            Self::Underline => "Underline",
            Self::AlignLeft => "Align left",
            Self::AlignCenter => "Align center",
            Self::AlignRight => "Align right",
            Self::Grid => "Grid",
            Self::List => "List",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Play => "Play",
            Self::Pause => "Pause",
            Self::More => "More",
            Self::Custom(_) => "Custom",
        }
    }
    /// Paint inside finite bounds. No accessible node or hit target is created.
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        if !rect.is_finite() || rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let painter = painter.with_clip_rect(painter.clip_rect().intersect(rect));
        match self {
            Self::Custom(paint) => paint(&painter, rect, color),
            _ => paint::show(self, &painter, rect, color),
        }
    }
}
