//! The folder tree: a TreeView with unread counts, inside the left pane.

use crate::mailbox::{Folder, FolderId, Mailbox};
use crate::text;
use dutchy_egui::{
    Badge, BadgeVariant, ListSelectionMode, Text, TextColor, TextSize, TreeNode, TreeState,
    TreeView, tokens::space,
};
use egui::{Align, Layout, Ui};

/// Tree rows are 40 points: one compact line with room for a badge.
const ROW_HEIGHT: f32 = 40.0;

pub fn show(ui: &mut Ui, mailbox: &Mailbox, state: &mut TreeState<FolderId>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.add(Text::label("Folders").color(TextColor::Muted));
            ui.add_space(space::SM);
            let children: Vec<Vec<TreeNode<'_, FolderId>>> = mailbox
                .folders()
                .iter()
                .map(|parent| nodes(mailbox.folders(), Some(parent.id), &[]))
                .collect();
            let roots: Vec<TreeNode<'_, FolderId>> = mailbox
                .folders()
                .iter()
                .zip(&children)
                .filter(|(folder, _)| folder.parent.is_none())
                .map(|(folder, children)| {
                    TreeNode::new(folder.id, folder.name.as_str()).children(children)
                })
                .collect();
            TreeView::new("folders", state, &roots)
                .accessible_name("Folders")
                .selection_mode(ListSelectionMode::Single)
                .row_height(ROW_HEIGHT)
                .max_height(ui.available_height())
                .show_with(ui, |ui, node, _| row(ui, mailbox, node));
        });
}

/// Leaf nodes under `parent`; this sketch nests one level deep.
fn nodes<'a>(
    folders: &'a [Folder],
    parent: Option<FolderId>,
    grandchildren: &'a [TreeNode<'a, FolderId>],
) -> Vec<TreeNode<'a, FolderId>> {
    folders
        .iter()
        .filter(|folder| folder.parent == parent)
        .map(|folder| TreeNode::new(folder.id, folder.name.as_str()).children(grandchildren))
        .collect()
}

fn row(ui: &mut Ui, mailbox: &Mailbox, node: &TreeNode<'_, FolderId>) {
    let unread = mailbox.unread(node.key);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if unread > 0 {
            ui.add(Badge::new(unread.to_string()).variant(BadgeVariant::Primary));
        }
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            text::line(
                ui,
                &node.label,
                TextSize::Sm,
                unread > 0,
                TextColor::Foreground,
            );
        });
    });
}
