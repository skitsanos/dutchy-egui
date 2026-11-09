use super::{config, harness};
use crate::{app::App, provider::Mode as ProviderMode};
use dutchy_egui::{
    ControlSize, Mode,
    tokens::{border, space},
};
use egui::{Pos2, Shape};
use egui_kittest::kittest::Queryable;
type Harness = egui_kittest::Harness<'static, App>;
const TITLES: [&str; 3] = [
    "Short",
    "A conversation title longer than the sidebar can display",
    "Current",
];

#[test]
fn shell_aligns_actions_and_conversation_labels_across_widths_themes_and_scales() {
    for mode in [Mode::Light, Mode::Dark] {
        for scale in [1.0, 1.5, 2.0] {
            for width in [480.0, 560.0, 1160.0] {
                check_layout(width, mode, scale);
            }
        }
    }
}

fn check_layout(width: f32, mode: Mode, scale: f32) {
    let mut app = App::new(config());
    for (i, title) in TITLES.iter().enumerate() {
        if i > 0 {
            app.chat.new_conversation().unwrap();
        }
        app.chat.current_mut().title = (*title).into();
    }
    if mode == Mode::Light {
        app.config.provider = ProviderMode::OpenAi;
    }
    let mut h = harness(app, width, mode, scale);
    check_header(&h, width);
    if width >= 960.0 {
        check_sidebar(&h);
    }
    check_actions(&mut h);
    h.check_accessibility();
}

fn check_header(h: &Harness, width: f32) {
    let new_chat = h.get_by_label("Start new conversation").rect();
    let settings = h
        .get_by_role_and_label(egui::accesskit::Role::Button, "Chat settings")
        .rect();
    assert!((settings.right() - (width - space::LG)).abs() < 1.0);
    assert!((new_chat.center().y - settings.center().y).abs() < 1.0);
    assert!(new_chat.right() + space::SM <= settings.left() + 1.0);
    let brand = text_origin(&h.output().shapes, "DUTCHYAGENT");
    assert!((brand.x - space::LG).abs() < 1.0);
    assert!(brand.y < new_chat.bottom());
}

fn check_sidebar(h: &Harness) {
    for title in TITLES {
        let row = h
            .get_by_label(&format!("Open conversation: {title}"))
            .rect();
        let origin = text_origin(&h.output().shapes, &title.to_uppercase());
        assert!((origin.x - row.left() - ControlSize::Small.padding()).abs() < 1.0);
        assert!(
            (row.width() - (264.0 - 2.0 * space::LG)).abs() <= border::STANDARD + 1.0,
            "{title}: {row:?}"
        );
    }
}

fn check_actions(h: &mut Harness) {
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Chat settings")
        .click();
    h.run_steps(4);
    assert!(h.state().settings_open);
    h.key_press(egui::Key::Escape);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Chat settings")
            .is_focused()
    );
    h.get_by_label("Start new conversation").focus();
    h.step();
    h.key_press(egui::Key::Enter);
    h.step();
    assert_eq!(h.state().chat.conversations.len(), 4);
}

fn text_origin(shapes: &[egui::epaint::ClippedShape], label: &str) -> Pos2 {
    fn find(shape: &Shape, label: &str) -> Option<Pos2> {
        match shape {
            Shape::Text(text) if text.galley.job.text == label => Some(text.pos),
            Shape::Vec(children) => children.iter().find_map(|shape| find(shape, label)),
            _ => None,
        }
    }
    shapes
        .iter()
        .find_map(|s| find(&s.shape, label))
        .expect("label painted")
}
