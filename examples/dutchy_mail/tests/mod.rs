//! Tests run by `cargo test --all-features` through the `[[example]]` entry:
//! mailbox units, interaction flows and two image baselines.

mod compose;
mod flows;
mod rendering;

use crate::app::App;
use crate::mailbox::{FolderId, MessageId};
use crate::support::data::FIXED_TODAY;
use dutchy_egui::{Accent, Mode};
use egui::{Key, Modifiers};
use egui_kittest::Harness;

/// The seeded application at desktop size with motion disabled.
fn harness() -> Harness<'static, App> {
    harness_with(Accent::Orange, Mode::Light, 1.0)
}

fn harness_with(accent: Accent, mode: Mode, scale: f32) -> Harness<'static, App> {
    let mut app = App::seeded(FIXED_TODAY);
    app.theme.accent = accent;
    app.theme.mode = mode;
    app.theme.reduced_motion = true;
    let theme = app.theme.theme();
    let mut h = Harness::builder()
        .with_os(egui::os::OperatingSystem::Mac)
        .with_size([1280.0, 800.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(match mode {
            Mode::Light => egui::Theme::Light,
            Mode::Dark => egui::Theme::Dark,
        })
        .build_eframe(move |cc| {
            dutchy_egui::install(&cc.egui_ctx, theme);
            app
        });
    h.run();
    h
}

/// Accessible label of a message row: sender and subject.
fn row_label(h: &Harness<'_, App>, id: MessageId) -> String {
    let message = h.state().mailbox.message(id).expect("message exists");
    format!("{}, {}", message.from, message.subject)
}

/// The newest message of a folder.
fn newest(h: &Harness<'_, App>, folder: FolderId) -> MessageId {
    h.state().mailbox.view(folder, "")[0].id
}

fn chord(h: &mut Harness<'_, App>, modifiers: Modifiers, key: Key) {
    h.key_press_modifiers(modifiers, key);
    h.run();
}
