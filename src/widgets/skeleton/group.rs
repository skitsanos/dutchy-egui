use egui::accesskit::{Live, Role};
use egui::{IdSalt, InnerResponse, Ui, UiBuilder};
use std::borrow::Cow;

/// One named busy region for a set of decorative skeletons.
#[must_use = "Call a show method to render the loading region"]
pub struct SkeletonGroup<'a> {
    id: IdSalt,
    label: Cow<'a, str>,
    announce: bool,
}

impl<'a> SkeletonGroup<'a> {
    /// Bind a stable ID to a meaningful loading label, e.g. "Loading projects".
    pub fn new(id: impl egui::AsIdSalt, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            label: label.into(),
            announce: true,
        }
    }
    /// Disable live announcements while retaining the busy label.
    pub fn announce(mut self, announce: bool) -> Self {
        self.announce = announce;
        self
    }
    /// Compose application-specific placeholders without adding keyboard stops.
    /// The host replaces this group with real content when loading finishes.
    pub fn show<R>(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let result = ui.scope_builder(
            UiBuilder::new().scope_id(ui.make_persistent_id(("dutchy-skeleton", self.id))),
            contents,
        );
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| {
                node.set_role(if self.announce {
                    Role::Status
                } else {
                    Role::Group
                });
                node.set_label(self.label.to_string());
                node.set_busy();
                node.set_live(if self.announce {
                    Live::Polite
                } else {
                    Live::Off
                });
            });
        result
    }
}
