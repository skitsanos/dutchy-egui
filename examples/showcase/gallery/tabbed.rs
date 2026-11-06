use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, Field, Input, Stat, StatSize, Tab, Tabs,
    TabsVariant, Text, TextColor, TextSize,
};
use std::collections::BTreeMap;

pub struct TabbedState {
    pub selected: Option<&'static str>,
    pub billing: Option<&'static str>,
    pub documents: Vec<u32>,
    pub document: Option<u32>,
    pub next_document: u32,
    pub notes: BTreeMap<u32, String>,
    pub workspace: String,
    pub saves: u32,
}
impl Default for TabbedState {
    fn default() -> Self {
        Self {
            selected: Some("overview"),
            billing: Some("monthly"),
            documents: vec![1, 2, 3, 4, 5],
            document: Some(1),
            next_document: 6,
            notes: BTreeMap::new(),
            workspace: "Studio workspace".into(),
            saves: 0,
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut TabbedState) {
    ui.add(Text::label("Interaction components / Tabs").color(TextColor::Primary));
    ui.add(Text::heading("One view. Clear choices.").size(TextSize::Xl4));
    ui.add(Text::new("Left/Right and Home/End move focus. Enter or Space activates. Delete requests closing a document.").color(TextColor::Muted));
    section(ui, "01", "Content navigation", |ui| {
        let items = [
            Tab::new("overview", "Overview"),
            Tab::new("settings", "Workspace settings"),
            Tab::new("history", "Audit history").enabled(false),
        ];
        Tabs::new("content", &mut state.selected, &items)
            .accessible_name("Workspace views")
            .show(ui, |ui, key| {
                if *key == "overview" {
                    ui.add(
                        Stat::new("Settings saved", state.saves.to_string()).size(StatSize::Small),
                    );
                    ui.add(Text::new(&state.workspace).size(TextSize::Sm));
                } else {
                    Field::new("Tabbed workspace name").show(ui, |ui| {
                        ui.add(Input::new("workspace", &mut state.workspace))
                    });
                    if ui.add(Button::new("Save tab settings")).clicked() {
                        state.saves += 1;
                    }
                    if state.saves > 0 {
                        Alert::new("saved", "Settings saved in the host")
                            .variant(AlertVariant::Success)
                            .show(ui);
                    }
                }
            });
    });
    section(ui, "02", "Segmented mode switch", |ui| {
        let items = [Tab::new("monthly", "Monthly"), Tab::new("yearly", "Yearly")];
        Tabs::new("billing", &mut state.billing, &items)
            .variant(TabsVariant::Segmented)
            .accessible_name("Billing interval")
            .show(ui, |ui, key| {
                ui.add(
                    Text::new(if *key == "monthly" {
                        "€24 per month / sample pricing"
                    } else {
                        "€240 per year / sample pricing"
                    })
                    .size(TextSize::Sm),
                );
            });
    });
    section(ui, "03", "Scrollable documents / host-owned close", |ui| {
        documents(ui, state)
    });
}

fn documents(ui: &mut egui::Ui, state: &mut TabbedState) {
    let add = ui.add(Button::new("Add document").variant(ButtonVariant::Outline));
    if add.clicked() {
        let id = state.next_document;
        state.next_document += 1;
        state.documents.push(id);
        state.document = Some(id);
    }
    let items: Vec<_> = state
        .documents
        .iter()
        .map(|id| Tab::new(*id, format!("Document {id}")).closable(true))
        .collect();
    let result = Tabs::new("documents", &mut state.document, &items)
        .accessible_name("Open documents")
        .focus_when_empty(add.id)
        .show(ui, |ui, key| {
            let note = state.notes.entry(*key).or_default();
            Field::new(format!("Document {key} note")).show(ui, |ui| {
                ui.add(Input::new("note", note).placeholder("A note kept by the application"))
            });
        });
    if let Some(closed) = result.close_requested {
        close_document(state, closed);
        ui.ctx().request_repaint();
    }
    if state.documents.is_empty() {
        ui.add(
            Text::new("No open documents. Add one to begin.")
                .size(TextSize::Sm)
                .color(TextColor::Muted),
        );
    }
    ui.add(Text::new("Closing a tab retains its note in this example. Your application decides whether to save or discard data.").size(TextSize::Sm).color(TextColor::Muted));
}

fn close_document(state: &mut TabbedState, closed: u32) {
    let Some(index) = state.documents.iter().position(|id| *id == closed) else {
        return;
    };
    state.documents.remove(index);
    if state.document == Some(closed) {
        state.document = state
            .documents
            .get(index.min(state.documents.len().saturating_sub(1)))
            .copied();
    }
}
