//! The optional icon font belongs to the example's egui context.

mod glyphs;

use crate::icons;
use dutchy_egui::Icon;
use egui::{
    Color32, Context, FontData, FontFamily, FontId, FontInsert, Painter, Rect,
    epaint::text::{FontPriority, InsertFontFamily},
};
use std::sync::OnceLock;

const NAME: &str = "dutchy-mail::phosphor-regular";
const BYTES: &[u8] = include_bytes!("../assets/Phosphor-Mail-Regular.ttf");

fn family() -> &'static FontFamily {
    static FAMILY: OnceLock<FontFamily> = OnceLock::new();
    FAMILY.get_or_init(|| FontFamily::Name(NAME.into()))
}

pub fn install(ctx: &Context) {
    ctx.add_font(FontInsert::new(
        NAME,
        FontData::from_static(BYTES),
        vec![InsertFontFamily {
            family: family().clone(),
            priority: FontPriority::Highest,
        }],
    ));
}

pub fn plus(painter: &Painter, bounds: Rect, color: Color32) {
    paint_or(painter, bounds, color, glyphs::PLUS, Icon::Plus);
}

pub fn archive(painter: &Painter, bounds: Rect, color: Color32) {
    paint_or(
        painter,
        bounds,
        color,
        glyphs::ARCHIVE,
        Icon::Custom(icons::archive),
    );
}

pub fn trash(painter: &Painter, bounds: Rect, color: Color32) {
    paint_or(painter, bounds, color, glyphs::TRASH, Icon::Trash);
}

pub fn envelope(painter: &Painter, bounds: Rect, color: Color32) {
    paint_or(
        painter,
        bounds,
        color,
        glyphs::ENVELOPE,
        Icon::Custom(icons::envelope),
    );
}

fn paint_or(painter: &Painter, bounds: Rect, color: Color32, glyph: char, fallback: Icon) {
    let galley = painter.fonts_mut(|fonts| {
        let family = family();
        if !fonts.definitions().families.contains_key(family) {
            return None;
        }
        let font = FontId::new(bounds.width().min(bounds.height()), family.clone());
        fonts
            .has_glyph(&font, glyph)
            .then(|| fonts.layout_no_wrap(glyph.to_string(), font, Color32::PLACEHOLDER))
    });
    if let Some(galley) = galley.filter(|galley| galley.mesh_bounds.is_positive()) {
        painter.galley(
            bounds.center() - galley.mesh_bounds.center().to_vec2(),
            galley,
            color,
        );
    } else {
        fallback.paint(painter, bounds, color);
    }
}
