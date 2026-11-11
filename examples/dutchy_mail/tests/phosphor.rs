use super::{harness_with_icons, newest, row_label};
use crate::{app::App, mailbox::INBOX};
use dutchy_egui::{Accent, Mode};
use egui::{FontFamily, Shape, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

const GLYPHS: [(&str, char); 4] = [
    ("Compose", '\u{e3d4}'),
    ("Archive", '\u{e00e}'),
    ("Delete", '\u{e4a6}'),
    ("Read / unread", '\u{e214}'),
];

fn select_message(h: &mut Harness<'_, App>) {
    let id = newest(h, INBOX);
    h.get_by_role_and_label(Role::ListBoxOption, &row_label(h, id))
        .click();
    h.run_steps(3);
}

#[test]
fn bundled_phosphor_glyphs_render_inside_named_controls_at_both_scales() {
    for (accent, mode, scale) in [
        (Accent::Orange, Mode::Light, 1.0),
        (Accent::Blue, Mode::Dark, 2.0),
    ] {
        let mut h = harness_with_icons(accent, mode, scale, true);
        select_message(&mut h);
        let toolbar = h.get_by_role_and_label(Role::Toolbar, "Message actions");
        let output = h.output();
        for (label, glyph) in GLYPHS {
            let rect = toolbar.get_by_role_and_label(Role::Button, label).rect();
            let text = glyph.to_string();
            let shape = output
                .shapes
                .iter()
                .find_map(|clipped| match &clipped.shape {
                    Shape::Text(shape) if shape.galley.text() == text => Some(shape),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{label} is missing its Phosphor glyph"));
            let ink = shape.visual_bounding_rect();
            assert!(ink.is_positive() && rect.contains_rect(ink));
        }
        h.check_accessibility();
        let image = h.render().unwrap();
        assert_eq!(image.width(), (1280.0 * scale) as u32);
    }
}

#[test]
fn removing_the_optional_family_restores_the_original_toolbar_pixels() {
    let mut fallback = harness_with_icons(Accent::Orange, Mode::Light, 1.0, false);
    select_message(&mut fallback);
    let expected = fallback.render().unwrap();

    let mut optional = harness_with_icons(Accent::Orange, Mode::Light, 1.0, true);
    select_message(&mut optional);
    let mut fonts = optional.ctx.fonts(|fonts| fonts.definitions().clone());
    let name = "dutchy-mail::phosphor-regular";
    assert!(
        fonts
            .families
            .remove(&FontFamily::Name(name.into()))
            .is_some()
    );
    assert!(fonts.font_data.remove(name).is_some());
    optional.ctx.set_fonts(fonts);
    optional.run_steps(3);
    let actual = optional.render().unwrap();
    let toolbar = optional
        .get_by_role_and_label(Role::Toolbar, "Message actions")
        .rect();
    for y in toolbar.top().ceil() as u32..toolbar.bottom().floor() as u32 {
        for x in toolbar.left().ceil() as u32..toolbar.right().floor() as u32 {
            assert_eq!(
                actual.get_pixel(x, y),
                expected.get_pixel(x, y),
                "pixel at {x}, {y}"
            );
        }
    }
}
