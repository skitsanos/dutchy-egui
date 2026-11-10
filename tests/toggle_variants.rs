//! Every supported action treatment must expose pressed state visually.
mod common;
use dutchy_egui::{
    Accent, ButtonVariant, ControlSize, Icon, Mode, Theme, ToggleButton, Toolbar, ToolbarItem,
};
use egui::{
    Id, Rect,
    accesskit::{Role, Toggled},
    pos2,
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct State {
    on: bool,
    enabled: bool,
    rect: Rect,
    id: Id,
}

#[test]
fn all_variants_distinguish_on_and_off_in_buttons_and_toolbars() {
    for (accent, mode, scale) in configurations() {
        for &variant in ButtonVariant::ALL {
            for toolbar in [false, true] {
                check_states(
                    Theme::new(accent, mode).with_reduced_motion(true),
                    scale,
                    variant,
                    toolbar,
                );
            }
        }
    }
}

fn configurations() -> [(Accent, Mode, f32); 3] {
    [
        (Accent::Orange, Mode::Light, 1.0),
        (Accent::Orange, Mode::Dark, 1.5),
        (Accent::Blue, Mode::Light, 2.0),
    ]
}

fn check_states(theme: Theme, scale: f32, variant: ButtonVariant, toolbar: bool) {
    let mut h = Harness::builder()
        .with_size([460.0, 240.0])
        .with_theme(if theme.mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .with_pixels_per_point(scale)
        .with_context_setup(move |ctx| common::install(ctx, theme))
        .build_ui_state(
            move |ui, s: &mut State| {
                let response = if toolbar {
                    let items = [ToolbarItem::toggle(0u8, "Preview", s.on)
                        .icon(Icon::Play)
                        .icon_only(false)
                        .variant(variant)
                        .tooltip("")];
                    Toolbar::new("tools", &items)
                        .size(ControlSize::Medium)
                        .enabled(s.enabled)
                        .show(ui)
                        .items
                        .remove(0)
                        .1
                } else {
                    ui.add(
                        ToggleButton::new("preview", &mut s.on, "Preview")
                            .icon(Icon::Play)
                            .variant(variant)
                            .enabled(s.enabled)
                            .tooltip(""),
                    )
                };
                s.rect = response.rect;
                s.id = response.id;
            },
            State {
                on: false,
                enabled: true,
                rect: Rect::NOTHING,
                id: Id::NULL,
            },
        );
    h.run();
    let id = h.state().id;
    for enabled in [true, false] {
        h.state_mut().enabled = enabled;
        for hovered in [false, true] {
            h.hover_at(if hovered {
                h.state().rect.center()
            } else {
                pos2(450.0, 230.0)
            });
            h.state_mut().on = false;
            h.run();
            let off = h.output().shapes.clone();
            h.state_mut().on = true;
            h.run();
            let node = h.get_by_role_and_label(Role::Button, "Preview");
            assert_eq!(node.accesskit_node().data().toggled(), Some(Toggled::True));
            assert_eq!(h.state().id, id);
            assert_ne!(
                off,
                h.output().shapes,
                "{variant:?}, toolbar={toolbar}, enabled={enabled}, hover={hovered}, {:?}@{scale}",
                theme.mode
            );
        }
    }
}

#[cfg(feature = "bundled-fonts")]
fn matrix(ui: &mut egui::Ui) {
    use dutchy_egui::{Text, TextColor};
    let p = Theme::from_context(ui.ctx()).palette;
    ui.painter().rect_filled(ui.max_rect(), 0, p.background);
    ui.add(Text::heading("State you can see."));
    ui.add(Text::new("Off / On · Standalone and toolbar toggles").color(TextColor::Muted));
    for &variant in ButtonVariant::ALL {
        ui.push_id(variant.name(), |ui| {
            ui.add(Text::label(variant.name()));
            ui.horizontal(|ui| {
                let mut off = false;
                let mut on = true;
                ui.add(
                    ToggleButton::new("off", &mut off, "Preview")
                        .icon(Icon::Play)
                        .variant(variant),
                );
                ui.add(
                    ToggleButton::new("on", &mut on, "Preview")
                        .icon(Icon::Play)
                        .variant(variant),
                );
                let items = [
                    ToolbarItem::toggle(0u8, "Preview", false)
                        .icon(Icon::Play)
                        .variant(variant),
                    ToolbarItem::toggle(1, "Preview", true)
                        .icon(Icon::Play)
                        .variant(variant),
                ];
                Toolbar::new("toolbar", &items).width(100.0).show(ui);
            });
        });
    }
}

#[cfg(feature = "bundled-fonts")]
#[test]
fn variant_states_render_across_themes_and_display_scales() {
    let mut results = egui_kittest::SnapshotResults::new();
    for ((accent, mode, scale), name) in configurations().into_iter().zip([
        "toggle-variants-light-1x",
        "toggle-variants-dark-1_5x",
        "toggle-variants-blue-2x",
    ]) {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([720.0, 650.0])
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(matrix);
        h.run();
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
