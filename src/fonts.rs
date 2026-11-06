//! Static typeface installation. Installation preserves existing host fonts.

use egui::{Context, FontDefinitions, FontFamily, FontId};

use crate::tokens::{Family, Weight};

/// Resolve a face, keeping distinct weights out of each other's fallback chain.
pub fn font_id(family: Family, weight: Weight, size: f32) -> FontId {
    FontId::new(size, font_family(family, weight))
}

/// Named family for a static face, or a host family without `bundled-fonts`.
pub fn font_family(family: Family, weight: Weight) -> FontFamily {
    #[cfg(feature = "bundled-fonts")]
    {
        FontFamily::Name(face_name(family, weight).into())
    }
    #[cfg(not(feature = "bundled-fonts"))]
    {
        let _ = weight;
        match family {
            Family::Mono => FontFamily::Monospace,
            _ => FontFamily::Proportional,
        }
    }
}

/// Install bundled faces once per Context, without replacing application fonts.
///
/// Without `bundled-fonts`, the host must supply proportional/monospace fonts.
pub fn install(ctx: &Context) {
    #[cfg(feature = "bundled-fonts")]
    {
        let id = egui::Id::unique("dutchy::fonts-installed");
        if ctx.data(|data| data.get_temp::<bool>(id).unwrap_or(false)) {
            return;
        }
        for &(family, weight, bytes) in FACES {
            ctx.add_font(egui::FontInsert::new(
                &face_name(family, weight),
                egui::FontData::from_static(bytes),
                insert_families(family, weight),
            ));
        }
        ctx.data_mut(|data| data.insert_temp(id, true));
    }
    #[cfg(not(feature = "bundled-fonts"))]
    let _ = ctx;
}

/// Add the bundled faces to caller-owned definitions, preserving fallback fonts.
///
/// Call before `Context::set_fonts`. This is a no-op without `bundled-fonts`.
pub fn extend(definitions: &mut FontDefinitions) {
    #[cfg(feature = "bundled-fonts")]
    for &(family, weight, bytes) in FACES {
        let name = face_name(family, weight);
        definitions.font_data.insert(
            name.clone(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
        for insertion in insert_families(family, weight) {
            let chain = definitions.families.entry(insertion.family).or_default();
            chain.retain(|existing| existing != &name);
            chain.insert(0, name.clone());
        }
    }
    #[cfg(not(feature = "bundled-fonts"))]
    let _ = definitions;
}

#[cfg(feature = "bundled-fonts")]
fn face_name(family: Family, weight: Weight) -> String {
    format!("dutchy::{family:?}::{weight:?}")
}

#[cfg(feature = "bundled-fonts")]
fn insert_families(family: Family, weight: Weight) -> Vec<egui::epaint::text::InsertFontFamily> {
    use egui::epaint::text::{FontPriority, InsertFontFamily};
    let mut families = vec![InsertFontFamily {
        family: font_family(family, weight),
        priority: FontPriority::Highest,
    }];
    let standard = match (family, weight) {
        (Family::Body, Weight::Regular) => Some(FontFamily::Proportional),
        (Family::Mono, Weight::Regular) => Some(FontFamily::Monospace),
        _ => None,
    };
    if let Some(family) = standard {
        families.push(InsertFontFamily {
            family,
            priority: FontPriority::Highest,
        });
    }
    families
}

#[cfg(feature = "bundled-fonts")]
type Face = (Family, Weight, &'static [u8]);

#[cfg(feature = "bundled-fonts")]
const FACES: &[Face] = &[
    (
        Family::Body,
        Weight::Regular,
        include_bytes!("../assets/fonts/Body-Regular.ttf"),
    ),
    (
        Family::Body,
        Weight::Medium,
        include_bytes!("../assets/fonts/Body-Medium.ttf"),
    ),
    (
        Family::Body,
        Weight::SemiBold,
        include_bytes!("../assets/fonts/Body-SemiBold.ttf"),
    ),
    (
        Family::Body,
        Weight::Bold,
        include_bytes!("../assets/fonts/Body-Bold.ttf"),
    ),
    (
        Family::Display,
        Weight::Regular,
        include_bytes!("../assets/fonts/Display-Regular.ttf"),
    ),
    (
        Family::Display,
        Weight::Medium,
        include_bytes!("../assets/fonts/Display-Medium.ttf"),
    ),
    (
        Family::Display,
        Weight::SemiBold,
        include_bytes!("../assets/fonts/Display-SemiBold.ttf"),
    ),
    (
        Family::Display,
        Weight::Bold,
        include_bytes!("../assets/fonts/Display-Bold.ttf"),
    ),
    (
        Family::Mono,
        Weight::Regular,
        include_bytes!("../assets/fonts/Mono-Regular.ttf"),
    ),
    (
        Family::Mono,
        Weight::Medium,
        include_bytes!("../assets/fonts/Mono-Medium.ttf"),
    ),
    (
        Family::Mono,
        Weight::SemiBold,
        include_bytes!("../assets/fonts/Mono-SemiBold.ttf"),
    ),
    (
        Family::Mono,
        Weight::Bold,
        include_bytes!("../assets/fonts/Mono-Bold.ttf"),
    ),
];
