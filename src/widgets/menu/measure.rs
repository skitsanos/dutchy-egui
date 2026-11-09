use super::MenuItem;
use crate::{
    Family, MenuWidth, Text, TextSize, Theme, Tracking, Weight,
    tokens::{border, menu as geometry},
};
use egui::Ui;

pub(super) fn child_width<T>(ui: &Ui, items: &[MenuItem<'_, T>], maximum: MenuWidth) -> f32 {
    let theme = Theme::from_context(ui.ctx());
    let check = items
        .iter()
        .any(|i| i.checked.is_some() && i.children.is_none());
    let slot = geometry::ICON_SIZE + geometry::ITEM_GAP;
    let mut natural = 0.0_f32;
    for item in items {
        let label = width(
            ui,
            Text::new(item.label.as_ref()).size(TextSize::Sm),
            &theme,
        );
        let hint = item.shortcut.as_deref().map_or(0.0, |s| {
            width(
                ui,
                Text::new(s).family(Family::Mono).size(TextSize::Xs),
                &theme,
            ) + geometry::ITEM_GAP
        });
        let columns = slot
            * (usize::from(check)
                + usize::from(item.icon.is_some())
                + usize::from(item.children.is_some())) as f32;
        natural = natural.max(label + hint + columns);
        if let Some(group) = &item.group {
            natural = natural.max(width(
                ui,
                Text::new(group.as_ref())
                    .size(TextSize::Xs)
                    .weight(Weight::Bold)
                    .tracking(Tracking::Wider)
                    .uppercase(true),
                &theme,
            ));
        }
    }
    (natural + 2.0 * (geometry::PADDING_X + border::STANDARD))
        .max(geometry::SUBMENU_MIN_WIDTH)
        .min(maximum.points())
        .min(ui.ctx().content_rect().width())
}

fn width(ui: &Ui, text: Text<'_>, theme: &Theme) -> f32 {
    let job = text.layout_job(theme, f32::INFINITY);
    ui.fonts_mut(|fonts| fonts.layout_job(job).size().x)
}
