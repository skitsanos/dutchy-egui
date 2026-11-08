//! Native editing and independent search adornments through the public API.
mod common;
use dutchy_egui::{ControlSize, Field, SearchInput, Theme};
use egui::{Key, Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    query: String,
    enabled: bool,
    parent: bool,
    read_only: bool,
    prefix: bool,
    size: ControlSize,
    width: f32,
    changes: usize,
    escapes: usize,
    rect: Rect,
}
fn harness() -> Harness<'static, State> {
    let mut h = Harness::builder()
        .with_size([520.0, 300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.prefix {
                    ui.label("Inserted sibling");
                }
                ui.add_enabled_ui(state.parent, |ui| {
                    let response = Field::new("Search styles")
                        .id_salt("field")
                        .required(true)
                        .show(ui, |ui| {
                            ui.add(
                                SearchInput::new("styles", &mut state.query)
                                    .accessible_name("Search styles")
                                    .placeholder("Styles, moods or techniques…")
                                    .key_hint("Ctrl+K")
                                    .size(state.size)
                                    .width(state.width)
                                    .char_limit(4)
                                    .enabled(state.enabled)
                                    .read_only(state.read_only),
                            )
                        });
                    state.changes += usize::from(response.changed());
                    state.rect = response.rect;
                });
                state.escapes += usize::from(ui.input(|i| i.key_pressed(Key::Escape)));
            },
            State {
                query: String::new(),
                enabled: true,
                parent: true,
                read_only: false,
                prefix: false,
                size: ControlSize::Medium,
                width: 460.0,
                changes: 0,
                escapes: 0,
                rect: Rect::NOTHING,
            },
        );
    h.run();
    h
}

#[test]
fn editing_and_pointer_clear_are_host_edits_and_clear_restores_focus() {
    let mut h = harness();
    h.ctx
        .set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
    assert!(h.query_by_label("Clear Search styles").is_none());
    h.get_by_role_and_label(Role::SearchInput, "Search styles")
        .click();
    h.run();
    assert!(
        h.get_by_role_and_label(Role::SearchInput, "Search styles")
            .is_focused()
    );
    h.event(egui::Event::Paste("é🦊abc".into()));
    h.run();
    assert_eq!(h.state().query, "é🦊ab");
    let before = h.state().changes;
    h.get_by_label("Clear Search styles").click();
    h.run();
    assert_eq!(h.state().query, "");
    assert_eq!(h.state().changes, before + 1);
    assert!(
        h.get_by_role_and_label(Role::SearchInput, "Search styles")
            .is_focused()
    );
    h.run();
    assert!(h.query_by_label("Clear Search styles").is_none());
    assert_eq!(h.state().changes, before + 1);
    h.check_accessibility();
}

#[test]
fn keyboard_clear_works_and_escape_remains_available_to_the_host() {
    let mut h = harness();
    h.state_mut().query = "ink".into();
    h.run();
    h.get_by_label("Clear Search styles").focus();
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().query, "");
    assert!(
        h.get_by_role_and_label(Role::SearchInput, "Search styles")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(h.state().escapes, 1);
}

#[test]
fn read_only_and_disabled_fields_preserve_host_values_and_reject_clear() {
    let mut h = harness();
    h.state_mut().query = "Host value longer than the typing limit".into();
    h.state_mut().read_only = true;
    h.run();
    assert!(
        h.get_by_label("Search styles")
            .accesskit_node()
            .is_read_only()
    );
    h.get_by_label("Search styles").focus();
    h.run();
    h.get_by_label("Search styles").type_text("x");
    h.run();
    h.get_by_label("Clear Search styles").click();
    h.run();
    assert_eq!(h.state().query, "Host value longer than the typing limit");
    assert_eq!(h.state().changes, 0);
    for parent in [true, false] {
        h.state_mut().read_only = false;
        h.state_mut().enabled = parent;
        h.state_mut().parent = !parent;
        h.run();
        assert!(
            h.get_by_label("Search styles")
                .accesskit_node()
                .is_disabled()
        );
        assert!(
            h.get_by_label("Clear Search styles")
                .accesskit_node()
                .is_disabled()
        );
        h.get_by_label("Clear Search styles").click();
        h.run();
        assert_eq!(h.state().changes, 0);
    }
}

#[test]
fn field_label_and_stable_ids_preserve_focus_across_sibling_and_host_changes() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Label, "Search styles")
        .click();
    h.run();
    assert!(
        h.get_by_role_and_label(Role::SearchInput, "Search styles")
            .is_focused()
    );
    let id = h.ctx.memory(|m| m.focused());
    h.state_mut().prefix = true;
    h.state_mut().query = "blue".into();
    h.run();
    assert_eq!(h.ctx.memory(|m| m.focused()), id);
    assert_eq!(
        h.get_by_role_and_label(Role::SearchInput, "Search styles")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("blue")
    );
    assert_eq!(h.state().changes, 0);
}

#[test]
fn narrow_adornments_keep_clear_inside_each_control_size() {
    let mut h = harness();
    h.state_mut().query = "ink".into();
    for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
        for width in [120.0, 240.0, 460.0] {
            h.state_mut().size = size;
            h.state_mut().width = width;
            h.run();
            let outer = h.state().rect;
            let clear = h.get_by_label("Clear Search styles").rect();
            assert_eq!(outer.height(), size.height());
            assert!(outer.contains_rect(clear), "{outer:?}, {clear:?}");
            assert_eq!(
                h.query_by_role_and_label(Role::Label, "Ctrl+K").is_some(),
                width > 240.0
            );
        }
    }
}
