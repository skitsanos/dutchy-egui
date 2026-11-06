use dutchy_egui::{Button, ButtonVariant, Checkbox, ControlSize, GalleryItem, Text, TextColor};

struct Photo {
    id: u16,
    alt: &'static str,
    author: &'static str,
    credit: &'static str,
}
const PHOTOS: [Photo; 6] = [
    Photo {
        id: 10,
        alt: "Forest overlooking a blue lake and distant mountains",
        author: "Paul Jarvis",
        credit: "https://unsplash.com/photos/6J--NXulQCs",
    },
    Photo {
        id: 28,
        alt: "Rocky stream through a green forest",
        author: "Jerry Adney",
        credit: "https://unsplash.com/photos/_WiFMBRT7Aw",
    },
    Photo {
        id: 237,
        alt: "Black puppy looking up from a wooden floor",
        author: "André Spieker",
        credit: "https://unsplash.com/photos/8wTPqxlnKM4",
    },
    Photo {
        id: 870,
        alt: "Illuminated lighthouse against a starry blue sky",
        author: "Joshua Hibbert",
        credit: "https://unsplash.com/photos/Pn6iimgM-wo",
    },
    Photo {
        id: 1025,
        alt: "Pug wrapped in a blanket on a woodland path",
        author: "Matthew Wiebe",
        credit: "https://unsplash.com/photos/U5rMrSI7Pn4",
    },
    Photo {
        id: 1043,
        alt: "River beneath forest and tall granite cliffs",
        author: "Christian Joudrey",
        credit: "https://unsplash.com/photos/mWRR1xj95hg",
    },
];

pub fn items() -> Vec<GalleryItem<'static, u8>> {
    PHOTOS
        .iter()
        .enumerate()
        .map(|(key, photo)| {
            let (width, height) = if key == 1 { (720, 960) } else { (960, 640) };
            GalleryItem::new(
                key as u8,
                format!("https://picsum.photos/id/{}/{width}/{height}.jpg", photo.id),
                photo.alt,
            )
            .caption(photo.alt)
            .caption_source(format!("{} / Picsum · Unsplash", photo.author))
            .enabled(key != 3)
        })
        .collect()
}

pub fn controls(ui: &mut egui::Ui, enabled: &mut bool) {
    ui.horizontal_wrapped(|ui| {
        ui.add(Checkbox::new("online-photos", enabled, "Use Picsum photos"));
        if *enabled
            && ui
                .add(
                    Button::new("Reload photos")
                        .variant(ButtonVariant::Outline)
                        .size(ControlSize::Small),
                )
                .clicked()
        {
            for item in items() {
                if let Some(uri) = item.source.uri() {
                    ui.ctx().forget_image(uri);
                }
            }
        }
    });
    if *enabled {
        ui.add(
            Text::new(
                "Online photos load on demand. If a request fails, use Retry or Reload photos.",
            )
            .color(TextColor::Muted),
        );
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to("Photos via Lorem Picsum", "https://picsum.photos/");
            for photo in &PHOTOS {
                ui.hyperlink_to(photo.author, photo.credit);
            }
        });
    }
}
