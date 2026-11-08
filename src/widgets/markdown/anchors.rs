//! Fragment links scroll to headings. A click stores the requested anchor;
//! the heading that owns it scrolls into view on the next pass and clears it.

use egui::{Align, Context, Id, Rect, Ui};

fn key(root: Id) -> Id {
    root.with("scroll-target")
}

/// Request a scroll to the heading with this anchor.
pub(super) fn request(ctx: &Context, root: Id, fragment: &str) {
    ctx.data_mut(|data| data.insert_temp(key(root), fragment.to_string()));
    ctx.request_repaint();
}

/// Scroll a heading into view if it is the pending target. Returns whether it was.
pub(super) fn reveal(ui: &Ui, root: Id, anchor: &str, rect: Rect, reduced_motion: bool) -> bool {
    let pending = ui.ctx().data(|data| data.get_temp::<String>(key(root)));
    if pending.as_deref() != Some(anchor) {
        return false;
    }
    ui.ctx().data_mut(|data| data.remove::<String>(key(root)));
    let animation = if reduced_motion {
        egui::style::ScrollAnimation::none()
    } else {
        ui.style().scroll_animation
    };
    ui.scroll_to_rect_animation(rect, Some(Align::Min), animation);
    true
}
