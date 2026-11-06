mod components;
mod forms;
mod loading;
mod surfaces;
mod typography;

pub use forms::FormState;
pub use loading::LoadingState;
pub use surfaces::SurfacesState;

use dutchy_egui::tokens::{border, space};
use dutchy_egui::{
    Accent, Button, ButtonVariant, ControlSize, Mode, Text, TextColor, TextSize, Theme,
};
use egui::{Frame, Margin, Stroke, Ui};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Overview,
    Typography,
    Forms,
    Cards,
    Alerts,
    Progress,
    Skeletons,
    Buttons,
    Badges,
    Motion,
    Tokens,
}

impl Page {
    const ALL: [Self; 11] = [
        Self::Overview,
        Self::Typography,
        Self::Forms,
        Self::Cards,
        Self::Alerts,
        Self::Progress,
        Self::Skeletons,
        Self::Buttons,
        Self::Badges,
        Self::Motion,
        Self::Tokens,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Typography => "Typography",
            Self::Forms => "Forms",
            Self::Cards => "Cards",
            Self::Alerts => "Alerts",
            Self::Progress => "Progress",
            Self::Skeletons => "Skeletons",
            Self::Buttons => "Buttons",
            Self::Badges => "Badges",
            Self::Motion => "Motion",
            Self::Tokens => "Tokens",
        }
    }
}

pub struct Showcase {
    pub accent: Accent,
    pub mode: Mode,
    pub reduced_motion: bool,
    pub page: Page,
    pub clicks: u32,
    pub badge_visible: bool,
    pub native_text: String,
    pub native_checked: bool,
    pub forms: FormState,
    pub surfaces: SurfacesState,
    pub loading: LoadingState,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            accent: Accent::Orange,
            mode: Mode::Light,
            reduced_motion: false,
            page: Page::Overview,
            clicks: 0,
            badge_visible: true,
            native_text: "Native egui, Dutchy style".into(),
            native_checked: true,
            forms: FormState::default(),
            surfaces: SurfacesState::default(),
            loading: LoadingState::default(),
        }
    }
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

impl Showcase {
    pub fn theme(&self) -> Theme {
        Theme::new(self.accent, self.mode).with_reduced_motion(self.reduced_motion)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.forms.tick(ui.ctx());
        self.header(ui);
        let wide = ui.available_width() >= 820.0;
        if wide {
            self.sidebar(ui);
        }
        egui::CentralPanel::default()
            .frame(frame(self.theme(), 24))
            .show(ui, |ui| {
                if !wide {
                    ui.horizontal_wrapped(|ui| self.navigation(ui, false));
                    ui.add_space(space::LG);
                }
                egui::ScrollArea::vertical()
                    .id_salt("showcase-content")
                    .show(ui, |ui| {
                        ui.add_space(space::XS);
                        match self.page {
                            Page::Overview => components::overview(ui, self),
                            Page::Typography => typography::show(ui),
                            Page::Forms => forms::show(ui, &mut self.forms),
                            Page::Cards => surfaces::cards(ui, &mut self.surfaces),
                            Page::Alerts => surfaces::alerts(ui, &mut self.surfaces),
                            Page::Progress => loading::progress(ui, &mut self.loading),
                            Page::Skeletons => loading::skeletons(ui, &mut self.loading),
                            Page::Buttons => components::buttons(ui, self),
                            Page::Badges => components::badges(ui, self),
                            Page::Motion => components::motion(ui, self),
                            Page::Tokens => typography::tokens(ui, self.theme()),
                        }
                        ui.add_space(space::SECTION);
                        ui.add(
                            Text::label("Built for clarity. Drawn with egui.")
                                .color(TextColor::Muted),
                        );
                    });
            });
    }

    fn header(&mut self, ui: &mut Ui) {
        let previous = self.theme();
        egui::Panel::top("showcase-header")
            .frame(frame(previous, 16))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add(Text::heading("D. DUTCHY / EGUI").size(TextSize::Xl2));
                    ui.add_space(space::LG);
                    ui.add(
                        Text::label(concat!("Component library · ", env!("CARGO_PKG_VERSION")))
                            .color(TextColor::Muted),
                    );
                });
                ui.add_space(space::SM);
                ui.horizontal_wrapped(|ui| {
                    for (accent, label) in [
                        (Accent::Orange, "Orange"),
                        (Accent::Purple, "Purple"),
                        (Accent::Crimson, "Crimson"),
                    ] {
                        let variant = if accent == self.accent {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Ghost
                        };
                        if ui
                            .add(Button::new(label).variant(variant).size(ControlSize::Small))
                            .clicked()
                        {
                            self.accent = accent;
                        }
                    }
                    let mode_label = if self.mode == Mode::Light {
                        "Dark mode"
                    } else {
                        "Light mode"
                    };
                    if ui
                        .add(
                            Button::new(mode_label)
                                .variant(ButtonVariant::Outline)
                                .size(ControlSize::Small),
                        )
                        .clicked()
                    {
                        self.mode = if self.mode == Mode::Light {
                            Mode::Dark
                        } else {
                            Mode::Light
                        };
                    }
                    ui.checkbox(&mut self.reduced_motion, "Reduce motion");
                });
            });
        if self.theme() != previous {
            self.theme().apply(ui.ctx());
        }
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        egui::Panel::left("showcase-navigation")
            .exact_size(208.0)
            .resizable(false)
            .frame(frame(self.theme(), 16))
            .show(ui, |ui| {
                ui.add(Text::label("The system").color(TextColor::Primary));
                ui.add_space(space::LG);
                self.navigation(ui, true);
                ui.add_space(space::SECTION);
                ui.add(Text::label("01 / Foundation"));
                ui.add(
                    Text::new("Zero radius.\nFlat color.\nBold typography.")
                        .color(TextColor::Muted),
                );
            });
    }

    fn navigation(&mut self, ui: &mut Ui, full_width: bool) {
        for page in Page::ALL {
            let variant = if page == self.page {
                ButtonVariant::Secondary
            } else {
                ButtonVariant::Ghost
            };
            if ui
                .add(
                    Button::new(page.name())
                        .variant(variant)
                        .size(ControlSize::Small)
                        .full_width(full_width),
                )
                .clicked()
            {
                self.page = page;
            }
        }
    }
}

fn frame(theme: Theme, inset: i8) -> Frame {
    Frame::NONE
        .fill(theme.palette.background)
        .inner_margin(Margin::same(inset))
}

fn section(ui: &mut Ui, number: &str, title: &str, contents: impl FnOnce(&mut Ui)) {
    let theme = Theme::from_context(ui.ctx());
    ui.add_space(space::LG);
    Frame::NONE
        .stroke(Stroke::new(border::THIN, theme.palette.border))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(Text::label(format!("{number} / {title}")).color(TextColor::Muted));
            ui.add_space(space::MD);
            contents(ui);
        });
}
