//! Table surfaces, custom cells, state bodies and public-API host record workflows.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{
    Accent, Badge, BadgeVariant, Button, ButtonVariant, ControlSize, DataTable, Family, Mode,
    TableAlign, TableColumn, TableColumnWidth, TableDensity, TableHeader, TableSort,
    TableSortDirection, Text, TextSize, Theme, Weight,
};
use egui::accesskit::Role;
use egui_kittest::{
    Harness, SnapshotResults,
    kittest::{NodeT, Queryable},
};
struct Record {
    id: u8,
    name: &'static str,
    active: bool,
    amount: u32,
}
fn matrix(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Read. Compare. Decide."));
    let rows = [
        Record {
            id: 1,
            name: "Alice Example",
            active: true,
            amount: 1200,
        },
        Record {
            id: 2,
            name: "Bob Example",
            active: true,
            amount: 800,
        },
        Record {
            id: 3,
            name: "Charlie Example",
            active: false,
            amount: 2400,
        },
        Record {
            id: 4,
            name: "Dana Example",
            active: true,
            amount: 1600,
        },
    ];
    let columns = [
        TableColumn::new(0, "Name")
            .sortable(true)
            .width(TableColumnWidth::Flexible(180.0)),
        TableColumn::new(1, "Status").width(TableColumnWidth::Fixed(140.0)),
        TableColumn::new(2, "Amount")
            .sortable(true)
            .align(TableAlign::Right)
            .width(TableColumnWidth::Fixed(140.0)),
        TableColumn::new(3, "Actions")
            .align(TableAlign::Right)
            .width(TableColumnWidth::Fixed(130.0)),
    ];
    let sort = TableSort {
        column: 2,
        direction: TableSortDirection::Ascending,
    };
    DataTable::new("members", &columns, &rows)
        .accessible_name("Members")
        .sort(Some(&sort))
        .striped(true)
        .selection(|row| row.id == 1)
        .row_enabled(|row| row.id != 3)
        .max_height(280.0)
        .show(
            ui,
            |row| row.id,
            |ui, row, column| match column {
                0 => {
                    ui.add(Text::new(row.name).size(TextSize::Sm).weight(Weight::Bold));
                }
                1 => {
                    ui.add(
                        Badge::new(if row.active { "Active" } else { "Inactive" }).variant(
                            if row.active {
                                BadgeVariant::Success
                            } else {
                                BadgeVariant::Muted
                            },
                        ),
                    );
                }
                2 => {
                    ui.add(
                        Text::new(format!("${}", row.amount))
                            .family(Family::Mono)
                            .size(TextSize::Sm),
                    );
                }
                _ => {
                    let response = ui.add(
                        Button::new("Edit")
                            .size(ControlSize::Small)
                            .variant(ButtonVariant::Ghost),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            Role::Button,
                            ui.is_enabled(),
                            format!("Edit {}", row.name),
                        )
                    });
                }
            },
        );
    ui.add_space(24.0);
    let compact = [
        TableColumn::new(0, "Product").width(TableColumnWidth::Flexible(200.0)),
        TableColumn::new(1, "Price")
            .align(TableAlign::Right)
            .width(TableColumnWidth::Fixed(140.0)),
    ];
    DataTable::new("inverse", &compact, &rows[..3])
        .accessible_name("Inverse compact")
        .density(TableDensity::Compact)
        .header(TableHeader::Inverse)
        .striped(true)
        .show(
            ui,
            |row| row.id,
            |ui, row, column| {
                ui.add(
                    Text::new(if *column == 0 {
                        row.name.to_owned()
                    } else {
                        format!("${}", row.amount)
                    })
                    .size(TextSize::Sm),
                );
            },
        );
    ui.add_space(24.0);
    let empty: [Record; 0] = [];
    DataTable::new("empty", &compact, &empty)
        .accessible_name("Empty table")
        .show(ui, |row| row.id, |_, _, _| unreachable!());
    ui.add_space(24.0);
    DataTable::new("loading", &columns, &rows)
        .accessible_name("Loading table")
        .loading(true)
        .max_height(160.0)
        .show(ui, |row| row.id, |_, _, _| unreachable!());
}
fn harness(theme: Theme, scale: f32, size: [f32; 2]) -> Harness<'static> {
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(if theme.mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui(matrix)
}
#[test]
fn surfaces_states_and_focus_render_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "table-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "table-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "table-purple-2x"),
    ] {
        let mut h = harness(
            Theme::new(accent, mode).with_reduced_motion(true),
            scale,
            [900.0, 1400.0],
        );
        h.run_steps(5);
        h.get_by_role_and_label(Role::Table, "Members")
            .get_by_label("Sort by Name")
            .focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
#[test]
fn row_hover_is_visible_without_selection_or_sorting() {
    let mut h = harness(
        Theme::default().with_reduced_motion(true),
        1.0,
        [900.0, 1400.0],
    );
    h.run_steps(4);
    let before = h.render().unwrap();
    h.get_by_role_and_label(Role::Table, "Members")
        .get_by_label("Bob Example")
        .hover();
    h.run_steps(2);
    assert_ne!(before, h.render().unwrap());
    h.snapshot("table-row-hover");
}
#[test]
fn narrow_viewport_contains_the_table_chrome_and_empty_guidance() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    let mut h = harness(
        Theme::default().with_reduced_motion(true),
        1.0,
        [280.0, 1800.0],
    );
    h.run_steps(5);
    for name in [
        "Members viewport",
        "Empty table viewport",
        "Loading table viewport",
    ] {
        let rect = h.get_by_role_and_label(Role::Group, name).rect();
        assert!(
            rect.left() >= 8.0 && rect.right() <= 272.5,
            "{name}: {rect:?}"
        );
    }
    let text = h
        .get_by_role_and_label(Role::Heading, "No data available")
        .rect();
    assert!(text.left() >= 8.0 && text.right() <= 272.5, "{text:?}");
    let guidance = h
        .get_by_label("Try adjusting your search or filters.")
        .rect();
    assert!(
        guidance.left() >= 8.0 && guidance.right() <= 272.5,
        "{guidance:?}"
    );
    h.render()
        .unwrap()
        .save("target/visual-proof/table-matrix-narrow.png")
        .unwrap();
}
fn example(size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::DataTables,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn showcase_host_filter_sort_selection_actions_and_loading_are_independent() {
    let mut h = example([1200.0, 2200.0]);
    h.get_by_label("View Ada Lovelace").click();
    h.run_steps(3);
    assert_eq!(h.state().tabular.inspections, 1);
    assert!(h.state().tabular.selected.is_empty());
    h.get_by_label("Select row 1").click();
    h.run_steps(3);
    assert!(h.state().tabular.selected.contains(&1));
    h.get_by_label("Sort by Budget").click();
    h.run_steps(4);
    assert_eq!(
        h.state().tabular.sort.as_ref().unwrap().direction,
        TableSortDirection::Ascending
    );
    let rows = h.state().tabular.filtered();
    assert!(rows.windows(2).all(|pair| pair[0].cents <= pair[1].cents));
    h.get_by_label("Sort by Budget").click();
    h.run_steps(3);
    let rows = h.state().tabular.filtered();
    assert!(rows.windows(2).all(|pair| pair[0].cents >= pair[1].cents));
    finish_host_workflow(&mut h);
}
fn finish_host_workflow(h: &mut Harness<'_, gallery::Showcase>) {
    h.get_by_label("Search records").focus();
    h.run_steps(2);
    h.get_by_label("Search records").type_text("zzzz");
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Heading, "No matching records")
            .is_some()
    );
    assert!(h.state().tabular.selected.contains(&1));
    h.get_by_label("Clear search").click();
    h.run_steps(3);
    h.get_by_label("Loading records").click();
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Table, "Workspace records")
            .accesskit_node()
            .data()
            .is_busy()
    );
    h.get_by_label("Loading records").click();
    h.run_steps(3);
    assert!(!h.state().tabular.loading);
    h.get_by_label("Search records").focus();
    h.run_steps(2);
    h.get_by_label("Search records").type_text("Ada");
    h.run_steps(3);
    assert_eq!(h.state().tabular.filtered().len(), 1);
    assert!(h.query_by_label("View Ada Lovelace").is_some());
}
#[test]
fn showcase_renders_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [([1200.0, 2200.0], "desktop"), ([480.0, 2800.0], "narrow")] {
        let mut h = example(size);
        h.run_steps(4);
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/table-{name}.png"))
            .unwrap();
    }
}
