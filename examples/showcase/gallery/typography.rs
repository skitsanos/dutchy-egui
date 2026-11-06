use super::section;
use dutchy_egui::tokens::{border, space};
use dutchy_egui::{Family, Text, TextColor, TextSize, Theme, Tracking, Weight};
use egui::{Frame, Margin, Stroke, Ui, vec2};

pub(super) fn show(ui: &mut Ui) {
    ui.add(Text::heading("Type with purpose.").size(TextSize::Xl5));
    for (family, title) in [
        (Family::Display, "Space Grotesk"),
        (Family::Body, "Inter"),
        (Family::Mono, "JetBrains Mono"),
    ] {
        section(ui, "01", title, |ui| {
            for (weight, label) in [
                (Weight::Regular, "Regular 400"),
                (Weight::Medium, "Medium 500"),
                (Weight::SemiBold, "Semibold 600"),
                (Weight::Bold, "Bold 700"),
            ] {
                ui.add(
                    Text::new(format!("{label} — Design with clarity."))
                        .family(family)
                        .weight(weight)
                        .size(TextSize::Xl2),
                );
            }
        });
    }
    section(ui, "02", "12–96 point scale", |ui| {
        for size in [
            TextSize::Xs,
            TextSize::Sm,
            TextSize::Base,
            TextSize::Lg,
            TextSize::Xl,
            TextSize::Xl2,
            TextSize::Xl3,
            TextSize::Xl4,
            TextSize::Xl5,
            TextSize::Xl6,
            TextSize::Xl7,
            TextSize::Xl8,
        ] {
            ui.add(Text::heading(format!("{} / Dutchy", size.points())).size(size));
        }
    });
    section(ui, "03", "Tracking", |ui| {
        for tracking in [
            Tracking::Tighter,
            Tracking::Tight,
            Tracking::Normal,
            Tracking::Wide,
            Tracking::Wider,
            Tracking::Widest,
        ] {
            ui.add(
                Text::new(format!("{:?} / Architectural clarity", tracking))
                    .tracking(tracking)
                    .size(TextSize::Xl),
            );
        }
    });
}

pub(super) fn tokens(ui: &mut Ui, theme: Theme) {
    ui.add(Text::heading("A warm foundation.").size(TextSize::Xl5));
    let p = theme.palette;
    section(ui, "01", "Semantic palette", |ui| {
        for (name, color) in [
            ("Primary", p.primary),
            ("Background", p.background),
            ("Foreground", p.foreground),
            ("Border", p.border),
            ("Muted", p.muted),
            ("Muted foreground", p.muted_foreground),
            ("Success", p.success),
            ("Warning", p.warning),
            ("Destructive", p.destructive),
            ("Info", p.info),
        ] {
            ui.horizontal(|ui| {
                let (rect, _) = ui
                    .allocate_exact_size(vec2(space::LARGE, space::SECTION), egui::Sense::hover());
                ui.painter().rect(
                    rect,
                    0,
                    color,
                    Stroke::new(border::THIN, p.border),
                    egui::StrokeKind::Inside,
                );
                ui.add(
                    Text::new(format!(
                        "{name} / #{:02X}{:02X}{:02X}",
                        color.r(),
                        color.g(),
                        color.b()
                    ))
                    .family(Family::Mono)
                    .size(TextSize::Sm),
                );
            });
        }
    });
    section(ui, "02", "Borders and spacing", |ui| {
        for width in [
            border::THIN,
            border::STANDARD,
            border::ACCENT,
            border::HEAVY,
        ] {
            Frame::NONE
                .stroke(Stroke::new(width, p.foreground))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.add(Text::label(format!("{width} point border")));
                });
        }
        ui.add_space(space::LG);
        ui.add(
            Text::new(
                "4-point spacing rhythm. Control heights: 32 / 48 / 56. Sharp corners throughout.",
            )
            .color(TextColor::Muted),
        );
    });
}
