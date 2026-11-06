use super::section;
use dutchy_egui::{
    Button, ButtonVariant, Card, CardVariant, ControlSize, EmptyState, EmptyStateIcon,
    EmptyStateVariant, Stat, StatSize, StatVariant, StatsGrid, StatsLayout, Text, TextColor,
    TextSize, Trend, TrendDirection, TrendSentiment, tokens::space,
};
use egui::Ui;

pub struct InsightsState {
    pub refreshes: u32,
    pub projects: u32,
    pub templates: u32,
    pub filters_active: bool,
    pub entries: u32,
    pub widgets: u32,
}

impl Default for InsightsState {
    fn default() -> Self {
        Self {
            refreshes: 0,
            projects: 0,
            templates: 0,
            filters_active: true,
            entries: 0,
            widgets: 0,
        }
    }
}

fn toolbar(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add(
        Button::new(label)
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small),
    )
}

pub fn stats(ui: &mut Ui, state: &mut InsightsState) {
    ui.add(Text::label("Application essentials / stats").color(TextColor::Primary));
    ui.add(Text::heading("Numbers with context.").size(TextSize::Xl4));
    ui.add(
        Text::new("Sample metrics with clear labels and explicit change interpretation.")
            .color(TextColor::Muted),
    );
    section(ui, "01", "Monthly performance", |ui| {
        if toolbar(ui, "Refresh sample metrics").clicked() {
            state.refreshes += 1;
        }
        let revenue = if state.refreshes == 0 {
            "$48,290"
        } else {
            "$52,460"
        };
        let values = [
            Stat::new("Revenue", revenue).trend(
                Trend::new("+12.5%", TrendDirection::Up).sentiment(TrendSentiment::Positive),
            ),
            Stat::new("Orders", "1,429").variant(StatVariant::Muted),
            Stat::new("Conversion", "3.24%").variant(StatVariant::Dark),
            Stat::new("Average order", "$33.80").variant(StatVariant::Primary),
        ];
        ui.add(StatsGrid::new("monthly", &values).accessible_name("Monthly performance"));
        ui.add(
            Text::label(format!("Sample refreshes: {}", state.refreshes)).color(TextColor::Muted),
        );
    });
    section(ui, "02", "Compact overview", |ui| {
        let values = [
            Stat::new("Users", "2,847"),
            Stat::new("Views", "14,209"),
            Stat::new("Uptime", "99.9%"),
            Stat::new("Average load", "3.2s"),
        ];
        ui.add(
            StatsGrid::new("compact", &values)
                .layout(StatsLayout::Compact)
                .accessible_name("Compact overview"),
        );
    });
    section(ui, "03", "Trend context", |ui| {
        let draw = |ui: &mut Ui, index| {
            if index == 0 {
                ui.add(
                    Stat::new("Error rate", "0.8%")
                        .accented(true)
                        .trend(
                            Trend::new("-18%", TrendDirection::Down)
                                .sentiment(TrendSentiment::Positive)
                                .comparison("fewer failed requests"),
                        )
                        .description("A decreasing value can be a positive outcome."),
                );
            } else {
                ui.add(
                    Stat::new("Queue depth", "42")
                        .size(StatSize::Small)
                        .trend(Trend::new("0%", TrendDirection::Flat))
                        .description("No change from the previous period."),
                );
            }
        };
        if ui.available_width() >= 640.0 {
            ui.columns(2, |columns| {
                draw(&mut columns[0], 0);
                draw(&mut columns[1], 1);
            });
        } else {
            draw(ui, 0);
            ui.add_space(space::LG);
            draw(ui, 1);
        }
    });
}

pub fn empty_states(ui: &mut Ui, state: &mut InsightsState) {
    ui.add(Text::label("Application essentials / empty states").color(TextColor::Primary));
    ui.add(Text::heading("A useful next step.").size(TextSize::Xl4));
    ui.add(
        Text::new("Actions update this example; your application decides what happens next.")
            .color(TextColor::Muted),
    );
    section(ui, "01", "No content yet", |ui| {
        let reset = toolbar(ui, "Reset projects");
        if reset.clicked() {
            state.projects = 0;
        }
        if state.projects == 0 {
            let result = EmptyState::new("projects", "No projects yet")
                .icon(EmptyStateIcon::Inbox)
                .description("Create your first project or start from a template.")
                .action("Create project")
                .secondary_action("Browse project templates")
                .announce(true)
                .show(ui);
            if result.action_clicked {
                state.projects += 1;
                reset.request_focus();
            }
            if result.secondary_clicked {
                state.templates += 1;
            }
        } else {
            Card::new().variant(CardVariant::Bordered).show(ui, |ui| {
                Card::title(ui, "My first project");
                Card::description(ui, "Your example project is ready.");
                ui.add(Stat::new("Projects", state.projects.to_string()).size(StatSize::Small));
            });
        }
    });
    section(ui, "02", "Search with no matches", |ui| {
        let apply = toolbar(ui, "Apply sample filters");
        if apply.clicked() {
            state.filters_active = true;
        }
        if state.filters_active {
            let result = EmptyState::new("search", "No results found")
                .variant(EmptyStateVariant::Search)
                .description("Try another search term or clear your filters to see all projects.")
                .action("Clear search")
                .announce(true)
                .show(ui);
            if result.action_clicked {
                state.filters_active = false;
                apply.request_focus();
            }
        } else {
            Card::new().variant(CardVariant::Muted).show(ui, |ui| {
                Card::title(ui, "All projects");
                ui.add(Text::new("Project Phoenix"));
                ui.add(Text::new("Project Atlas"));
            });
        }
    });
    section(ui, "03", "Empty table body", |ui| {
        let clear = toolbar(ui, "Clear sample entries");
        if clear.clicked() {
            state.entries = 0;
        }
        if state.entries == 0 {
            let result = EmptyState::new("table", "No entries yet")
                .variant(EmptyStateVariant::Table)
                .description("Create your first entry to get started.")
                .action("Add entry")
                .show(ui);
            if result.action_clicked {
                state.entries += 1;
                clear.request_focus();
            }
        } else {
            ui.add(Text::new("Sample entry created"));
        }
    });
    section(ui, "04", "Dashboard setup", |ui| {
        let reset = toolbar(ui, "Reset dashboard");
        if reset.clicked() {
            state.widgets = 0;
        }
        if state.widgets == 0 {
            let result = EmptyState::new("dashboard", "Set up your dashboard")
                .variant(EmptyStateVariant::Dashboard)
                .description("Add a widget to monitor a key metric, or start with a template.")
                .action("Add widget")
                .secondary_action("Browse dashboard templates")
                .show(ui);
            if result.action_clicked {
                state.widgets += 1;
                reset.request_focus();
            }
            if result.secondary_clicked {
                state.templates += 1;
            }
        } else {
            ui.add(Stat::new("Dashboard widgets", state.widgets.to_string()).accented(true));
        }
    });
    ui.add(
        Text::label(format!("Template browser requests: {}", state.templates))
            .color(TextColor::Muted),
    );
}
