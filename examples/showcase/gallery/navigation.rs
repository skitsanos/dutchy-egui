use super::section;
use dutchy_egui::{
    BreadcrumbItem, BreadcrumbSeparator, Breadcrumbs, Button, ButtonVariant, Pagination,
    PaginationVariant, Text, TextColor,
    tokens::{border, space},
};
use egui::{Color32, Painter, Rect, Stroke, Ui, pos2};

pub struct NavigationState {
    pub depth: usize,
    pub ancestors_open: bool,
    pub destination: Option<usize>,
    pub page: usize,
    pub window_page: usize,
    pub compact_page: usize,
    pub requests: usize,
}
impl Default for NavigationState {
    fn default() -> Self {
        Self {
            depth: 6,
            ancestors_open: false,
            destination: None,
            page: 1,
            window_page: 7,
            compact_page: 3,
            requests: 0,
        }
    }
}
fn path() -> [BreadcrumbItem<'static, usize>; 7] {
    let labels = [
        "Home",
        "Studio",
        "Workspaces",
        "Design team",
        "Projects",
        "Library",
        "Navigation patterns",
    ];
    std::array::from_fn(|index| BreadcrumbItem::new(index, labels[index]))
}
fn home(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(border::STANDARD, color);
    painter.add(egui::Shape::line(
        vec![rect.left_center(), rect.center_top(), rect.right_center()],
        stroke,
    ));
    let inset = rect.width() / 4.0;
    painter.add(egui::Shape::line(
        vec![
            pos2(rect.left() + inset, rect.center().y),
            pos2(rect.left() + inset, rect.bottom()),
            pos2(rect.right() - inset, rect.bottom()),
            pos2(rect.right() - inset, rect.center().y),
        ],
        stroke,
    ));
}
pub fn breadcrumbs(ui: &mut Ui, state: &mut NavigationState) {
    ui.add(Text::heading("Know where you are."));
    ui.add(Text::new(
        "A clear hierarchy, with navigation owned by the application.",
    ));
    section(ui, "01", "Standard / slash separators", |ui| {
        let items = &path()[..4];
        let result = Breadcrumbs::new("slash", items)
            .accessible_name("Standard hierarchy")
            .separator(BreadcrumbSeparator::Slash)
            .show(ui);
        if let Some(value) = result.navigate {
            state.destination = Some(value);
        }
    });
    section(ui, "02", "Chevron / icon root", |ui| {
        let mut items = path();
        items[0] = items[0].clone().icon(home);
        let result = Breadcrumbs::new("chevron", &items[..4])
            .accessible_name("Icon hierarchy")
            .show(ui);
        if let Some(value) = result.navigate {
            state.destination = Some(value);
        }
    });
    section(ui, "03", "Collapsed ancestors / host navigation", |ui| {
        let reset = ui.add(Button::new("Restore deep path").variant(ButtonVariant::Outline));
        if reset.clicked() {
            state.depth = 6;
        }
        ui.add_space(space::SM);
        let items = path();
        let result = Breadcrumbs::new("collapsed", &items[..=state.depth])
            .accessible_name("Project hierarchy")
            .collapse(2, &mut state.ancestors_open)
            .show(ui);
        if let Some(value) = result.navigate {
            state.depth = value;
            state.destination = Some(value);
            reset.request_focus();
        }
        ui.add_space(space::SM);
        ui.add(Text::new("The last item is current. Hidden ancestors remain reachable by keyboard through the ellipsis menu.").color(TextColor::Muted));
    });
    section(ui, "04", "Muted navigation bar", |ui| {
        let result = Breadcrumbs::new("bar", &path()[..3])
            .bar(true)
            .accessible_name("Bar hierarchy")
            .show(ui);
        if let Some(value) = result.navigate {
            state.destination = Some(value);
        }
    });
    ui.add_space(space::LG);
    ui.add(
        Text::new(format!(
            "Last destination request: {}",
            state
                .destination
                .map_or_else(|| "None".into(), |id| path()[id].label.to_string())
        ))
        .color(TextColor::Muted),
    );
}
pub fn pagination(ui: &mut Ui, state: &mut NavigationState) {
    ui.add(Text::heading("One page at a time."));
    ui.add(Text::new(
        "Predictable ranges, clear endpoints, and host-owned data.",
    ));
    section(ui, "01", "Record browser / numbered pages", |ui| {
        ui.add(Text::label(format!(
            "Records {}–{} of 25",
            (state.page - 1) * 5 + 1,
            state.page * 5
        )));
        for record in (state.page - 1) * 5 + 1..=state.page * 5 {
            ui.add(Text::new(format!("Project {record:02} / sample record")));
        }
        ui.add_space(space::LG);
        let result = Pagination::new("records", state.page, 5)
            .accessible_name("Record pages")
            .show(ui);
        if let Some(page) = result.requested_page {
            state.page = page;
            state.requests += 1;
        }
    });
    section(ui, "02", "Large range / ellipses", |ui| {
        let result = Pagination::new("window", state.window_page, 20)
            .accessible_name("Library pages")
            .show(ui);
        if let Some(page) = result.requested_page {
            state.window_page = page;
            state.requests += 1;
        }
    });
    section(ui, "03", "Compact / page summary", |ui| {
        let result = Pagination::new("compact", state.compact_page, 12)
            .accessible_name("Compact pages")
            .variant(PaginationVariant::Compact)
            .show(ui);
        if let Some(page) = result.requested_page {
            state.compact_page = page;
            state.requests += 1;
        }
    });
    section(ui, "04", "Empty / disabled navigation", |ui| {
        Pagination::new("empty", 0, 0)
            .accessible_name("Empty pages")
            .show(ui);
        ui.add_space(space::SM);
        Pagination::new("loading", 2, 5)
            .accessible_name("Loading pages")
            .enabled(false)
            .show(ui);
        ui.add(Text::new("The host can disable navigation while loading. An empty range never invents page 1.").color(TextColor::Muted));
    });
    ui.add_space(space::LG);
    ui.add(
        Text::label(format!("Host navigation requests: {}", state.requests))
            .color(TextColor::Muted),
    );
}
