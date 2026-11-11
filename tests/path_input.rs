//! Native editing, typed browse intents, and responsive PathInput geometry.
mod common;

use dutchy_egui::{ControlSize, Field, Mode, PathBrowseRequest, PathInput, Theme};
use egui::{Key, Ui, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const NAME: &str = "Project path";
const BROWSE: &str = "Browse for Project path";

struct State {
    path: String,
    mode: PathBrowseRequest,
    requests: Vec<PathBrowseRequest>,
    width: f32,
    size: ControlSize,
    read_only: bool,
    enabled: bool,
    parent_enabled: bool,
    prefix: bool,
    changes: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            path: String::new(),
            mode: PathBrowseRequest::OpenFile,
            requests: Vec::new(),
            width: 340.0,
            size: ControlSize::Medium,
            read_only: false,
            enabled: true,
            parent_enabled: true,
            prefix: false,
            changes: 0,
        }
    }
}

fn show(ui: &mut Ui, state: &mut State) {
    if state.prefix {
        ui.label("Inserted sibling");
    }
    ui.add_enabled_ui(state.parent_enabled, |ui| {
        let editor = Field::new(NAME).show(ui, |ui| {
            let result = PathInput::new("project", &mut state.path, state.mode)
                .accessible_name(NAME)
                .placeholder("Enter a path")
                .size(state.size)
                .width(state.width)
                .enabled(state.enabled)
                .read_only(state.read_only)
                .show(ui);
            state.requests.extend(result.request);
            result.response
        });
        state.changes += usize::from(editor.changed());
    });
}

fn harness(state: State, theme: Theme, scale: f32) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([440.0, 240.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(show, state);
    h.run_steps(2);
    h
}

#[test]
fn editor_reports_edits_and_browse_emits_only_the_configured_request() {
    use PathBrowseRequest::{OpenFile, OpenFolder, SaveFile};

    let mut h = harness(State::default(), Theme::default(), 1.0);
    h.get_by_role_and_label(Role::Label, NAME).click();
    h.run();
    assert!(h.get_by_role_and_label(Role::TextInput, NAME).is_focused());
    h.event(egui::Event::Paste("../Project files/canal.txt".into()));
    h.run();
    assert_eq!(h.state().path, "../Project files/canal.txt");
    assert_eq!(h.state().changes, 1);
    assert!(h.state().requests.is_empty());

    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_role_and_label(Role::Button, BROWSE).is_focused());
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(h.state().requests, [OpenFile]);

    h.state_mut().mode = OpenFolder;
    h.run();
    h.get_by_role_and_label(Role::Button, BROWSE).click();
    h.run_steps(2);
    assert_eq!(h.state().requests, [OpenFile, OpenFolder]);

    h.state_mut().mode = SaveFile;
    h.run();
    h.get_by_role_and_label(Role::Button, BROWSE)
        .click_accesskit();
    h.run_steps(2);
    assert_eq!(h.state().requests, [OpenFile, OpenFolder, SaveFile]);
    assert_eq!(h.state().path, "../Project files/canal.txt");
    assert_eq!(h.state().changes, 1);
    h.check_accessibility();
}

#[test]
fn read_only_and_disabled_states_block_browsing() {
    let mut h = harness(
        State {
            path: "host path".into(),
            read_only: true,
            ..Default::default()
        },
        Theme::default(),
        1.0,
    );
    let editor = h.get_by_role_and_label(Role::TextInput, NAME);
    assert!(editor.accesskit_node().is_read_only());
    editor.click();
    h.run();
    assert!(h.get_by_role_and_label(Role::TextInput, NAME).is_focused());
    h.event(egui::Event::Paste("more text".into()));
    h.run();
    for (read_only, enabled, parent_enabled) in [
        (true, true, true),
        (false, false, true),
        (false, true, false),
    ] {
        let state = h.state_mut();
        state.read_only = read_only;
        state.enabled = enabled;
        state.parent_enabled = parent_enabled;
        h.run();
        let browse = h.get_by_role_and_label(Role::Button, BROWSE);
        assert!(browse.accesskit_node().is_disabled());
        assert_eq!(
            h.get_by_role_and_label(Role::TextInput, NAME)
                .accesskit_node()
                .is_disabled(),
            !read_only
        );
        browse.click();
        browse.click_accesskit();
        h.run();
        assert!(h.state().requests.is_empty());
    }
    assert_eq!(h.state().path, "host path");
    assert_eq!(h.state().changes, 0);
}

#[test]
fn ids_and_focus_survive_switching_to_stacked_layout() {
    let mut h = harness(State::default(), Theme::default(), 1.0);
    let editor = h.get_by_role_and_label(Role::TextInput, NAME);
    let editor_id = editor.accesskit_node().id();
    let browse = h.get_by_role_and_label(Role::Button, BROWSE);
    let browse_id = browse.accesskit_node().id();
    assert!(editor.rect().right() < browse.rect().left());
    browse.focus();
    h.run();
    assert!(h.get_by_role_and_label(Role::Button, BROWSE).is_focused());

    h.state_mut().width = 90.0;
    h.state_mut().prefix = true;
    h.run_steps(2);
    let editor = h.get_by_role_and_label(Role::TextInput, NAME);
    let browse = h.get_by_role_and_label(Role::Button, BROWSE);
    assert_eq!(editor.accesskit_node().id(), editor_id);
    assert_eq!(browse.accesskit_node().id(), browse_id);
    assert!(browse.is_focused());
    assert!(editor.rect().bottom() < browse.rect().top());
    assert_eq!(editor.rect().left(), browse.rect().left());
    assert_eq!(editor.rect().right(), browse.rect().right());
}

#[test]
fn browse_stacks_before_the_editor_gets_narrower_than_the_button() {
    let mut h = harness(State::default(), Theme::default(), 1.0);
    for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
        let mut beside = false;
        for width in (100..=420).step_by(10) {
            h.state_mut().size = size;
            h.state_mut().width = width as f32;
            h.run_steps(2);
            let editor = h.get_by_role_and_label(Role::TextInput, NAME).rect();
            let browse = h.get_by_role_and_label(Role::Button, BROWSE).rect();
            if editor.bottom() > browse.top() {
                beside = true;
                assert!(
                    editor.width() >= browse.width(),
                    "{size:?} at {width}: {editor:?} beside {browse:?}"
                );
            }
        }
        assert!(beside, "{size:?} never placed Browse beside the editor");
    }
}

#[test]
fn renderer_preserves_shared_heights_and_browse_hover_in_both_layouts() {
    for (mode, scale, width) in [(Mode::Light, 1.0, 340.0), (Mode::Dark, 2.0, 92.0)] {
        let theme = Theme::new(dutchy_egui::Accent::Orange, mode);
        let mut h = harness(
            State {
                width,
                ..Default::default()
            },
            theme,
            scale,
        );
        for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
            h.state_mut().size = size;
            h.run_steps(2);
            let editor = h.get_by_role_and_label(Role::TextInput, NAME).rect();
            let browse = h.get_by_role_and_label(Role::Button, BROWSE);
            let button = browse.rect();
            assert!((editor.height() - size.height()).abs() <= 1.0, "{editor:?}");
            assert!((button.height() - size.height()).abs() <= 1.0, "{button:?}");
            assert!(
                (editor.left() + width - button.right()).abs() <= 1.0,
                "{size:?} {mode:?} {editor:?} {button:?}"
            );
            assert!(!editor.intersects(button));
            browse.hover();
            h.run_steps(2);
            let pixels = h.render().unwrap();
            let x = ((button.left() + 5.0) * scale) as u32;
            let y = ((button.top() + 5.0) * scale) as u32;
            assert_eq!(
                pixels.get_pixel(x, y).0,
                theme.palette.foreground.to_array()
            );
        }
        h.check_accessibility();
    }
}
