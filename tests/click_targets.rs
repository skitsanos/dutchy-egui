//! Text drawn on a clickable surface must leave the surface its pointer input.
//!
//! egui labels are selectable by default and then sense clicks and drags. A
//! label added after a row, card or option that registered its own click would
//! sit on top of it and take its hover, click and focus. Every showcase page is
//! checked for selectable text inside a clickable control.

#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::Theme;
use egui::accesskit::{Action, Rect, Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use gallery::{Page, Showcase};

struct Target {
    role: Role,
    name: String,
    /// Bounds clipped to every ancestor, as egui clips interaction to the
    /// enclosing scroll viewport.
    visible: Rect,
}

fn clickable(h: &Harness<'_, Showcase>) -> Vec<Target> {
    h.root()
        .children_recursive()
        .filter_map(|node| {
            let node = node.accesskit_node();
            if !node.data().supports_action(Action::Click) || node.is_hidden() {
                return None;
            }
            let mut visible = node.bounding_box()?;
            let mut parent = node.parent();
            while let Some(ancestor) = parent {
                if let Some(bounds) = ancestor.bounding_box() {
                    visible = visible.intersect(bounds);
                }
                parent = ancestor.parent();
            }
            let name = node.label().or_else(|| node.value()).unwrap_or_default();
            Some(Target {
                role: node.role(),
                name,
                visible,
            })
        })
        .collect()
}

fn covers(surface: Rect, text: Rect) -> bool {
    let center = egui::accesskit::Point::new((text.x0 + text.x1) / 2.0, (text.y0 + text.y1) / 2.0);
    surface.area() > 0.0 && text.area() > 0.0 && surface.contains(center)
}

fn showcase(page: Page) -> Harness<'static, Showcase> {
    let mut h = Harness::builder()
        .with_size([1200.0, 2400.0])
        .build_eframe(move |cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            Showcase {
                reduced_motion: true,
                page,
                ..Default::default()
            }
        });
    h.run_steps(4);
    h
}

fn conflicts(page: Page) -> Vec<String> {
    let h = showcase(page);
    let targets = clickable(&h);
    let texts = targets.iter().filter(|t| t.role == Role::Label);
    texts
        .flat_map(|text| {
            targets
                .iter()
                .filter(|surface| surface.role != Role::Label)
                .filter(|surface| covers(surface.visible, text.visible))
                .map(move |surface| {
                    format!(
                        "{}: {:?} {:?} sits under selectable text {:?}",
                        page.name(),
                        surface.role,
                        surface.name,
                        text.name
                    )
                })
        })
        .collect()
}

#[test]
fn no_showcase_page_puts_selectable_text_on_a_clickable_surface() {
    let found: Vec<String> = Page::all().flat_map(conflicts).collect();
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn clicking_a_project_name_in_the_lists_showcase_selects_its_row() {
    let name = "Harbor / project 00001";
    let mut h = showcase(Page::Lists);
    h.get_by(|node| node.role() == Role::Label && node.value().as_deref() == Some(name))
        .click();
    h.run_steps(3);
    let row = h.get_by_role_and_label(Role::ListBoxOption, name);
    assert_eq!(row.accesskit_node().is_selected(), Some(true));
    assert!(row.is_focused());
}
