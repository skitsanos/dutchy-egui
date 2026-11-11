//! Focused rendering baselines and showcase smoke checks.

#![cfg(feature = "bundled-fonts")]

mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::{
    Accent, Badge, BadgeVariant, Button, ButtonVariant, ControlSize, Mode, Text, TextColor,
    TextSize, Theme,
};
use egui::{Event, Key, accesskit::Role};
use egui_kittest::{
    Harness, SnapshotResults,
    kittest::{NodeT, Queryable},
};

fn state_matrix(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Sharp. Clear.").size(TextSize::Xl5));
    ui.add(Text::new("Inter / Body 16 / Warm neutrals / Zero radius"));
    ui.add_space(16.0);
    egui::Grid::new("button-states")
        .spacing([12.0, 12.0])
        .show(ui, |ui| {
            for &variant in ButtonVariant::ALL {
                ui.add(
                    Button::new(variant.name())
                        .variant(variant)
                        .size(ControlSize::Small),
                );
                ui.add(
                    Button::new("Disabled")
                        .variant(variant)
                        .size(ControlSize::Small)
                        .enabled(false),
                );
                ui.add(
                    Button::new("Working")
                        .variant(variant)
                        .size(ControlSize::Small)
                        .loading(true),
                );
                ui.end_row();
            }
        });
    ui.add_space(16.0);
    ui.horizontal_wrapped(|ui| {
        for &variant in BadgeVariant::ALL {
            ui.add(Badge::new(variant.name()).variant(variant));
        }
    });
    ui.add_space(16.0);
    ui.add(Text::label("Tab / Enter / Space").color(TextColor::Primary));
}

#[test]
fn states_render_across_accents_modes_and_display_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "orange-light-1x"),
        (Accent::Purple, Mode::Light, 1.5, "purple-light-1_5x"),
        (Accent::Crimson, Mode::Light, 2.0, "crimson-light-2x"),
        (Accent::Orange, Mode::Dark, 1.0, "orange-dark-1x"),
        (Accent::Blue, Mode::Light, 1.5, "blue-light-1_5x"),
        (Accent::Green, Mode::Dark, 2.0, "green-dark-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut harness = Harness::builder()
            .with_size([640.0, 600.0])
            .with_render_cursor(false)
            .with_theme(match mode {
                Mode::Light => egui::Theme::Light,
                Mode::Dark => egui::Theme::Dark,
            })
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(state_matrix);
        harness.run();
        harness.snapshot(name);
        results.extend_harness(&mut harness);
    }
    results.unwrap();
}

#[test]
fn keyboard_focus_and_hover_render() {
    let mut harness = Harness::builder()
        .with_size([640.0, 600.0])
        .with_render_cursor(false)
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(state_matrix);
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Primary")
        .focus();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Outline")
        .hover();
    harness.run();
    harness.snapshot("focus-and-hover");
}

#[test]
fn showcase_is_usable_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [
        ([1200.0, 940.0], "showcase-desktop"),
        ([480.0, 900.0], "showcase-narrow"),
    ] {
        let mut harness = Harness::builder()
            .with_size(size)
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| {
                common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                gallery::Showcase {
                    reduced_motion: true,
                    ..Default::default()
                }
            });
        harness.run();
        harness.check_accessibility();
        harness
            .render()
            .unwrap()
            .save(format!("target/visual-proof/{name}.png"))
            .unwrap();
        assert!(harness.get_by_label("D. DUTCHY / EGUI").rect().width() < size[0]);
        let role = if size[0] >= 820.0 {
            Role::ListBoxOption
        } else {
            Role::Button
        };
        harness.get_by_role_and_label(role, "Overview");
        assert!(
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Destructive")
                .rect()
                .right()
                <= size[0]
        );
    }
}

#[test]
fn showcase_navigation_and_theme_switches_update_the_host_state() {
    let mut harness = Harness::builder()
        .with_size([1200.0, 940.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                reduced_motion: true,
                ..Default::default()
            }
        });
    harness.get_by_label("Purple").click();
    harness.run();
    assert_eq!(harness.state().accent, Accent::Purple);
    harness.get_by_label("Blue").click();
    harness.run();
    assert_eq!(harness.state().accent, Accent::Blue);
    assert_eq!(
        harness.ctx.global_style().visuals.selection.bg_fill,
        Accent::Blue.color()
    );
    save_proof(&mut harness, "showcase-blue-light");
    harness.get_by_label("Green").click();
    harness.run();
    assert_eq!(harness.state().accent, Accent::Green);
    harness.get_by_label("Dark mode").click();
    harness.run();
    assert_eq!(harness.state().mode, Mode::Dark);
    assert!(harness.ctx.global_style().visuals.dark_mode);
    save_proof(&mut harness, "showcase-green-dark");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Badges")
        .click();
    harness.run();
    harness.get_by_label("Dismiss Removable tag").click();
    harness.run();
    assert!(!harness.state().badge_visible);
    harness.get_by_label("Restore tag").click();
    harness.run();
    assert!(harness.state().badge_visible);
}

#[test]
fn showcase_navigation_leads_with_the_foundation_pages() {
    let mut harness = Harness::builder()
        .with_size([1200.0, 940.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                reduced_motion: true,
                ..Default::default()
            }
        });
    harness.run();
    let top = |label: &str| page_rect(&harness, label).top();
    assert!(top("Overview") < top("Typography"));
    assert!(top("Typography") < top("Tokens"));
    assert!(
        top("Tokens") < top("Accordions"),
        "foundation pages lead the navigation"
    );
    assert!(top("Accordions") < top("Alerts"));
    assert!(top("Alerts") < top("Badges"));
}

#[test]
fn showcase_page_groups_cover_every_page_once_in_sorted_order() {
    use gallery::Page;
    assert_eq!(
        Page::FOUNDATION,
        [Page::Overview, Page::Typography, Page::Tokens]
    );
    let names: Vec<&str> = Page::COMPONENTS.iter().map(|page| page.name()).collect();
    assert!(
        names.windows(2).all(|pair| pair[0] < pair[1]),
        "component pages are sorted by display name: {names:?}"
    );
    // Markdown contributes one page only when its parser feature is enabled.
    assert_eq!(
        Page::all().count(),
        48 + usize::from(cfg!(feature = "markdown"))
    );
    for page in Page::all() {
        assert_eq!(Page::all().filter(|other| *other == page).count(), 1);
    }
}

fn save_proof(harness: &mut Harness<'_, gallery::Showcase>, name: &str) {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    harness
        .render()
        .unwrap()
        .save(format!("target/visual-proof/{name}.png"))
        .unwrap();
}

#[test]
fn showcase_sidebar_pins_the_foundation_footer_and_scrolls_the_pages() {
    let mut harness = Harness::builder()
        .with_size([1200.0, 600.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                reduced_motion: true,
                ..Default::default()
            }
        });
    harness.run();
    let footer = harness.get_by_label("01 / Foundation").rect();
    assert!(
        footer.bottom() <= 600.0,
        "the footer stays inside the viewport: {footer:?}"
    );
    assert!(
        harness
            .query_by_role_and_label(Role::ListBoxOption, "Tree view")
            .is_none()
    );
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Overview")
        .focus();
    harness.run();
    harness.key_press(Key::End);
    harness.run_steps(3);
    assert_eq!(harness.state().page, gallery::Page::TreeView);
    assert_eq!(
        harness.get_by_label("01 / Foundation").rect(),
        footer,
        "the footer does not move while the list scrolls"
    );
    assert!(
        page_rect(&harness, "Tree view").bottom() <= footer.top(),
        "the scrolled page sits above the footer"
    );
    save_proof(&mut harness, "showcase-short");
}

fn page_rect(harness: &Harness<'_, gallery::Showcase>, label: &str) -> egui::Rect {
    harness
        .get_by_role_and_label(Role::ListBoxOption, label)
        .rect()
}

#[test]
fn sidebar_type_ahead_selects_and_reveals_pages_without_overriding_manual_scrolling() {
    use egui::{Modifiers, MouseWheelSource, MouseWheelUnit, TouchPhase, vec2};
    for height in [600.0, 940.0] {
        let mut harness = Harness::builder()
            .with_size([1200.0, height])
            .with_theme(egui::Theme::Light)
            .build_eframe(|cc| {
                common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
                gallery::Showcase {
                    reduced_motion: true,
                    ..Default::default()
                }
            });
        harness.run();
        let footer = harness.get_by_label("01 / Foundation").rect();
        harness.get_by_role_and_label(Role::ListBox, "Showcase pages");
        harness.get_by_label("Reduce motion").focus();
        harness.run();
        harness.key_press(Key::Tab);
        harness.run_steps(2);
        assert!(
            harness
                .get_by_role_and_label(Role::ListBoxOption, "Overview")
                .is_focused()
        );
        harness.key_press(Key::ArrowDown);
        harness.run_steps(2);
        assert_eq!(harness.state().page, gallery::Page::Typography);
        harness.event(Event::Text("toolti".into()));
        harness.run_steps(4);
        assert_eq!(harness.state().page, gallery::Page::Tooltips);
        let option = harness.get_by_role_and_label(Role::ListBoxOption, "Tooltips");
        assert_eq!(option.accesskit_node().is_selected(), Some(true));
        assert!(option.is_focused());
        let focused = option.rect();
        assert!(
            focused.bottom() <= footer.top(),
            "arriving focus must reveal the page above the footer: {focused:?} / {footer:?}"
        );
        assert!(focused.top() >= harness.get_by_label("The system").rect().bottom());
        let outline = focused.shrink(
            dutchy_egui::tokens::border::FOCUS_OFFSET + dutchy_egui::tokens::border::FOCUS_WIDTH,
        );
        assert!(
            harness.output().shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Rect(rect) if rect.rect == outline)
                    && shape.clip_rect.contains_rect(outline)
            }),
            "the entire painted focus outline must fit inside the scrolling viewport"
        );
        harness.hover_at(focused.center());
        harness.run_steps(2);
        harness.input_mut().events.push(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            phase: TouchPhase::Move,
            source: MouseWheelSource::Unknown,
            delta: vec2(0.0, 3000.0),
            modifiers: Modifiers::NONE,
        });
        harness.run_steps(80);
        assert!(page_rect(&harness, "Tooltips").bottom() > footer.top());
        assert!(
            harness
                .get_by_role_and_label(Role::ListBoxOption, "Tooltips")
                .is_focused(),
            "wheel scrolling keeps focus without pulling the page list back"
        );
        assert_eq!(harness.get_by_label("01 / Foundation").rect(), footer);
    }
}

#[test]
fn switching_pages_opens_the_new_page_at_the_top() {
    use egui::{Event, Modifiers, MouseWheelSource, MouseWheelUnit, TouchPhase, pos2, vec2};
    let mut harness = Harness::builder()
        .with_size([1200.0, 700.0])
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Colors,
                reduced_motion: true,
                ..Default::default()
            }
        });
    harness.run();
    let before = harness.get_by_label("Make it your color.").rect();
    harness.hover_at(pos2(700.0, 600.0));
    harness.run_steps(2);
    harness.input_mut().events.push(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        phase: TouchPhase::Move,
        source: MouseWheelSource::Unknown,
        delta: vec2(0.0, -400.0),
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(80);
    assert!(
        harness.get_by_label("Make it your color.").rect().top() < before.top() - 100.0,
        "the first page scrolled"
    );
    harness.state_mut().page = gallery::Page::IconButtons;
    harness.run_steps(2);
    let next = harness.get_by_label("Small targets. Clear actions.").rect();
    assert!(
        (next.top() - before.top()).abs() < 1.0,
        "a newly opened page starts at the top: {next:?} vs {before:?}"
    );
}
