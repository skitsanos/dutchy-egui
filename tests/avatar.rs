//! Avatar initials, sizes, presence, image states and semantics.
mod common;
#[path = "common/media.rs"]
mod media;
#[cfg(feature = "bundled-fonts")]
#[path = "avatar_support/rendering.rs"]
mod rendering;

use dutchy_egui::{Avatar, AvatarSize, Presence, Theme};
use egui::{Rect, Shape, TextureId, accesskit::Role, epaint::ClippedShape, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use media::Outcome;
use std::sync::{Arc, Mutex};

fn install(ctx: &egui::Context) {
    common::install(ctx, Theme::default().with_reduced_motion(true));
}
fn harness(app: impl FnMut(&mut egui::Ui) + 'static) -> Harness<'static> {
    let mut h = Harness::builder()
        .with_size([400.0, 200.0])
        .with_context_setup(install)
        .build_ui(app);
    h.run_steps(2);
    h
}
fn painted_text(h: &Harness<'_>) -> Vec<String> {
    let output = h.output();
    common::galleys(&output.shapes)
        .iter()
        .map(|g| g.text().to_owned())
        .collect()
}
/// Texture coordinates spanned by each textured mesh.
fn textured(shapes: &[ClippedShape]) -> Vec<Rect> {
    fn collect(shape: &Shape, out: &mut Vec<Rect>) {
        match shape {
            Shape::Mesh(mesh) if mesh.texture_id != TextureId::default() => {
                out.push(Rect::from_points(
                    &mesh.vertices.iter().map(|v| v.uv).collect::<Vec<_>>(),
                ));
            }
            Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for clipped in shapes {
        collect(&clipped.shape, &mut out);
    }
    out
}

#[test]
fn initials_use_the_first_and_last_words_and_skip_symbols() {
    for (name, expected) in [
        ("Ada Lovelace", "AL"),
        ("Grace Brewster Murray Hopper", "GH"),
        ("ada", "A"),
        ("  (Ada)  Lovelace ", "AL"),
        ("émile zola", "ÉZ"),
        ("ada@example.com", "A"),
    ] {
        let h = harness(move |ui| {
            ui.add(Avatar::new(name));
        });
        assert_eq!(painted_text(&h), [expected], "{name:?}");
        h.get_by_role_and_label(Role::Image, name);
    }
    let h = harness(|ui| {
        ui.add(Avatar::new("Ada Lovelace").initials("Ad"));
    });
    assert_eq!(painted_text(&h), ["Ad"]);
}
#[test]
fn empty_names_paint_no_initials_and_keep_a_generic_name() {
    let h = harness(|ui| {
        ui.add(Avatar::new("  "));
    });
    assert!(painted_text(&h).is_empty());
    h.get_by_role_and_label(Role::Image, "Avatar");
    let h = harness(|ui| {
        ui.add(Avatar::new("").accessible_name("Unknown sender"));
    });
    h.get_by_role_and_label(Role::Image, "Unknown sender");
}
#[test]
fn sizes_are_square_token_sides() {
    let rects = Arc::new(Mutex::new(Vec::new()));
    let seen = rects.clone();
    let _h = harness(move |ui| {
        let mut seen = seen.lock().unwrap();
        seen.clear();
        for size in AvatarSize::ALL {
            seen.push(ui.add(Avatar::new("Ada Lovelace").size(*size)).rect);
        }
    });
    let sides: Vec<f32> = rects.lock().unwrap().iter().map(|r| r.width()).collect();
    assert_eq!(sides, [24.0, 32.0, 48.0, 64.0]);
    assert!(
        rects
            .lock()
            .unwrap()
            .iter()
            .all(|r| r.size() == vec2(r.width(), r.width()))
    );
}
#[test]
fn presence_is_described_and_marked_inside_the_bottom_right_corner() {
    let rect = Arc::new(Mutex::new(Rect::NOTHING));
    let seen = rect.clone();
    let h = harness(move |ui| {
        *seen.lock().unwrap() = ui
            .add(
                Avatar::new("Ada Lovelace")
                    .size(AvatarSize::Large)
                    .presence(Presence::Online),
            )
            .rect;
    });
    let node = h.get_by_role_and_label(Role::Image, "Ada Lovelace");
    assert_eq!(
        node.accesskit_node().description().as_deref(),
        Some("Online")
    );
    let avatar = *rect.lock().unwrap();
    let success = Theme::default().palette.success;
    let output = h.output();
    let marker = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Rect(r) if r.fill == success => Some(r.rect),
            _ => None,
        })
        .expect("a success-colored presence marker");
    assert!(avatar.contains_rect(marker));
    assert_eq!(marker.size(), vec2(12.0, 12.0));
    assert_eq!(marker.right_bottom(), avatar.right_bottom());
}
#[test]
fn images_replace_initials_when_ready_and_fall_back_otherwise() {
    for (outcome, initials, busy) in [
        (Outcome::Ready, false, false),
        (Outcome::Pending, true, true),
        (Outcome::Failed, true, false),
    ] {
        let probe = Arc::new(Mutex::new(media::Probe {
            outcome,
            ..Default::default()
        }));
        let mut h = Harness::builder()
            .with_size([400.0, 200.0])
            .with_context_setup(move |ctx| {
                install(ctx);
                media::install(ctx, probe.clone());
            })
            .build_ui(|ui| {
                ui.add(
                    Avatar::new("Ada Lovelace")
                        .size(AvatarSize::XLarge)
                        .image("fixture://face.png"),
                );
            });
        h.run_steps(2);
        assert_eq!(!painted_text(&h).is_empty(), initials);
        let node = h.get_by_role_and_label(Role::Image, "Ada Lovelace");
        assert_eq!(node.accesskit_node().is_busy(), busy);
        let output = h.output();
        let uvs = textured(&output.shapes);
        if outcome == Outcome::Ready {
            // The 2:1 fixture is cropped to its centered square.
            assert_eq!(uvs.len(), 1);
            assert!((uvs[0].left() - 0.25).abs() < 1e-4 && (uvs[0].right() - 0.75).abs() < 1e-4);
        } else {
            assert!(uvs.is_empty());
        }
    }
}
#[test]
fn avatars_are_passive_and_leave_clicks_to_their_container() {
    let clicks = Arc::new(Mutex::new(0));
    let seen = clicks.clone();
    let mut h = harness(move |ui| {
        let row = ui.scope_builder(egui::UiBuilder::new().sense(egui::Sense::click()), |ui| {
            ui.add(Avatar::new("Ada Lovelace"));
        });
        if row.response.clicked() {
            *seen.lock().unwrap() += 1;
        }
    });
    let center = h
        .get_by_role_and_label(Role::Image, "Ada Lovelace")
        .rect()
        .center();
    h.hover_at(center);
    h.run_steps(1);
    h.event(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(3);
    assert_eq!(*clicks.lock().unwrap(), 1);
    h.check_accessibility();
}
