//! Form controls stay readable inside inverse (Dark/Primary) card surfaces.
#[cfg(feature = "bundled-fonts")]
mod common;
#[cfg(feature = "bundled-fonts")]
mod rendering {
    use super::common;
    use dutchy_egui::{Card, CardVariant, Checkbox, Field, Input, Theme};
    use egui::{Rect, pos2};
    use egui_kittest::Harness;

    struct State {
        value: String,
        agree: bool,
        input: Rect,
        checkbox: Rect,
    }
    fn harness(variant: CardVariant) -> Harness<'static, State> {
        Harness::builder()
            .with_size([420.0, 320.0])
            .with_pixels_per_point(1.0)
            .with_theme(egui::Theme::Light)
            .with_context_setup(|ctx| {
                common::install(ctx, Theme::default().with_reduced_motion(true));
            })
            .build_ui_state(
                move |ui, state: &mut State| {
                    Card::new().variant(variant).show(ui, |ui| {
                        state.input = Field::new("Record name")
                            .required(true)
                            .help("Shown in the host workspace.")
                            .show(ui, |ui| ui.add(Input::new("name", &mut state.value)))
                            .rect;
                        state.checkbox = ui
                            .add(Checkbox::new(
                                "agree",
                                &mut state.agree,
                                "Send product updates",
                            ))
                            .rect;
                    });
                },
                State {
                    value: "Design studio".into(),
                    agree: false,
                    input: Rect::NOTHING,
                    checkbox: Rect::NOTHING,
                },
            )
    }
    fn count<'p, P>(
        pixels: impl Iterator<Item = (u32, u32, &'p P)>,
        area: Rect,
        light: bool,
    ) -> usize
    where
        P: std::ops::Index<usize, Output = u8> + 'p,
    {
        pixels
            .filter(|(x, y, _)| area.contains(pos2(*x as f32, *y as f32)))
            .filter(|(_, _, pixel)| {
                let luma = u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2]);
                (light && luma > 600) || (!light && luma < 200)
            })
            .count()
    }
    #[test]
    fn labels_and_editor_text_are_visible_on_dark_and_primary_cards() {
        for variant in [CardVariant::Dark, CardVariant::Primary] {
            let mut h = harness(variant);
            h.run_steps(3);
            let image = h.render().unwrap();
            let input = h.state().input;
            let label = Rect::from_min_max(
                pos2(input.left(), input.top() - 30.0),
                pos2(input.right(), input.top() - 2.0),
            );
            assert!(
                count(image.enumerate_pixels(), label, true) > 20,
                "{variant:?}: the field label must contrast with the surface"
            );
            assert!(
                count(image.enumerate_pixels(), input.shrink(8.0), false) > 20,
                "{variant:?}: editor text must stay dark on the light editor"
            );
            let checkbox = h.state().checkbox;
            let caption = Rect::from_min_max(
                pos2(checkbox.left() + 40.0, checkbox.top()),
                checkbox.right_bottom(),
            );
            assert!(
                count(image.enumerate_pixels(), caption, true) > 20,
                "{variant:?}: the checkbox label must contrast with the surface"
            );
        }
    }
}
