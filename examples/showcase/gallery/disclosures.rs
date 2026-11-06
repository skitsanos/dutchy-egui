use super::section;
use dutchy_egui::{
    Accordion, AccordionItem, AccordionState, AccordionVariant, Button, Field, Input, Text,
    TextColor, TextSize,
};

pub struct DisclosureState {
    pub single: AccordionState<&'static str>,
    pub multiple: AccordionState<&'static str>,
    pub name: String,
    pub applies: u32,
}
impl Default for DisclosureState {
    fn default() -> Self {
        Self {
            single: AccordionState::Single(Some("basics")),
            multiple: AccordionState::Multiple(vec!["accessibility"]),
            name: "Studio workspace".into(),
            applies: 0,
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut DisclosureState) {
    ui.add(Text::label("Interaction components / Accordions").color(TextColor::Primary));
    ui.add(Text::heading("Reveal what matters.").size(TextSize::Xl4));
    ui.add(
        Text::new(
            "Enter or Space toggles a section. Up/Down and Home/End move between enabled headers.",
        )
        .color(TextColor::Muted),
    );
    section(ui, "01", "One section at a time", |ui| {
        let items = [
            AccordionItem::new("basics", "What is Dutchy?"),
            AccordionItem::new("controls", "Can I compose interactive content?"),
            AccordionItem::new("future", "Unavailable section").enabled(false),
        ];
        Accordion::new("single", &mut state.single, &items).accessible_name("Getting started").show(ui, |ui, key| {
            if *key == "basics" {
                ui.add(Text::new("Sharp geometry, warm colors, and clear hierarchy. The host owns which section is open.").size(TextSize::Sm));
            } else {
                Field::new("Example workspace name").show(ui, |ui| ui.add(Input::new("name", &mut state.name)));
                if ui.add(Button::new("Apply example name")).clicked() { state.applies += 1; }
                ui.add(Text::new(format!("Applied {} times. Editing and actions do not toggle the header.", state.applies)).size(TextSize::Sm));
            }
        });
    });
    section(ui, "02", "Independent sections / shared border", |ui| {
        let items = [
            AccordionItem::new("accessibility", "Accessible by composition"),
            AccordionItem::new("state", "Keep durable state in the application"),
            AccordionItem::new(
                "long",
                "Long headings wrap inside the available column, leaving the disclosure icon visible",
            ),
        ];
        Accordion::new("multiple", &mut state.multiple, &items).variant(AccordionVariant::Bordered).accessible_name("Reference notes").show(ui, |ui, key| {
            let text = match *key {
                "accessibility" => "Named native buttons expose expansion state and control their visible regions. Focus rings remain sharp.",
                "state" => "Expansion, text values, and action execution stay with the host. Closing a section does not erase its data.",
                _ => "Body text wraps too. Nested controls keep their normal keyboard and pointer behavior.",
            };
            ui.add(Text::new(text).size(TextSize::Sm));
        });
    });
}
