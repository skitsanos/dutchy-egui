use super::ListSelectionMode;
use egui::{
    Response, WidgetInfo,
    accesskit::{Action, Orientation, Role},
};

pub(super) fn row(
    response: &Response,
    name: &str,
    index: usize,
    mode: ListSelectionMode,
    selected: bool,
    read_only: bool,
) {
    let role = if mode == ListSelectionMode::None {
        Role::ListItem
    } else {
        Role::ListBoxOption
    };
    response.widget_info(|| WidgetInfo::labeled(role, response.enabled(), name));
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_position_in_set(index);
        if mode != ListSelectionMode::None {
            node.set_selected(selected);
        }
        if response.enabled() {
            node.add_action(Action::Focus);
            node.add_action(Action::ScrollIntoView);
        }
        if read_only {
            node.set_read_only();
            node.remove_action(Action::Click);
        }
    });
}

pub(super) fn group(
    response: &Response,
    name: &str,
    count: usize,
    mode: ListSelectionMode,
    read_only: bool,
) {
    let role = if mode == ListSelectionMode::None {
        Role::List
    } else {
        Role::ListBox
    };
    response.widget_info(|| WidgetInfo::labeled(role, response.enabled(), name));
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_size_of_set(count);
        node.set_orientation(Orientation::Vertical);
        if mode == ListSelectionMode::Multiple {
            node.set_multiselectable();
        }
        if read_only {
            node.set_read_only();
        }
    });
}
