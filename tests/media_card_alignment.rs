//! A card owns its title/arrow allocation independently of host alignment.
mod common;
use dutchy_egui::{Accent, MediaCard, Mode, Theme, tokens::media_card as tokens};
use egui::{Align, Layout, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

const LONG: &str = "An unusually long title that still leaves the direction arrow visible";

#[test]
fn title_keeps_the_token_gap_in_all_host_alignments_and_display_scales() {
    for (accent, mode, scale) in [
        (Accent::Orange, Mode::Light, 1.0),
        (Accent::Orange, Mode::Dark, 1.5),
        (Accent::Blue, Mode::Light, 2.0),
    ] {
        for align in [Align::Min, Align::Center, Align::Max] {
            for justified in [false, true] {
                for width in [160.0, 200.0, 360.0] {
                    check_title(accent, mode, scale, align, justified, width);
                }
            }
        }
    }
}

fn check_title(accent: Accent, mode: Mode, scale: f32, align: Align, justified: bool, width: f32) {
    let theme = Theme::new(accent, mode).with_reduced_motion(true);
    let mut h = Harness::builder()
        .with_size([460.0, 900.0])
        .with_theme(if mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_pixels_per_point(scale)
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui(move |ui| {
            ui.with_layout(
                Layout::top_down(align).with_cross_justify(justified),
                |ui| {
                    for (id, title) in ["Canal houses", LONG].into_iter().enumerate() {
                        MediaCard::new(id, "fixture://pending", "Artwork", title)
                            .width(width)
                            .show(ui);
                    }
                },
            );
        });
    h.run();
    for title in ["Canal houses", LONG] {
        let details = h
            .get_by_role_and_label(Role::Button, &format!("Open {title}"))
            .rect();
        let text = h
            .output()
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == title.to_uppercase() => Some(t),
                _ => None,
            })
            .expect("title galley");
        let title_right = text.pos.x + text.galley.rect.right();
        let reserved_left = details.right() - tokens::PADDING - tokens::ICON - tokens::GAP;
        assert!(
            title_right <= reserved_left + 0.5,
            "{align:?}/{justified}, width {width}, {mode:?}@{scale}: title ends {title_right}, reserved column starts {reserved_left}"
        );
        assert!(text.pos.x + text.galley.rect.left() >= details.left() + tokens::PADDING - 0.5);
        h.get_by_role_and_label(Role::Heading, title);
    }
}
