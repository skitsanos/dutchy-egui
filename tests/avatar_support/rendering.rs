use dutchy_egui::{
    Accent::{Blue, Orange},
    Avatar, AvatarSize, AvatarVariant,
    Mode::{Dark, Light},
    Presence, Theme,
};
use egui_kittest::{Harness, SnapshotResults};

const PEOPLE: [&str; 4] = [
    "Ada Lovelace",
    "Grace Hopper",
    "Alan Turing",
    "Emmy Noether",
];

#[test]
fn avatars_render_variants_sizes_and_presence_across_themes_and_scales() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Orange, Light, 1.0, "light-1x"),
        (Orange, Dark, 1.5, "dark-1_5x"),
        (Blue, Light, 2.0, "blue-2x"),
    ] {
        let mut h = Harness::builder()
            .with_size([300.0, 300.0])
            .with_pixels_per_point(scale)
            .with_context_setup(move |ctx| {
                super::common::install(ctx, Theme::new(accent, mode).with_reduced_motion(true))
            })
            .build_ui(|ui| {
                for variant in AvatarVariant::ALL {
                    ui.horizontal(|ui| {
                        for ((size, person), presence) in
                            AvatarSize::ALL.iter().zip(PEOPLE).zip(Presence::ALL)
                        {
                            ui.add(
                                Avatar::new(person)
                                    .size(*size)
                                    .variant(*variant)
                                    .presence(*presence),
                            );
                        }
                    });
                }
            });
        h.run_steps(2);
        h.check_accessibility();
        h.snapshot(format!("avatar-{name}"));
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
