use super::{focus, rows};
use egui::{Id, IdSalt, Response, Ui, UiBuilder, WidgetInfo};
use std::borrow::Cow;

/// Application-supplied transfer state. These widgets never execute transfers.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum FileStatus {
    /// Selected and awaiting a host action.
    #[default]
    Ready,
    /// Known 0..100 progress, or None for indeterminate progress.
    Uploading(Option<f64>),
    /// Host reported successful completion.
    Complete,
    /// Host error; the row exposes a retry request.
    Failed(String),
}
/// File-row presentation; keys must be stable and unique in a list.
#[non_exhaustive]
pub struct FileItem<'a, K> {
    /// Immutable host identity, independent of name, status, or order.
    pub key: K,
    /// Display name; the host retains the actual file handle/path.
    pub name: Cow<'a, str>,
    /// Optional byte size.
    pub bytes: Option<u64>,
    /// Current host-reported state.
    pub status: FileStatus,
    /// Enable actions, honoring disabled parent lists/UIs too.
    pub enabled: bool,
    /// Show remove/cancel. Retry remains available for failed enabled items.
    pub removable: bool,
}
impl<'a, K> FileItem<'a, K> {
    /// Removable, enabled row with unknown size and Ready status.
    pub fn new(key: K, name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            name: name.into(),
            bytes: None,
            status: Default::default(),
            enabled: true,
            removable: true,
        }
    }
    /// Known byte size.
    pub fn size(mut self, bytes: u64) -> Self {
        self.bytes = Some(bytes);
        self
    }
    /// Host transfer state.
    pub fn status(mut self, status: FileStatus) -> Self {
        self.status = status;
        self
    }
    /// Enable row actions.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Configure remove/cancel availability.
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }
}
/// Keyed, independent host actions from the list.
#[non_exhaustive]
pub struct FileListResponse<K> {
    /// Accessible List surface.
    pub response: Response,
    /// Remove a selected/completed/failed file.
    pub remove_requested: Option<K>,
    /// Cancel an in-progress transfer. The host cancels its own operation.
    pub cancel_requested: Option<K>,
    /// Retry a failed transfer.
    pub retry_requested: Option<K>,
}
/// Bordered rows with native actions and host-controlled progress/errors.
/// Keep calling this, including with an empty slice, to finish removal focus return.
#[must_use = "Call show to receive file actions"]
pub struct FileList<'a, K> {
    id: IdSalt,
    items: &'a [FileItem<'a, K>],
    name: Cow<'a, str>,
    enabled: bool,
    empty_focus: Option<Id>,
}
impl<'a, K: egui::AsIdSalt + Clone> FileList<'a, K> {
    /// A keyed list named Selected files, with no internal file/transfer state.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [FileItem<'a, K>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            name: "Selected files".into(),
            enabled: true,
            empty_focus: None,
        }
    }
    /// Accessible collection name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable all row actions.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Focus destination after the host removes the final actionable row.
    pub fn empty_focus(mut self, target: Id) -> Self {
        self.empty_focus = Some(target);
        self
    }
    /// Return actions; never remove files, read bytes or start/cancel transfers.
    pub fn show(self, ui: &mut Ui) -> FileListResponse<K> {
        let root = ui.make_persistent_id(("dutchy-file-list", self.id));
        let mut remove_requested = None;
        let mut cancel_requested = None;
        let mut retry_requested = None;
        let mut controls = Vec::new();
        let response = ui
            .scope_builder(UiBuilder::new().id_salt(root), |ui| {
                if !self.enabled {
                    ui.disable();
                }
                let anchor = ui.max_rect().min;
                for item in self.items {
                    let id = root.with(&item.key);
                    let (action, retry) = rows::show(ui, id, item, anchor);
                    if let Some(action) = action {
                        if action.enabled() {
                            controls.push(action.id);
                        }
                        if action.clicked() {
                            focus::remember(ui, root, action.id, controls.len().saturating_sub(1));
                            if matches!(item.status, FileStatus::Uploading(_)) {
                                cancel_requested = Some(item.key.clone());
                            } else {
                                remove_requested = Some(item.key.clone());
                            }
                        }
                    }
                    if retry {
                        retry_requested = Some(item.key.clone());
                    }
                }
                focus::restore(ui, root, &controls, self.empty_focus);
            })
            .response;
        response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::List,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        FileListResponse {
            response,
            remove_requested,
            cancel_requested,
            retry_requested,
        }
    }
}
