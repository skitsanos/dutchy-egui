use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, Card, CardPadding, CardVariant, ControlSize,
    Progress, ProgressSize, ProgressVariant, Skeleton, SkeletonGroup, SkeletonVariant, Text,
    TextColor, TextSize,
    tokens::{loading, space},
};
use egui::Ui;

pub struct LoadingState {
    pub value: f64,
    pub ready: bool,
    pub opens: u32,
}

impl Default for LoadingState {
    fn default() -> Self {
        Self {
            value: 64.0,
            ready: false,
            opens: 0,
        }
    }
}

pub fn progress(ui: &mut Ui, state: &mut LoadingState) {
    ui.add(Text::label("Application essentials / progress").color(TextColor::Primary));
    ui.add(Text::heading("Every step, clearly.").size(TextSize::Xl4));
    ui.add(
        Text::new("Show known completion or keep uncertainty explicit.").color(TextColor::Muted),
    );
    section(ui, "01", "Application-owned progress", |ui| {
        ui.add(
            Progress::new(state.value)
                .label("Workspace import")
                .id_salt("import")
                .show_value(true)
                .variant(if state.value >= 100.0 {
                    ProgressVariant::Success
                } else {
                    ProgressVariant::Primary
                }),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(
                    Button::new("Subtract 10")
                        .variant(ButtonVariant::Outline)
                        .size(ControlSize::Small),
                )
                .clicked()
            {
                state.value = (state.value - 10.0).max(0.0);
            }
            if ui
                .add(Button::new("Add 10").size(ControlSize::Small))
                .clicked()
            {
                state.value = (state.value + 10.0).min(100.0);
            }
            if ui
                .add(
                    Button::new("Reset progress")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Small),
                )
                .clicked()
            {
                state.value = 0.0;
            }
        });
        if state.value >= 100.0 {
            Alert::new("complete", "The example reached completion.")
                .variant(AlertVariant::Success)
                .show(ui);
        }
        ui.add_space(space::LG);
        ui.add(
            Progress::new(420.0)
                .max(1200.0)
                .label("420 of 1,200 records")
                .show_value(true),
        );
    });
    section(ui, "02", "Track sizes", |ui| {
        for (size, label) in [
            (ProgressSize::Small, "Small / 4 points"),
            (ProgressSize::Medium, "Medium / 8 points"),
            (ProgressSize::Large, "Large / 16 points"),
        ] {
            ui.add(Progress::new(64.0).size(size).label(label).show_value(true));
        }
    });
    section(ui, "03", "Intent", |ui| {
        for variant in ProgressVariant::ALL {
            ui.add(
                Progress::new(64.0)
                    .variant(variant)
                    .label(variant.name())
                    .show_value(true),
            );
        }
    });
    section(ui, "04", "Unknown completion", |ui| {
        ui.add(
            Progress::indeterminate()
                .label("Discovering files")
                .show_value(true)
                .id_salt("discovery"),
        );
        ui.add(
            Text::new("Reduced motion keeps a static segment without implying a percentage.")
                .size(TextSize::Sm)
                .color(TextColor::Muted),
        );
    });
}

pub fn skeletons(ui: &mut Ui, state: &mut LoadingState) {
    ui.add(Text::label("Application essentials / skeletons").color(TextColor::Primary));
    ui.add(Text::heading("Make room for content.").size(TextSize::Xl4));
    ui.add(
        Text::new("Quiet placeholders preserve structure while your application loads.")
            .color(TextColor::Muted),
    );
    section(ui, "01", "Loading to content", |ui| {
        let toggle = if state.ready {
            "Show placeholders"
        } else {
            "Show loaded content"
        };
        if ui
            .add(
                Button::new(toggle)
                    .variant(ButtonVariant::Outline)
                    .size(ControlSize::Small),
            )
            .clicked()
        {
            state.ready = !state.ready;
        }
        Card::new().variant(CardVariant::Bordered).padding(CardPadding::Small).show(ui, |ui| {
            ui.set_min_height(loading::CARD_HEIGHT);
            if state.ready {
                Card::title(ui, "Workspace ready");
                Card::description(ui, "Your workspace content is ready. The surrounding card keeps its reserved height.");
                ui.add_space(space::LG);
                if ui.add(Button::new("Open workspace").size(ControlSize::Small)).clicked() { state.opens += 1; }
                ui.add(Text::label(format!("Opened {} times", state.opens)).color(TextColor::Muted));
            } else {
                SkeletonGroup::new("workspace", "Loading workspace").show(ui, |ui| {
                    ui.add(Skeleton::new().variant(SkeletonVariant::Title));
                    ui.add(Skeleton::new().count(2));
                    ui.add_space(space::LG);
                    ui.add(Skeleton::new().width(space::PAGE * 2.0).height(ControlSize::Small.height()));
                });
            }
        });
    });
    section(ui, "02", "Shape primitives", |ui| {
        SkeletonGroup::new("shapes", "Loading shape preview")
            .announce(false)
            .show(ui, |ui| {
                ui.add(Skeleton::new().variant(SkeletonVariant::Title));
                ui.add(Skeleton::new().count(2));
                ui.add(Skeleton::new().variant(SkeletonVariant::Avatar));
                ui.add(Skeleton::new().variant(SkeletonVariant::TableRow));
            });
    });
    section(ui, "03", "Card and list patterns", |ui| {
        if ui.available_width() >= 640.0 {
            ui.columns(2, |columns| {
                SkeletonGroup::new("card", "Loading card").show_card(&mut columns[0]);
                SkeletonGroup::new("list", "Loading list").show_list(&mut columns[1], 2);
            });
        } else {
            SkeletonGroup::new("card", "Loading card").show_card(ui);
            ui.add_space(space::LG);
            SkeletonGroup::new("list", "Loading list").show_list(ui, 2);
        }
    });
    section(ui, "04", "Table pattern", |ui| {
        SkeletonGroup::new("table", "Loading table").show_table(ui, 3);
    });
    section(ui, "05", "Content pattern", |ui| {
        SkeletonGroup::new("content", "Loading article").show_content(ui);
    });
}
