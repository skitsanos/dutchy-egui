//! Original flat-color raster illustrations; no downloads or image codec dependency.
use dutchy_egui::{Accent, Mode, tokens::Palette};
use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions};

pub const NAMES: [&str; 6] = [
    "Canal houses",
    "Purple tower",
    "Crimson courtyard",
    "Concrete bridge",
    "Window rhythm",
    "Evening skyline",
];
pub fn textures(ctx: &Context) -> Vec<TextureHandle> {
    [
        (384, 240),
        (240, 384),
        (320, 320),
        (480, 240),
        (256, 384),
        (400, 250),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (width, height))| {
        ctx.load_texture(
            format!("Dutchy original artwork {index}"),
            art(index, [width, height]),
            TextureOptions::LINEAR,
        )
    })
    .collect()
}
fn art(index: usize, size: [usize; 2]) -> ColorImage {
    let accent = [Accent::Orange, Accent::Purple, Accent::Crimson][index % 3];
    let p = Palette::new(accent, Mode::Light);
    let mut image = ColorImage::filled(size, p.background);
    block(&mut image, [0, 780, 1000, 220], p.foreground);
    block(&mut image, [0, 700, 1000, 40], p.primary);
    match index {
        0 => houses(&mut image, p),
        1 => tower(&mut image, p),
        2 => courtyard(&mut image, p),
        3 => bridge(&mut image, p),
        4 => windows(&mut image, p),
        _ => skyline(&mut image, p),
    }
    image
}
fn block(image: &mut ColorImage, rect: [usize; 4], color: Color32) {
    let [x, y, w, h] = rect;
    let [width, height] = image.size;
    for row in y * height / 1000..((y + h) * height / 1000).min(height) {
        for column in x * width / 1000..((x + w) * width / 1000).min(width) {
            image.pixels[row * width + column] = color;
        }
    }
}
fn houses(image: &mut ColorImage, p: Palette) {
    for (index, x) in [70, 360, 650].into_iter().enumerate() {
        block(
            image,
            [x, 250 + index * 60, 230, 530 - index * 60],
            if index == 1 { p.foreground } else { p.primary },
        );
        block(image, [x + 55, 150 + index * 60, 120, 120], p.foreground);
        for y in [350, 500, 650] {
            for dx in [40, 140] {
                block(image, [x + dx, y, 50, 60], p.background);
            }
        }
    }
}
fn tower(image: &mut ColorImage, p: Palette) {
    block(image, [200, 120, 550, 660], p.primary);
    block(image, [600, 120, 150, 660], p.foreground);
    for y in [180, 300, 420, 540, 660] {
        block(image, [270, y, 250, 55], p.background);
    }
    block(image, [130, 760, 700, 30], p.foreground);
}
fn courtyard(image: &mut ColorImage, p: Palette) {
    block(image, [100, 180, 800, 600], p.foreground);
    block(image, [200, 280, 600, 500], p.primary);
    block(image, [320, 400, 360, 380], p.background);
    block(image, [445, 520, 110, 260], p.foreground);
}
fn bridge(image: &mut ColorImage, p: Palette) {
    block(image, [50, 380, 900, 130], p.foreground);
    for x in [130, 450, 770] {
        block(image, [x, 510, 100, 270], p.primary);
    }
    for x in [100, 260, 420, 580, 740, 900] {
        block(image, [x, 280, 20, 100], p.foreground);
    }
    block(image, [50, 260, 900, 25], p.primary);
}
fn windows(image: &mut ColorImage, p: Palette) {
    block(image, [100, 100, 800, 680], p.primary);
    for x in [170, 400, 630] {
        for y in [180, 370, 560] {
            block(image, [x, y, 150, 120], p.foreground);
            block(image, [x + 15, y + 15, 45, 90], p.background);
        }
    }
}
fn skyline(image: &mut ColorImage, p: Palette) {
    for (x, y, w) in [
        (40, 430, 140),
        (200, 200, 180),
        (410, 340, 120),
        (560, 100, 140),
        (730, 280, 220),
    ] {
        block(image, [x, y, w, 780 - y], p.foreground);
        block(image, [x + 20, y + 30, 35, 50], p.primary);
        block(image, [x + 20, y + 130, 35, 50], p.background);
    }
    block(image, [0, 820, 1000, 45], p.primary);
}
