use dutchy_egui::validation::ValidationState;
use dutchy_egui::{Field, TagInput, TagInputRequest, TagItem, Text, TextSize, tokens::space};
use egui::Ui;

pub struct TagsState {
    tags: Vec<(usize, String)>,
    next_key: usize,
    draft: String,
    validation: ValidationState,
}

impl Default for TagsState {
    fn default() -> Self {
        Self {
            tags: ["Design system", "Typography", "Layout"]
                .into_iter()
                .map(String::from)
                .enumerate()
                .collect(),
            next_key: 3,
            draft: String::new(),
            validation: ValidationState::Unchecked,
        }
    }
}

impl TagsState {
    fn has_tag(&self, label: &str) -> bool {
        self.tags
            .iter()
            .any(|(_, tag)| tag.eq_ignore_ascii_case(label))
    }

    fn apply(&mut self, request: TagInputRequest<usize>) {
        match request {
            TagInputRequest::Add(label) => {
                if self.has_tag(&label) {
                    self.validation = ValidationState::Invalid("This tag already exists.".into());
                    return;
                }
                self.tags.push((self.next_key, label));
                self.next_key += 1;
                self.draft.clear();
                self.validation = ValidationState::Unchecked;
            }
            TagInputRequest::Remove(key) => {
                self.tags.retain(|(tag_key, _)| *tag_key != key);
                if !self.has_tag(self.draft.trim()) {
                    self.validation = ValidationState::Unchecked;
                }
            }
            _ => {}
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut TagsState) {
    const LABEL: &str = "Project tags";
    ui.add(Text::heading("Tag input").size(TextSize::Xl5));
    ui.add_space(space::LG);
    let items: Vec<_> = state
        .tags
        .iter()
        .map(|(key, label)| TagItem::new(*key, label.as_str()).removable(*key != 0))
        .collect();
    let mut requests = Vec::new();
    let editor = Field::new(LABEL)
        .help("Design system is required for this project.")
        .validation(&state.validation)
        .show(ui, |ui| {
            let result = TagInput::new("project-tags", &mut state.draft, &items)
                .accessible_name(LABEL)
                .show(ui);
            requests = result.requests;
            result.response
        });
    if editor.changed() {
        state.validation = ValidationState::Unchecked;
    }
    if !requests.is_empty() {
        for request in requests {
            state.apply(request);
        }
        ui.ctx().request_repaint();
    }
}
