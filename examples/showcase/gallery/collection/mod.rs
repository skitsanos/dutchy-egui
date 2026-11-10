mod search;
use super::media::{art, pending};
use dutchy_egui::{
    Button, ButtonVariant, Checkbox, ColumnWidth, ControlSize, EmptyState, GalleryItem, Lightbox,
    MediaCard, OverlayOptions, ResponsiveGrid, SearchInput, Select, SelectOption, Spacing, Text,
    TextColor, TextSize, tokens::space,
};
use egui::{Id, Key, Modifiers, TextureHandle, Ui, load::SizedTexture, vec2};

pub struct CollectionState {
    textures: Vec<TextureHandle>,
    pub query: String,
    pub saved: [bool; 6],
    pub selected: Option<u8>,
    sort: Option<u8>,
    category: u8,
    only_saved: bool,
    opener: Option<Id>,
    changes: usize,
    opens: usize,
    saves: usize,
    samples: [String; 3],
    disabled: bool,
    read_only: bool,
}
impl Default for CollectionState {
    fn default() -> Self {
        Self {
            textures: Vec::new(),
            query: String::new(),
            saved: [false; 6],
            selected: None,
            sort: Some(0),
            category: 0,
            only_saved: false,
            opener: None,
            changes: 0,
            opens: 0,
            saves: 0,
            samples: Default::default(),
            disabled: false,
            read_only: false,
        }
    }
}
const DESCRIPTIONS: [&str; 6] = [
    "Canal-side geometry, warm orange and a quiet architectural rhythm.",
    "A bold vertical landmark with a rich purple palette.",
    "Flat color and strong negative space in an intimate courtyard.",
    "Engineered structure in a wide architectural composition.",
    "Repeated windows make everyday architecture into graphic form.",
    "An evening city silhouette, reduced to simple geometric blocks.",
];
fn category(index: usize) -> &'static str {
    if [2, 5].contains(&index) {
        "Graphic"
    } else {
        "Architecture"
    }
}

impl CollectionState {
    fn ensure(&mut self, ui: &Ui) {
        if self.textures.is_empty() {
            self.textures = art::textures(ui.ctx());
            ui.ctx()
                .add_texture_loader(std::sync::Arc::new(pending::Pending));
        }
    }
    pub fn cards(&mut self, ui: &mut Ui) {
        self.ensure(ui);
        ui.add(Text::label("Native additions / Artwork library").color(TextColor::Primary));
        ui.add(Text::heading("Find your next perspective.").size(TextSize::Xl5));
        ui.add(Text::new("A searchable collection of original Dutchy artwork. Preview and save are independent application actions.").color(TextColor::Muted));
        ui.add_space(space::LG);
        let search_id = self.toolbar(ui);
        ui.horizontal_wrapped(|ui| {
            for (index, label) in ["All artworks", "Architecture", "Graphic"]
                .into_iter()
                .enumerate()
            {
                if ui
                    .add(Button::new(label).size(ControlSize::Small).variant(
                        if self.category == index as u8 {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Outline
                        },
                    ))
                    .clicked()
                {
                    self.category = index as u8;
                }
            }
            ui.add(Checkbox::new(
                "saved-only",
                &mut self.only_saved,
                "Saved only",
            ));
        });
        ui.add_space(space::LG);
        let indices = self.filtered();
        ui.add(
            Text::label(format!(
                "{} {} / {} saved",
                indices.len(),
                if indices.len() == 1 {
                    "artwork"
                } else {
                    "artworks"
                },
                self.saved.iter().filter(|saved| **saved).count()
            ))
            .color(TextColor::Muted),
        );
        ui.add_space(space::MD);
        if indices.is_empty() {
            EmptyState::new("empty-artwork", "No matching artwork")
                .description("Change your search or filters to explore the collection.")
                .show(ui);
            if ui
                .add(Button::new("Reset artwork filters").variant(ButtonVariant::Outline))
                .clicked()
            {
                self.query.clear();
                self.category = 0;
                self.only_saved = false;
                ui.memory_mut(|m| m.request_focus(search_id));
            }
        }
        ResponsiveGrid::new("artwork-cards")
            .columns(3)
            .min_column_width(ColumnWidth::Compact)
            .gap(Spacing::Lg)
            .show(
                ui,
                &indices,
                |index| *index,
                |ui, index| {
                    let index = *index;
                    let response = MediaCard::new(
                        "card",
                        SizedTexture::from_handle(&self.textures[index]),
                        format!("{} original artwork", art::NAMES[index]),
                        art::NAMES[index],
                    )
                    .category(category(index))
                    .description(DESCRIPTIONS[index])
                    .action_label("View artwork")
                    .saved(self.saved[index])
                    .show(ui);
                    if response.open_requested {
                        self.selected = Some(index as u8);
                        self.opens += 1;
                        self.opener = Some(if response.preview.clicked() {
                            response.preview.id
                        } else {
                            response.details.id
                        });
                    }
                    if let Some(saved) = response.save_requested {
                        self.saved[index] = saved;
                        self.saves += 1;
                        if self.only_saved && !saved {
                            ui.memory_mut(|m| m.request_focus(search_id));
                        }
                    }
                },
            );
        ui.add_space(space::LG);
        ui.add(
            Text::new(format!(
                "Host observed {} query edits, {} previews and {} saved changes.",
                self.changes, self.opens, self.saves
            ))
            .color(TextColor::Muted),
        );
        let items: Vec<_> = self
            .textures
            .iter()
            .enumerate()
            .map(|(i, texture)| {
                GalleryItem::new(i as u8, SizedTexture::from_handle(texture), art::NAMES[i])
                    .caption(DESCRIPTIONS[i])
            })
            .collect();
        let options = OverlayOptions::default().return_focus(self.opener);
        Lightbox::new("artwork-preview", &items, &mut self.selected)
            .accessible_name("Artwork preview")
            .options(options)
            .show(ui.ctx());
    }
    fn toolbar(&mut self, ui: &mut Ui) -> Id {
        let available = ui.available_width();
        let sort_width = ColumnWidth::Compact.points().min(available);
        let wide = available >= 2.0 * sort_width + space::LG;
        let search_width = if wide {
            available - sort_width - space::LG
        } else {
            available
        };
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(space::LG, space::LG);
            let response = ui.add(
                SearchInput::new("artwork-search", &mut self.query)
                    .accessible_name("Search artworks")
                    .placeholder("Search artwork, categories or techniques…")
                    .key_hint(if cfg!(target_os = "macos") {
                        "⌘K"
                    } else {
                        "Ctrl+K"
                    })
                    .width(search_width),
            );
            self.changes += usize::from(response.changed());
            if ui.memory(|m| m.allows_interaction(response.layer_id))
                && ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::K))
            {
                response.request_focus();
            }
            let options = [
                SelectOption::new(0, "Curated for you"),
                SelectOption::new(1, "Name: A–Z"),
                SelectOption::new(2, "Saved first"),
            ];
            ui.allocate_ui_with_layout(
                vec2(sort_width, ControlSize::Medium.height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add(
                        Select::new("sort-artwork", &mut self.sort, &options)
                            .accessible_name("Sort artworks")
                            .placeholder("Curated for you"),
                    );
                },
            );
            response.id
        })
        .inner
    }
    fn filtered(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        let mut indices: Vec<_> = (0..self.textures.len())
            .filter(|index| {
                (!self.only_saved || self.saved[*index])
                    && (self.category == 0
                        || category(*index)
                            == if self.category == 1 {
                                "Architecture"
                            } else {
                                "Graphic"
                            })
                    && format!(
                        "{} {} {}",
                        art::NAMES[*index],
                        category(*index),
                        DESCRIPTIONS[*index]
                    )
                    .to_lowercase()
                    .contains(&query)
            })
            .collect();
        match self.sort.unwrap_or(0) {
            1 => indices.sort_by_key(|index| art::NAMES[*index]),
            2 => indices.sort_by_key(|index| !self.saved[*index]),
            _ => {}
        }
        indices
    }
    pub fn search(&mut self, ui: &mut Ui) {
        search::show(ui, self);
    }
}
