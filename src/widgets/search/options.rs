use super::{Cursor, Results, SearchKeys, Selection, results::ResultsResponse};
use crate::{
    MenuItem, MenuItemVariant, SelectOption, Theme,
    tokens::{border, menu as geometry},
    widgets::menu::paint,
};
use egui::{Id, InnerResponse, Popup, PopupCloseBehavior, Response};

pub(crate) fn option_items<'a, T: Clone>(
    options: &'a [SelectOption<'a, T>],
    selection: Selection<'_>,
) -> Vec<MenuItem<'a, T>> {
    options
        .iter()
        .map(|option| MenuItem {
            checked: selection
                .is_multiple()
                .then(|| selection.contains(option.id)),
            children: None,
            value: option.value.clone(),
            label: option.label.clone(),
            group: None,
            shortcut: None,
            variant: MenuItemVariant::Default,
            enabled: option.enabled,
            separator_before: false,
            icon: None,
            id: option.id,
        })
        .collect()
}

pub(crate) fn options_popup<T: Clone>(
    response: &Response,
    root: Id,
    open: &mut bool,
    results: Results<'_, T>,
    cursor: &mut Cursor,
    keys: &SearchKeys,
) -> Option<InnerResponse<ResultsResponse<T>>> {
    if !*open {
        return None;
    }
    crate::commands::dispatch::mark_overlay(&response.ctx);
    Popup::from_response(response)
        .id(root.with("popup"))
        .open_bool(open)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .gap(geometry::POPUP_GAP)
        .frame(paint::frame(Theme::from_context(&response.ctx)))
        .width(response.rect.width())
        .accessible_name(results.name.to_owned())
        .show(|ui| {
            ui.set_width((response.rect.width() - border::STANDARD * 2.0).max(0.0));
            results.show(ui, root.with("results"), cursor, keys)
        })
}
