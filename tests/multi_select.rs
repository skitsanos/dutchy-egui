//! Host ownership, native search interaction and wrapping selection chips.
mod common;

use dutchy_egui::{Accent, Mode, MultiSelect, SearchState, SelectOption, Theme, tokens::space};
use egui::{Event, ImeEvent, Key, Modifiers, Ui, accesskit::Role, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

const NAME: &str = "Media types";
const RESULTS: &str = "Media types options";

struct State {
    options: Vec<SelectOption<'static, u8>>,
    selected: Vec<u8>,
    search: SearchState,
    requests: Vec<(u8, bool)>,
    enabled: bool,
    parent_enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            options: vec![
                SelectOption::new(1, "Photography").id_salt(1),
                SelectOption::new(2, "Video").id_salt(2).enabled(false),
                SelectOption::new(3, "Prints").id_salt(3),
                SelectOption::new(4, "Drawings").id_salt(4),
            ],
            selected: vec![1, 99],
            search: SearchState::default(),
            requests: Vec::new(),
            enabled: true,
            parent_enabled: true,
        }
    }
}

fn show(ui: &mut Ui, state: &mut State) {
    let _ = ui.button("Before choices");
    ui.add_enabled_ui(state.parent_enabled, |ui| {
        let result = MultiSelect::new("media", &state.selected, &mut state.search, &state.options)
            .accessible_name(NAME)
            .enabled(state.enabled)
            .show(ui);
        state.requests.extend(
            result
                .requests
                .into_iter()
                .map(|request| (request.value, request.selected)),
        );
    });
    let _ = ui.button("After choices");
}

fn harness(theme: Theme, scale: f32, width: f32) -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([width, 600.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(show, State::default());
    h.run_steps(3);
    h
}

fn focus_query(h: &mut Harness<'_, State>) {
    h.get_by_role_and_label(Role::EditableComboBox, NAME)
        .focus();
    h.run_steps(2);
}

fn is_selected(h: &Harness<'_, State>, option: &str) -> Option<bool> {
    h.get_by_role_and_label(Role::ListBoxOption, option)
        .accesskit_node()
        .is_selected()
}

#[test]
fn pointer_requests_keep_the_popup_open_and_leave_selection_to_the_host() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    let group = h.get_by_role_and_label(Role::Group, NAME);
    group.get_by_label("Remove Photography");
    group
        .get_by_role_and_label(Role::EditableComboBox, NAME)
        .click();
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::ListBox, RESULTS)
            .accesskit_node()
            .is_multiselectable()
    );
    assert_eq!(is_selected(&h, "Photography"), Some(true));
    assert_eq!(is_selected(&h, "Prints"), Some(false));
    h.get_by_role_and_label(Role::ListBoxOption, "Video")
        .click_accesskit();
    h.run_steps(2);
    assert!(h.state().requests.is_empty());
    h.get_by_role_and_label(Role::ListBoxOption, "Prints")
        .click();
    h.run_steps(3);
    assert!(h.state().search.open);
    assert_eq!(h.state().requests, [(3, true)]);
    assert_eq!(h.state().selected, [1, 99]);
    assert_eq!(is_selected(&h, "Prints"), Some(false));
    assert!(h.query_by_label("Remove Prints").is_none());
}

#[test]
fn assistive_activation_and_chip_removal_use_the_host_selection() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    h.state_mut().selected.extend([2, 3]);
    focus_query(&mut h);
    h.key_press(Key::ArrowDown);
    h.run_steps(3);
    assert_eq!(is_selected(&h, "Prints"), Some(true));
    assert_eq!(is_selected(&h, "Video"), Some(true));
    assert!(h.query_by_label("Remove Video").is_none());
    h.get_by_role_and_label(Role::ListBoxOption, "Prints")
        .click_accesskit();
    h.run_steps(3);
    assert_eq!(h.state().requests, [(3, false)]);
    assert!(h.state().search.open);
    h.get_by_label("Remove Photography").click();
    h.run_steps(3);
    assert_eq!(h.state().requests, [(3, false), (1, false)]);
    assert!(!h.state().search.open);
    assert!(
        h.get_by_role_and_label(Role::EditableComboBox, NAME)
            .is_focused()
    );
    h.check_accessibility();
}

#[test]
fn keyboard_search_preserves_query_and_focus_and_skips_disabled_options() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    focus_query(&mut h);
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(h.state().search.open);
    assert!(h.state().requests.is_empty());
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    let input = h.get_by_role_and_label(Role::EditableComboBox, NAME);
    let active = input.accesskit_node().active_descendant().unwrap().id();
    assert_eq!(
        active,
        h.get_by_role_and_label(Role::ListBoxOption, "Prints")
            .accesskit_node()
            .id()
    );
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().requests, [(3, true)]);

    h.event(Event::Text("phtg".into()));
    h.run_steps(3);
    h.get_by_role_and_label(Role::ListBoxOption, "Photography");
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Prints")
            .is_none()
    );
    h.key_down(Key::Enter);
    h.run_steps(2);
    h.key_down(Key::Enter);
    h.key_up(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().requests, [(3, true), (1, false)]);
    assert!(h.state().search.open);
    assert_eq!(h.state().search.query, "phtg");
}

#[test]
fn escape_tab_and_chip_removal_preserve_native_focus() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    focus_query(&mut h);
    h.key_press(Key::ArrowDown);
    h.run_steps(3);
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.state().search.open);
    assert!(
        h.get_by_role_and_label(Role::EditableComboBox, NAME)
            .is_focused()
    );
    h.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Remove Photography").is_focused());
    h.key_press(Key::Space);
    h.run_steps(3);
    assert_eq!(h.state().requests, [(1, false)]);
    assert!(
        h.get_by_role_and_label(Role::EditableComboBox, NAME)
            .is_focused()
    );
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(!h.state().search.open);
    assert!(h.get_by_label("After choices").is_focused());
}

#[test]
fn no_results_and_ime_confirmation_leave_the_selection_alone() {
    let mut h = harness(Theme::default(), 1.0, 520.0);
    focus_query(&mut h);
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    h.input_mut().events = vec![Event::Ime(ImeEvent::Preedit {
        text: "phtg".into(),
        active_range_chars: None,
    })];
    h.run_steps(2);
    h.input_mut().events = vec![
        Event::Ime(ImeEvent::Commit("phtg".into())),
        Event::Key {
            key: Key::Enter,
            physical_key: Some(Key::Enter),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        },
    ];
    h.run_steps(2);
    h.key_up(Key::Enter);
    assert!(h.state().requests.is_empty());
    assert_eq!(h.state().search.query, "phtg");
    h.state_mut().search.query = "xyzxyz".into();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Status, "No matches");
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert!(h.state().requests.is_empty());
    assert_eq!(h.state().selected, [1, 99]);
    h.get_by_label("Remove Photography");
    h.get_by_label("Before choices").click();
    h.run_steps(2);
    assert!(!h.state().search.open);
}

#[test]
fn becoming_disabled_blocks_queued_edits_and_chip_actions() {
    for (enabled, parent_enabled) in [(false, true), (true, false)] {
        let mut h = harness(Theme::default(), 1.0, 520.0);
        focus_query(&mut h);
        h.key_press(Key::ArrowDown);
        h.run_steps(3);
        h.state_mut().enabled = enabled;
        h.state_mut().parent_enabled = parent_enabled;
        h.event(Event::Paste("Blocked".into()));
        h.key_press(Key::Enter);
        h.run_steps(3);
        let chip = h.get_by_label("Remove Photography");
        assert!(chip.accesskit_node().is_disabled());
        chip.click_accesskit();
        h.run_steps(2);
        assert!(h.state().search.query.is_empty());
        assert!(!h.state().search.open);
        assert!(h.state().requests.is_empty());
    }
}

#[test]
fn chips_keep_focus_through_reorder_and_wrap_across_themes() {
    for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
        let theme = Theme::new(Accent::Orange, mode);
        let mut h = harness(theme, scale, 520.0);
        h.state_mut().selected.extend([3, 4]);
        h.run_steps(3);
        let photo = h.get_by_label("Remove Photography");
        let id = photo.accesskit_node().id();
        photo.focus();
        h.run_steps(2);
        h.state_mut().options.rotate_left(1);
        h.set_size(vec2(150.0, 600.0));
        h.run_steps(3);
        let first = h.get_by_role_and_label(Role::Label, "Prints").rect();
        let last = h.get_by_role_and_label(Role::Label, "Photography").rect();
        let query = h.get_by_role_and_label(Role::EditableComboBox, NAME).rect();
        assert!(last.top() >= first.bottom() + space::SM);
        assert!(query.top() >= last.bottom() + space::SM);
        assert!(query.right() <= 150.0);
        let remove = h.get_by_label("Remove Photography");
        assert_eq!(remove.accesskit_node().id(), id);
        assert!(remove.is_focused());
        let sample = (last.min + vec2(5.0, 5.0)) * scale;
        assert_eq!(
            h.render()
                .unwrap()
                .get_pixel(sample.x as u32, sample.y as u32)
                .0,
            theme.palette.background.to_array()
        );
        h.key_press(Key::Enter);
        h.run_steps(2);
        assert_eq!(h.state().requests, [(1, false)]);
    }
}
