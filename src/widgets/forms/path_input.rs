use super::Input;
use crate::{Button, ButtonVariant, ControlSize, tokens::space};
use egui::{Align, IdSalt, Layout, Response, Sense, Ui, UiBuilder, vec2};
use std::borrow::Cow;

/// The kind of path the host should ask the user to choose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PathBrowseRequest {
    /// Choose a single existing file.
    OpenFile,
    /// Choose a single existing directory.
    OpenFolder,
    /// Choose a destination for a file.
    SaveFile,
}

/// Native responses and the optional browse intent for the current frame.
#[non_exhaustive]
pub struct PathInputResponse {
    /// The text editor, with its own change, focus and blur events.
    pub response: Response,
    /// The Browse button. Hosts can return focus here when a picker closes.
    pub browse: Response,
    /// Emitted when Browse is activated. The host handles the request.
    pub request: Option<PathBrowseRequest>,
}

/// An editable path paired with a Browse button.
///
/// The host owns the path text, picker and validation. The text is left as
/// entered, including paths that the host has yet to validate. Browse stacks
/// below the editor when the editor would be narrower than the button.
#[must_use = "Call show to render the path input"]
pub struct PathInput<'a> {
    id: IdSalt,
    value: &'a mut String,
    request: PathBrowseRequest,
    name: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    size: ControlSize,
    width: Option<f32>,
    enabled: bool,
    read_only: bool,
}

impl<'a> PathInput<'a> {
    /// Bind an application-owned path and a stable ID within the parent UI.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut String, request: PathBrowseRequest) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            request,
            name: "Path".into(),
            placeholder: "".into(),
            size: ControlSize::Medium,
            width: None,
            enabled: true,
            read_only: false,
        }
    }

    /// Name the editor and distinguish its Browse button from other fields.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Show a hint while the path is empty.
    pub fn placeholder(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = hint.into();
        self
    }

    /// Set the shared editor and button size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Set a preferred width, capped by the available space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }

    /// Enable editing and browsing, honoring disabled ancestors.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Keep text selectable and copyable while blocking edits and browsing.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Render the controls and report a browse request on activation.
    pub fn show(self, ui: &mut Ui) -> PathInputResponse {
        let root = ui.make_persistent_id(("dutchy-path-input", self.id));
        let button = Button::new("Browse")
            .size(self.size)
            .variant(ButtonVariant::Outline)
            .enabled(!self.read_only);
        let width = self
            .width
            .unwrap_or(ui.available_width())
            .min(ui.available_width())
            .max(0.0);
        let gap = space::SM;
        let height = self.size.height();
        let browse_width = button.natural_width(ui);
        let editor_width = width - browse_width - gap;
        // Stack rather than squeeze the path below the width of its button.
        let stacked = editor_width < browse_width;
        let (height, editor_width, layout) = if stacked {
            (2.0 * height + gap, width, Layout::top_down(Align::Min))
        } else {
            (height, editor_width, Layout::left_to_right(Align::Center))
        };
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let mut child = ui.new_child(
            UiBuilder::new()
                .scope_id(root)
                .max_rect(rect)
                .layout(layout),
        );
        if !self.enabled {
            child.disable();
        }
        child.spacing_mut().item_spacing = vec2(gap, gap);
        let response = child.add(
            Input::new("editor", self.value)
                .accessible_name(self.name.as_ref())
                .placeholder(self.placeholder)
                .size(self.size)
                .width(editor_width)
                .read_only(self.read_only),
        );
        let browse = child
            .add(button.full_width(stacked).shrink_padding_to_fit())
            .accessible_name(format!("Browse for {}", self.name));
        PathInputResponse {
            response,
            request: browse.clicked().then_some(self.request),
            browse,
        }
    }
}
