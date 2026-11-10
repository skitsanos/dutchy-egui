//! Private tree adapter. Scrolling, row identity and selection stay in List.
use crate::tokens::{border, list, tree};
use egui::{Id, Rect, Response, Stroke, Ui, pos2};

#[derive(Clone, Copy)]
pub(crate) struct HierarchyRow {
    pub depth: usize,
    pub parent: Option<usize>,
    pub children: usize,
    pub position: usize,
    pub expanded: Option<bool>,
}

pub(crate) fn root(ui: &Ui, salt: egui::IdSalt) -> Id {
    ui.make_persistent_id(("dutchy-list", salt))
}
pub(crate) fn disclosure(rect: Rect, depth: usize) -> Rect {
    let left = (rect.left() + list::PADDING + depth as f32 * tree::INDENT).min(rect.right());
    Rect::from_min_max(
        pos2(left, rect.top()),
        pos2(
            (left + tree::DISCLOSURE_SLOT).min(rect.right()),
            rect.bottom(),
        ),
    )
}
pub(crate) fn clicked(response: &Response, meta: Option<HierarchyRow>) -> bool {
    meta.is_some_and(|meta| {
        meta.expanded.is_some()
            && response
                .interact_pointer_pos()
                .is_some_and(|pos| disclosure(response.rect, meta.depth).contains(pos))
    })
}
pub(crate) fn paint(ui: &Ui, rect: Rect, meta: HierarchyRow) {
    let Some(expanded) = meta.expanded else {
        return;
    };
    let rect = disclosure(rect, meta.depth);
    if rect.width() < tree::DISCLOSURE_SIZE {
        return;
    }
    let c = rect.center();
    let r = tree::DISCLOSURE_SIZE / 2.0;
    let points = if expanded {
        [
            pos2(c.x - r, c.y - r / 2.0),
            pos2(c.x, c.y + r / 2.0),
            pos2(c.x + r, c.y - r / 2.0),
        ]
    } else {
        [
            pos2(c.x - r / 2.0, c.y - r),
            pos2(c.x + r / 2.0, c.y),
            pos2(c.x - r / 2.0, c.y + r),
        ]
    };
    ui.painter().add(egui::Shape::line(
        points.to_vec(),
        Stroke::new(
            border::STANDARD,
            crate::Theme::from_context(ui.ctx()).palette.foreground,
        ),
    ));
}
pub(crate) fn metadata(response: &Response, meta: HierarchyRow, read_only: bool) {
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::TreeItem);
        // AccessKit levels are zero-based; adapters add one for the platform.
        node.set_level(meta.depth);
        node.set_position_in_set(meta.position);
        if let Some(expanded) = meta.expanded {
            node.set_expanded(expanded);
            if response.enabled() && !read_only {
                node.add_action(if expanded {
                    egui::accesskit::Action::Collapse
                } else {
                    egui::accesskit::Action::Expand
                });
            }
        }
    });
}

pub(super) struct View<'a> {
    pub metas: &'a [HierarchyRow],
    pub range: std::ops::Range<usize>,
    pub active: Option<usize>,
    pub origin: f32,
    pub height: f32,
    pub parent: Id,
}
/// Build parents before descendants, keeping virtualized ancestor nodes alive.
pub(super) fn prepare<K: Clone + Eq + std::hash::Hash>(
    ui: &mut Ui,
    model: &super::model::Model<'_, K>,
    view: View<'_>,
) -> std::collections::HashMap<usize, Response> {
    let mut needed: std::collections::HashSet<_> = view.range.clone().collect();
    if let Some(i) = view.active {
        needed.insert(i);
    }
    for index in needed.clone() {
        let mut parent = view.metas[index].parent;
        while let Some(i) = parent {
            needed.insert(i);
            parent = view.metas[i].parent;
        }
    }
    let mut ordered: Vec<_> = needed.into_iter().collect();
    ordered.sort_unstable();
    let mut responses = std::collections::HashMap::new();
    for index in ordered {
        let meta = view.metas[index];
        let rect = Rect::from_min_size(
            pos2(
                ui.max_rect().left(),
                view.origin + index as f32 * view.height,
            ),
            egui::vec2(ui.available_width(), view.height),
        );
        let parent = meta
            .parent
            .map_or(view.parent, |p| model.ids[p].with("tree-children"));
        let response = super::paint::interact_parent(
            ui,
            model.ids[index],
            rect,
            view.active == Some(index),
            model.is_enabled(index),
            Some(parent),
        );
        // Establish the node before adding its group. Full row metadata follows
        // interaction, once the final selected set is known.
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::TreeItem);
            node.set_label(model.items[index].label.as_ref());
        });
        if meta.expanded == Some(true) {
            group(ui, &response, meta.children);
        }
        responses.insert(index, response);
    }
    responses
}
fn group(ui: &mut Ui, parent: &Response, count: usize) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .accessibility_parent(parent.id)
            .max_rect(parent.rect),
    );
    if !parent.enabled() {
        child.disable();
    }
    let response = child.interact(
        parent.rect,
        parent.id.with("tree-children"),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::new(egui::accesskit::Role::Group));
    response
        .ctx
        .accesskit_node_builder(response.id, |node| node.set_size_of_set(count));
}
