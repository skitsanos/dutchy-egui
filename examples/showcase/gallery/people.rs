use dutchy_egui::{
    Accent, Avatar, AvatarSize, AvatarVariant, List, ListItem, ListState, Presence, Text,
    TextColor, TextSize, tokens::space,
};
use egui::{Color32, ColorImage, TextureHandle, TextureOptions, Ui, load::SizedTexture};

/// Fictional people for the directory.
const PEOPLE: [(&str, &str, Presence); 6] = [
    ("Mara Velde", "Design lead", Presence::Online),
    ("Joost de Wit", "Platform", Presence::Away),
    ("Ines Okafor", "Research", Presence::Busy),
    ("Theo Brandt", "Support", Presence::Offline),
    ("Lotte Maas", "Editorial", Presence::Online),
    ("Ravi Anand", "Infrastructure", Presence::Away),
];

pub struct PeopleState {
    directory: ListState<usize>,
    rows: Vec<ListItem<'static, usize>>,
    portrait: Option<TextureHandle>,
}

impl Default for PeopleState {
    fn default() -> Self {
        let mut directory = ListState::default();
        directory.selected.insert(0);
        Self {
            directory,
            rows: PEOPLE
                .iter()
                .enumerate()
                .map(|(key, (name, ..))| ListItem::new(key, *name))
                .collect(),
            portrait: None,
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut PeopleState) {
    ui.add(Text::heading("Put a face to every name.").size(TextSize::Xl5));
    ui.add(
        Text::new("Square initials, an image once it loads, and presence the host reports.")
            .color(TextColor::Muted),
    );
    super::section(ui, "01", "Sizes", |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::LG;
            for size in AvatarSize::ALL {
                ui.vertical(|ui| {
                    ui.add(Avatar::new("Mara Velde").size(*size));
                    ui.add(Text::label(size.name()).color(TextColor::Muted));
                });
            }
        });
    });
    super::section(ui, "02", "Variants and presence", |ui| {
        for variant in AvatarVariant::ALL {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::MD;
                for (name, _, presence) in &PEOPLE[..4] {
                    ui.add(
                        Avatar::new(*name)
                            .size(AvatarSize::Large)
                            .variant(*variant)
                            .presence(*presence),
                    );
                }
                ui.add(Text::label(variant.name()).color(TextColor::Muted));
            });
            ui.add_space(space::SM);
        }
    });
    super::section(ui, "03", "Directory", |ui| directory(ui, state));
    super::section(ui, "04", "Image source", |ui| portrait(ui, state));
}

fn directory(ui: &mut Ui, state: &mut PeopleState) {
    List::new("people", &mut state.directory, &state.rows)
        .accessible_name("People")
        .row_height(56.0)
        .max_height(280.0)
        .show_with(ui, |ui, item, _| {
            let (name, role, presence) = PEOPLE[item.key];
            ui.spacing_mut().item_spacing.x = space::MD;
            ui.add(Avatar::new(name).presence(presence));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.add(Text::new(name).size(TextSize::Sm));
                ui.add(
                    Text::new(format!("{role} · {}", presence.name()))
                        .size(TextSize::Xs)
                        .color(TextColor::Muted),
                );
            });
        });
    ui.add_space(space::SM);
    ui.add(
        Text::new("Rows stay clickable: the avatar is passive and leaves clicks to the row.")
            .size(TextSize::Sm)
            .color(TextColor::Muted),
    );
}

fn portrait(ui: &mut Ui, state: &mut PeopleState) {
    let texture = state.portrait.get_or_insert_with(|| {
        ui.ctx()
            .load_texture("portrait", tiles(), TextureOptions::NEAREST)
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::LG;
        ui.add(
            Avatar::new("Mara Velde")
                .size(AvatarSize::XLarge)
                .image(SizedTexture::from_handle(texture))
                .presence(Presence::Online),
        );
        ui.add(
            Avatar::new("Joost de Wit")
                .size(AvatarSize::XLarge)
                .image("bytes://missing-portrait.png"),
        );
        ui.add(
            Text::new("A ready texture fills the square; a missing image keeps the initials.")
                .size(TextSize::Sm)
                .color(TextColor::Muted),
        );
    });
}

/// A generated 2:1 tile pattern, cropped to its centered square by the avatar.
fn tiles() -> ColorImage {
    let (width, height) = (16, 8);
    let colors: [Color32; 3] = [Accent::Blue, Accent::Orange, Accent::Green].map(Accent::color);
    let pixels = (0..width * height)
        .map(|index| {
            let (x, y) = (index % width, index / width);
            colors[(x / 2 + y / 2) % colors.len()]
        })
        .collect();
    ColorImage::new([width, height], pixels)
}
