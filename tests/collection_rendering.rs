//! Picture-card composition, loading states and searchable showcase behavior.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{Accent, ColumnWidth, MediaCard, Mode, ResponsiveGrid, SearchInput, Text, Theme};
use egui::{ImageSource, TextureHandle, accesskit::Role, load::SizedTexture};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
use gallery::media::{art, pending};

#[derive(Default)]
struct State {
    textures: Vec<TextureHandle>,
    query: String,
    rects: Vec<egui::Rect>,
}
fn matrix(ui: &mut egui::Ui, state: &mut State) {
    let p = Theme::from_context(ui.ctx()).palette;
    ui.painter().rect_filled(ui.max_rect(), 0, p.background);
    if state.textures.is_empty() {
        state.textures = art::textures(ui.ctx());
        ui.ctx()
            .add_texture_loader(std::sync::Arc::new(pending::Pending));
    }
    ui.add(Text::heading("Picture. Search. Discover."));
    ui.add(
        SearchInput::new("search", &mut state.query)
            .accessible_name("Search artwork")
            .placeholder("Find your next perspective…")
            .key_hint("Ctrl+K"),
    );
    state.rects.clear();
    ResponsiveGrid::new("cards")
        .columns(3)
        .min_column_width(ColumnWidth::Compact)
        .show(
            ui,
            &[0usize, 1, 2],
            |i| *i,
            |ui, index| {
                let (source, name) = match index {
                    0 => (
                        ImageSource::from(SizedTexture::from_handle(&state.textures[0])),
                        art::NAMES[0],
                    ),
                    1 => (ImageSource::from("showcase://pending"), "Loading artwork"),
                    _ => (ImageSource::from("broken://artwork"), "Unavailable artwork"),
                };
                let response = MediaCard::new(index, source, name, name)
                    .category("Illustration")
                    .description("Sharp geometry, warm color and a clear visual hierarchy.")
                    .action_label("View artwork")
                    .saved(*index == 0)
                    .retryable(true)
                    .show(ui);
                state.rects.push(response.response.rect);
            },
        );
}

#[test]
fn media_and_search_render_across_themes_states_and_display_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "collection-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "collection-dark-1_5x"),
        (Accent::Blue, Mode::Light, 2.0, "collection-blue-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([1040.0, 680.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui_state(
                matrix,
                State {
                    query: "Canal".into(),
                    ..Default::default()
                },
            );
        h.run();
        h.get_by_label("View artwork: Canal houses").focus();
        h.run();
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn long_card_copy_wraps_within_a_narrow_column() {
    let mut h = Harness::builder()
        .with_size([200.0, 900.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            MediaCard::new(
                "long",
                "broken://long",
                "A narrow artwork preview",
                "An unusually long title that still leaves the direction arrow visible",
            )
            .description(
                "Supporting copy that wraps over several rows and remains within its column. "
                    .repeat(10),
            )
            .saved(false)
            .retryable(true)
            .show(ui);
        });
    h.run();
    h.check_accessibility();
    for galley in common::galleys(&h.output().shapes) {
        assert!(
            galley.size().x <= 184.5,
            "{}: {:?}",
            galley.job.text,
            galley.size()
        );
    }
    // The title reserves the direction arrow's column: no row may run under it.
    let details = h
        .get_by_label("Open An unusually long title that still leaves the direction arrow visible")
        .rect();
    let arrow_left = details.right()
        - dutchy_egui::tokens::media_card::PADDING
        - dutchy_egui::tokens::media_card::ICON;
    let title = h
        .output()
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) if text.galley.job.text.starts_with("AN UNUSUALLY") => {
                Some(text.pos.x + text.galley.rect.right())
            }
            _ => None,
        })
        .expect("title galley");
    assert!(
        title <= arrow_left + 0.5,
        "title right edge {title} overlaps the arrow starting at {arrow_left}"
    );
    std::fs::create_dir_all("target/collection-proof").unwrap();
    h.render()
        .unwrap()
        .save("target/collection-proof/narrow-card.png")
        .unwrap();
}

fn showcase(page: gallery::Page, size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    let mut h = Harness::builder().with_size(size).build_eframe(move |cc| {
        common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
        gallery::Showcase {
            page,
            reduced_motion: true,
            ..Default::default()
        }
    });
    h.run();
    h
}

#[test]
fn artwork_browser_filters_clears_saves_and_opens_host_owned_previews() {
    let mut h = showcase(gallery::Page::MediaCards, [1200.0, 1600.0]);
    h.get_by_role_and_label(Role::SearchInput, "Search artworks")
        .focus();
    h.run();
    h.get_by_role_and_label(Role::SearchInput, "Search artworks")
        .type_text("canal");
    h.run();
    assert_eq!(h.state().collection.query, "canal");
    assert!(h.query_by_label("Save Purple tower").is_none());
    h.get_by_label("Save Canal houses").click();
    h.run();
    assert!(h.state().collection.saved[0]);
    assert_eq!(h.state().collection.selected, None);
    h.get_by_label("Open Canal houses").click();
    h.run_steps(3);
    assert_eq!(h.state().collection.selected, Some(0));
    h.get_by_role_and_label(Role::Dialog, "Artwork preview");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_eq!(h.state().collection.selected, None);
    assert!(h.get_by_label("Open Canal houses").is_focused());
    h.get_by_label("Clear Search artworks").click();
    h.run();
    assert!(h.state().collection.query.is_empty());
    h.get_by_label("Save Purple tower");
    h.get_by_label("Saved only").click();
    h.run();
    h.get_by_label("Unsave Canal houses").click();
    h.run();
    assert!(!h.state().collection.saved[0]);
    h.get_by_role_and_label(Role::Heading, "No matching artwork");
    assert!(
        h.get_by_role_and_label(Role::SearchInput, "Search artworks")
            .is_focused()
    );
}

#[test]
fn component_pages_render_desktop_and_narrow_with_valid_accessibility() {
    for (page, label) in [
        (gallery::Page::MediaCards, "cards"),
        (gallery::Page::SearchInput, "search"),
    ] {
        for (width, height, suffix) in [(1200.0, 1500.0, "desktop"), (480.0, 2200.0, "narrow")] {
            let mut h = showcase(page, [width, height]);
            h.check_accessibility();
            std::fs::create_dir_all("target/collection-proof").unwrap();
            h.render()
                .unwrap()
                .save(format!("target/collection-proof/{label}-{suffix}.png"))
                .unwrap();
        }
    }
}
