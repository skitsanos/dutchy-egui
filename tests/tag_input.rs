//! Keyboard, accessibility and rendering contracts for the composed editor.
mod common;

use dutchy_egui::{Accent, Mode, TagInput, TagInputRequest, TagItem, Theme, tokens::space};
use egui::{Event, ImeEvent, Key, Modifiers, Ui, accesskit::Role, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const NAME: &str = "Topics";
const ADD: &str = "Add tag to Topics";
const REMOVE: &str = "Remove Typography";

struct State {
    items: Vec<TagItem<'static, u32>>,
    draft: String,
    requests: Vec<TagInputRequest<u32>>,
    enabled: bool,
    parent_enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            items: vec![
                TagItem::new(1, "Architecture"),
                TagItem::new(2, "Typography"),
                TagItem::new(3, "Design system").removable(false),
            ],
            draft: String::new(),
            requests: Vec::new(),
            enabled: true,
            parent_enabled: true,
        }
    }
}

fn show(ui: &mut Ui, state: &mut State) {
    let _ = ui.button("Before tags");
    ui.add_enabled_ui(state.parent_enabled, |ui| {
        let result = TagInput::new("topics", &mut state.draft, &state.items)
            .accessible_name(NAME)
            .enabled(state.enabled)
            .show(ui);
        state.requests.extend(result.requests);
    });
    let _ = ui.button("After tags");
}

fn harness(theme: Theme, scale: f32, width: f32) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([width, 350.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(show, State::default());
    h.run_steps(3);
    h
}

fn editor(h: &mut Harness<'_, State>) {
    h.get_by_role_and_label(Role::TextInput, NAME).focus();
    h.run_steps(2);
}

fn key_event(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn enter_and_button_submit_the_trimmed_draft_without_changing_host_tags() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    let group = h.get_by_role_and_label(Role::Group, NAME);
    group.get_by_role_and_label(Role::TextInput, NAME);
    assert!(group.get_by_label(ADD).accesskit_node().is_disabled());
    assert!(group.query_by_label("Remove Design system").is_none());
    editor(&mut h);
    h.event(Event::Text("   ".into()));
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert!(h.state().requests.is_empty());

    h.input_mut().events = vec![
        Event::Paste("  Grid system  ".into()),
        key_event(Key::Enter, true),
        key_event(Key::Enter, false),
    ];
    h.run_steps(2);
    assert_eq!(
        h.state().requests,
        [TagInputRequest::Add("Grid system".into())]
    );
    assert_eq!(h.state().draft, "     Grid system  ");
    assert!(h.query_by_label("Grid system").is_none());
    assert!(h.get_by_role_and_label(Role::TextInput, NAME).is_focused());
    h.get_by_label(ADD).click();
    h.run_steps(2);
    assert_eq!(
        h.state().requests,
        [(); 2].map(|()| TagInputRequest::Add("Grid system".into()))
    );
    assert!(h.get_by_role_and_label(Role::TextInput, NAME).is_focused());
    h.check_accessibility();
}

#[test]
fn backspace_only_removes_a_removable_tag_from_an_already_empty_draft() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    editor(&mut h);
    h.event(Event::Text("x".into()));
    h.run();
    h.key_down(Key::Backspace);
    h.run_steps(2);
    assert!(h.state().draft.is_empty());
    assert!(h.state().requests.is_empty());
    h.key_down(Key::Backspace);
    h.key_up(Key::Backspace);
    h.run_steps(2);
    h.key_press_modifiers(Modifiers::SHIFT, Key::Backspace);
    h.run_steps(2);
    assert!(h.state().requests.is_empty());
    h.key_press(Key::Backspace);
    h.run_steps(2);
    assert_eq!(h.state().requests, [TagInputRequest::Remove(2)]);
    h.get_by_label(REMOVE);
}

#[test]
fn remove_buttons_use_tab_order_and_independent_activation() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    h.state_mut().draft = "Kept draft".into();
    h.get_by_label("Before tags").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("Remove Architecture").is_focused());
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label(REMOVE).is_focused());
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert!(h.get_by_role_and_label(Role::TextInput, NAME).is_focused());
    h.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    h.run();
    assert!(h.get_by_label(REMOVE).is_focused());
    h.key_press(Key::Space);
    h.run_steps(2);
    h.get_by_label("Remove Architecture").click_accesskit();
    h.run_steps(2);
    h.get_by_label(REMOVE).click();
    h.run_steps(2);
    assert_eq!(h.state().draft, "Kept draft");
    assert_eq!(
        h.state().requests,
        [2, 2, 1, 2].map(TagInputRequest::Remove)
    );
}

#[test]
fn disabling_a_focused_tag_input_blocks_queued_entry_and_removal() {
    for (enabled, parent_enabled) in [(false, true), (true, false)] {
        let mut h = harness(Theme::default(), 1.0, 520.0);
        editor(&mut h);
        h.state_mut().enabled = enabled;
        h.state_mut().parent_enabled = parent_enabled;
        h.input_mut().events = vec![
            Event::Paste("Blocked".into()),
            key_event(Key::Enter, true),
            key_event(Key::Backspace, true),
        ];
        h.run_steps(2);
        let remove = h.get_by_label(REMOVE);
        assert!(remove.accesskit_node().is_disabled());
        remove.click_accesskit();
        h.run_steps(2);
        assert!(h.state().draft.is_empty());
        assert!(h.state().requests.is_empty());
    }
}

#[test]
fn enter_used_to_confirm_ime_text_does_not_also_submit() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    editor(&mut h);
    h.event(Event::Ime(ImeEvent::Preedit {
        text: "e".into(),
        active_range_chars: None,
    }));
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert!(h.state().requests.is_empty());
    h.input_mut().events = vec![
        Event::Ime(ImeEvent::Commit("é".into())),
        key_event(Key::Enter, true),
        key_event(Key::Enter, false),
    ];
    h.run_steps(2);
    assert!(h.state().requests.is_empty());
    assert_eq!(h.state().draft, "é");
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().requests, [TagInputRequest::Add("é".into())]);
}

#[test]
fn keyed_remove_focus_survives_rename_reorder_and_wrap() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    let remove = h.get_by_label(REMOVE);
    let id = remove.accesskit_node().id();
    remove.focus();
    h.run_steps(2);
    h.state_mut().items[1] = TagItem::new(2, "Type");
    h.state_mut().items.swap(1, 2);
    h.set_size(vec2(155.0, 350.0));
    h.run_steps(3);
    let renamed = h.get_by_label("Remove Type");
    assert_eq!(renamed.accesskit_node().id(), id);
    assert!(renamed.is_focused());
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().requests, [TagInputRequest::Remove(2)]);
}

#[test]
fn chips_wrap_and_entry_fits_at_two_themes_and_scales() {
    for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
        let theme = Theme::new(Accent::Orange, mode);
        let mut h = harness(theme, scale, 520.0);
        let draft_rect =
            |h: &Harness<'_, State>| h.get_by_role_and_label(Role::TextInput, NAME).rect();
        assert_eq!(draft_rect(&h).top(), h.get_by_label(ADD).rect().top());
        h.set_size(vec2(155.0, 350.0));
        h.run_steps(3);
        let tags =
            ["Architecture", "Typography", "Design system"].map(|name| h.get_by_label(name).rect());
        assert!(
            tags.windows(2)
                .all(|pair| pair[1].top() - pair[0].bottom() == space::SM)
        );
        let draft = draft_rect(&h);
        let add = h.get_by_label(ADD).rect();
        assert!(draft.top() >= tags[2].bottom() + space::SM);
        assert!(add.top() > draft.bottom());
        let after = h.get_by_label("After tags").rect();
        assert!(after.top() > add.bottom());
        assert!(
            tags.into_iter()
                .chain([draft, add])
                .all(|rect| rect.right() <= 155.0)
        );
        let point = (tags[0].min + vec2(5.0, 5.0)) * scale;
        assert_eq!(
            h.render()
                .unwrap()
                .get_pixel(point.x as u32, point.y as u32)
                .0,
            theme.palette.background.to_array()
        );
    }
}
