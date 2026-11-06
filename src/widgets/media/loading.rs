use crate::{
    Spinner, SpinnerSize, Text, TextSize, Theme,
    tokens::{Palette, space},
};
use egui::{Id, Rect, Ui, UiBuilder, WidgetInfo, pos2, vec2};

pub(super) fn show(ui: &mut Ui, root: Id, rect: Rect, alt: &str, palette: Palette) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let theme = Theme::from_context(ui.ctx());
    let opacity = crate::widgets::loading_motion::opacity(ui, !theme.reduced_motion);
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    painter.rect_filled(rect, 0, palette.muted.gamma_multiply(opacity));
    let size = SpinnerSize::Large;
    let extent = size.extent();
    let available = rect.shrink(space::SM);
    if available.width().min(available.height()) < extent {
        return;
    }

    let job = Text::new("Loading image")
        .size(TextSize::Sm)
        .color(palette.muted_foreground)
        .layout_job(&theme, available.width());
    let label = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let height = extent + space::SM + label.size().y;
    let show_label = height <= available.height() && label.size().x <= available.width();
    let center = if show_label {
        pos2(
            rect.center().x,
            rect.center().y - height / 2.0 + extent / 2.0,
        )
    } else {
        rect.center()
    };
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(root.with("loading"))
            .max_rect(rect),
    );
    child.set_clip_rect(ui.clip_rect().intersect(rect));
    let spinner = child.put(
        Rect::from_center_size(center, vec2(extent, extent)),
        Spinner::new().size(size).color(palette.muted_foreground),
    );
    spinner.widget_info(|| {
        WidgetInfo::labeled(
            egui::accesskit::Role::ProgressIndicator,
            ui.is_enabled(),
            format!("Loading {alt}"),
        )
    });
    if show_label {
        painter.galley(
            pos2(
                rect.center().x - label.size().x / 2.0,
                center.y + extent / 2.0 + space::SM,
            ),
            label,
            palette.muted_foreground,
        );
    }
}
