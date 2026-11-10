use egui::{Id, IdSalt, Response, Sense, Ui, WidgetInfo};
use std::borrow::Cow;

/// Sharp file selection presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileUploadVariant {
    /// Spacious dashed drop zone with a decorative upload glyph.
    #[default]
    DropZone,
    /// Compact solid-border file input; supports the same drop events.
    Compact,
}
/// Host requests from a file selection surface. No file contents are read.
#[non_exhaustive]
pub struct FileUploadResponse {
    /// Native clickable/focusable selection control.
    pub response: Response,
    /// Open the application's picker. Enter/Space/click share this request.
    pub browse_requested: bool,
    /// Claimed lazy egui handles, in the integration's original order.
    pub dropped: Vec<egui::DroppedFileHandle>,
    /// External files are over this visible, enabled, unobscured target.
    pub drag_hovered: bool,
}
/// File picker/drop target. The host owns selection, metadata, contents and I/O.
#[must_use = "Call show to receive browse/drop requests"]
pub struct FileUpload<'a> {
    pub(super) id: IdSalt,
    pub(super) label: Cow<'a, str>,
    pub(super) description: Cow<'a, str>,
    pub(super) hint: Option<Cow<'a, str>>,
    pub(super) variant: FileUploadVariant,
    pub(super) enabled: bool,
}
impl<'a> FileUpload<'a> {
    /// Dashed drop zone; stable IDs distinguish multiple independent targets.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: IdSalt::new(id),
            label: "Drop files here".into(),
            description: "or click to browse".into(),
            hint: None,
            variant: Default::default(),
            enabled: true,
        }
    }
    /// Visible and accessible control name.
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }
    /// Secondary instruction, hidden in Compact mode.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = description.into();
        self
    }
    /// Visible metadata guidance; does not implement a validation policy.
    pub fn hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    /// Choose the spacious drop zone or compact input.
    pub fn variant(mut self, variant: FileUploadVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Disable picker/drop requests, honoring a disabled parent too.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Draw and claim files only inside this target's visible, interactive area.
    /// Claimed files are consumed from egui input once per viewport/frame.
    pub fn show(self, ui: &mut Ui) -> FileUploadResponse {
        let id = ui.make_persistent_id(("dutchy-file-upload", self.id));
        let layout = super::zone_paint::allocate(ui, &self);
        let rect = layout.rect;
        let response = ui.interact(
            rect,
            id,
            if self.enabled && ui.is_enabled() {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let enabled = self.enabled && ui.is_enabled();
        response.widget_info(|| {
            WidgetInfo::labeled(egui::accesskit::Role::Button, enabled, self.label.as_ref())
        });
        response.ctx.accesskit_node_builder(id, |node| {
            node.set_description(self.hint.as_ref().map_or_else(
                || self.description.to_string(),
                |hint| format!("{}; {hint}", self.description),
            ));
            if !enabled {
                node.set_disabled();
            }
        });
        let over = enabled
            && response.contains_pointer()
            && ui.is_rect_visible(rect)
            && ui.memory(|memory| {
                memory
                    .top_modal_layer()
                    .is_none_or(|layer| layer == ui.layer_id())
            });
        let drag_hovered = over && ui.input(|input| !input.raw.hovered_files.is_empty());
        let dropped = if over { claim(ui) } else { Vec::new() };
        let emphasized =
            enabled && (response.hovered() || drag_hovered || response.is_pointer_button_down_on());
        if response.hovered() && enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        super::zone_paint::show(ui, &layout, &response, emphasized, enabled);
        let browse_requested = enabled && response.clicked();
        FileUploadResponse {
            response,
            browse_requested,
            dropped,
            drag_hovered,
        }
    }
}
fn claim(ui: &Ui) -> Vec<egui::DroppedFileHandle> {
    let ctx = ui.ctx();
    let frame = ctx.cumulative_frame_nr();
    let key = Id::unique("dutchy-file-drop-claim").with(ctx.viewport_id());
    if ctx.data(|data| data.get_temp::<u64>(key) == Some(frame)) {
        return Vec::new();
    }
    let files = ctx.input_mut(|input| std::mem::take(&mut input.raw.dropped_files));
    if !files.is_empty() {
        ctx.data_mut(|data| data.insert_temp(key, frame));
    }
    files
}
