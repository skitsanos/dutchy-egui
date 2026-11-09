mod actions;
mod collection;
mod colors;
mod command_surfaces;
mod components;
mod disclosures;
mod forms;
mod help;
mod insights;
mod layout;
mod loading;
#[cfg(feature = "markdown")]
mod markdown;
pub(crate) mod media;
mod menus;
mod navigation;
mod notifications;
mod overlays;
mod pages;
mod primitives;
pub(crate) mod sketch;
mod stepping;
mod surfaces;
mod tabbed;
mod tabular;
mod typography;
mod uploads;

pub use forms::FormState;
pub use insights::InsightsState;
pub use loading::LoadingState;
pub use surfaces::SurfacesState;

use dutchy_egui::tokens::{border, space};
use dutchy_egui::{
    Accent, Button, ButtonVariant, ControlSize, Flex, Mode, Text, TextColor, TextSize, Theme,
};
use egui::{Frame, Margin, Stroke, Ui};

pub use pages::Page;

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
    pub primitives: primitives::PrimitivesState,
    pub colors: colors::ColorsState,
    pub collection: collection::CollectionState,
    pub actions: actions::ActionsState,
    pub surfaces: SurfacesState,
    pub loading: LoadingState,
    pub insights: InsightsState,
    pub help: help::HelpState,
    pub layout: layout::LayoutState,
    pub disclosures: disclosures::DisclosureState,
    pub tabbed: tabbed::TabbedState,
    pub menus: menus::MenuState,
    pub navigation: navigation::NavigationState,
    pub stepping: stepping::SteppingState,
    pub overlays: overlays::OverlayState,
    pub notifications: notifications::NotificationState,
    pub tabular: tabular::TableState,
    #[cfg(feature = "markdown")]
    pub markdown: markdown::MarkdownState,
    pub media: media::MediaState,
    pub uploads: uploads::UploadState,
    pub sketch: sketch::SketchState,
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
            primitives: primitives::PrimitivesState::default(),
            colors: colors::ColorsState::default(),
            collection: collection::CollectionState::default(),
            actions: actions::ActionsState::default(),
            surfaces: SurfacesState::default(),
            loading: LoadingState::default(),
            insights: InsightsState::default(),
            help: help::HelpState::default(),
            layout: layout::LayoutState::default(),
            disclosures: disclosures::DisclosureState::default(),
            tabbed: tabbed::TabbedState::default(),
            menus: menus::MenuState::default(),
            navigation: navigation::NavigationState::default(),
            stepping: stepping::SteppingState::default(),
            overlays: overlays::OverlayState::default(),
            notifications: notifications::NotificationState::default(),
            tabular: tabular::TableState::default(),
            #[cfg(feature = "markdown")]
            markdown: markdown::MarkdownState::default(),
            media: media::MediaState::default(),
            uploads: uploads::UploadState::default(),
            sketch: sketch::SketchState::default(),
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
        self.uploads.tick(ui.ctx());
        self.sketch.tick(ui.ctx());
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
                // Each page keeps its own offset, so a newly opened page starts at the top.
                egui::ScrollArea::vertical()
                    .id_salt(("showcase-content", self.page))
                    .show(ui, |ui| {
                        ui.add_space(space::XS);
                        match self.page {
                            Page::Overview => components::overview(ui, self),
                            Page::Typography => typography::show(ui),
                            Page::Forms => forms::show(ui, &mut self.forms),
                            Page::Controls => primitives::show(ui, &mut self.primitives),
                            Page::Colors => colors::show(ui, &mut self.colors),
                            Page::MediaCards => self.collection.cards(ui),
                            Page::SearchInput => self.collection.search(ui),
                            Page::Cards => surfaces::cards(ui, &mut self.surfaces),
                            Page::Alerts => surfaces::alerts(ui, &mut self.surfaces),
                            Page::Progress => loading::progress(ui, &mut self.loading),
                            Page::Skeletons => loading::skeletons(ui, &mut self.loading),
                            Page::Stats => insights::stats(ui, &mut self.insights),
                            Page::EmptyStates => insights::empty_states(ui, &mut self.insights),
                            Page::Tooltips => help::show(ui, &mut self.help),
                            Page::Layout => layout::show(ui, &mut self.layout),
                            Page::Accordions => disclosures::show(ui, &mut self.disclosures),
                            Page::Tabs => tabbed::show(ui, &mut self.tabbed),
                            Page::Menus => menus::menus(ui, &mut self.menus),
                            Page::Dropdowns => menus::dropdowns(ui, &mut self.menus),
                            Page::Breadcrumbs => navigation::breadcrumbs(ui, &mut self.navigation),
                            Page::Pagination => navigation::pagination(ui, &mut self.navigation),
                            Page::Stepper => stepping::show(ui, &mut self.stepping),
                            Page::Modals => self.overlays.modals(ui),
                            Page::Drawers => self.overlays.drawers(ui),
                            Page::Toasts => self.notifications.show(ui),
                            Page::DataTables => self.tabular.show(ui),
                            Page::Images => self.media.images(ui),
                            Page::IconButtons => self.actions.icons(ui),
                            Page::ToggleButtons => self.actions.toggles(ui),
                            Page::Toolbars => self.actions.toolbars(ui, &mut self.menus.rich),
                            Page::Galleries => self.media.galleries(ui),
                            Page::Lightboxes => self.media.lightboxes(ui),
                            #[cfg(feature = "markdown")]
                            Page::Markdown => markdown::show(ui, &mut self.markdown),
                            Page::FileUploads => self.uploads.show(ui),
                            Page::Scribble => self.sketch.show(ui),
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
        self.media.render(ui.ctx());
        self.overlays.render(ui.ctx());
        self.menus.commands(ui.ctx());
        self.notifications.render(ui.ctx());
    }

    fn header(&mut self, ui: &mut Ui) {
        let previous = self.theme();
        egui::Panel::top("showcase-header")
            .frame(frame(previous, 16))
            .show(ui, |ui| {
                Flex::row("brand-heading").wrap(true).show(ui, |ui| {
                    ui.add(Text::heading("D. DUTCHY / EGUI").size(TextSize::Xl2));
                    ui.add_space(space::LG);
                    ui.add(
                        Text::label(concat!("Component library · ", env!("CARGO_PKG_VERSION")))
                            .color(TextColor::Muted),
                    );
                });
                ui.add_space(space::SM);
                Flex::row("theme-controls").wrap(true).show(ui, |ui| {
                    for (accent, label) in [
                        (Accent::Orange, "Orange"),
                        (Accent::Purple, "Purple"),
                        (Accent::Crimson, "Crimson"),
                        (Accent::Blue, "Blue"),
                        (Accent::Green, "Green"),
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
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    foundation(ui);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.add(Text::label("The system").color(TextColor::Primary));
                        ui.add_space(space::LG);
                        egui::ScrollArea::vertical()
                            .id_salt("navigation-scroll")
                            .show(ui, |ui| {
                                let focus_inset = border::FOCUS_OFFSET + border::FOCUS_WIDTH;
                                ui.add_space(focus_inset);
                                self.navigation(ui, true);
                                ui.add_space(focus_inset);
                            });
                    });
                });
            });
    }

    fn navigation(&mut self, ui: &mut Ui, full_width: bool) {
        for page in Page::all() {
            if full_width && page == Page::COMPONENTS[0] {
                ui.add_space(space::MD);
            }
            let variant = if page == self.page {
                ButtonVariant::Secondary
            } else {
                ButtonVariant::Ghost
            };
            let response = ui.add(
                Button::new(page.name())
                    .variant(variant)
                    .size(ControlSize::Small)
                    .full_width(full_width),
            );
            let outline = response
                .rect
                .expand(border::FOCUS_OFFSET + border::FOCUS_WIDTH);
            if full_width && response.gained_focus() && !ui.clip_rect().contains_rect(outline) {
                ui.scroll_to_rect_animation(outline, None, egui::style::ScrollAnimation::none());
            }
            if response.clicked() {
                self.page = page;
            }
        }
    }
}

/// The sidebar footer, pinned under the scrolling page list. Laid out bottom-up:
/// tagline, label, then a divider the list scrolls beneath.
fn foundation(ui: &mut Ui) {
    ui.add(Text::new("Zero radius.\nFlat color.\nBold typography.").color(TextColor::Muted));
    ui.add(Text::label("01 / Foundation"));
    ui.add_space(space::LG);
    let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), border::THIN));
    ui.painter()
        .rect_filled(rect, 0, Theme::from_context(ui.ctx()).palette.border);
    ui.add_space(space::LG);
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
