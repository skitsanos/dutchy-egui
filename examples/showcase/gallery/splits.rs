use dutchy_egui::{
    Button, ControlSize, SplitPane, Switch, Text, TextColor, TextSize, Textarea, TreeNode,
    TreeState, TreeView, tokens::space,
};
use egui::{Frame, Margin, Ui};

const TITLES: [&str; 3] = ["Design notes.md", "Release checklist.md", "Ideas.md"];
const NOTES: [&str; 3] = [
    "# A little room to think\n\nDrag the dividers to make space for your work.\n\n- Browse the notebook\n- Edit the source\n- Watch the preview follow along\n\n**Your draft stays with the application.**",
    "# Ready to ship\n\n- [x] Review the changes\n- [x] Run the checks\n- [ ] Write the release note\n\n> Make the next step clear.",
    "# Small ideas, big canvas\n\nA reading pane, an editor, a notebook. Compose them into a workspace that fits the task.",
];
pub struct SplitsState {
    left: f32,
    top: f32,
    tree: TreeState<usize>,
    sources: [String; 3],
    current: usize,
    read_only: bool,
    disabled: bool,
}
impl Default for SplitsState {
    fn default() -> Self {
        let mut tree = TreeState::default();
        tree.expanded.insert(10);
        tree.rows.selected.insert(0);
        Self {
            left: space::XXL * 10.0,
            top: space::XXL * 9.0,
            tree,
            sources: NOTES.map(str::to_owned),
            current: 0,
            read_only: false,
            disabled: false,
        }
    }
}
pub fn show(ui: &mut Ui, state: &mut SplitsState) {
    ui.add(Text::heading("Room to think.").size(TextSize::Xl5));
    ui.add(
        Text::new("A notebook, a live preview, and space you can make your own.")
            .color(TextColor::Muted),
    );
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        ui.add(Switch::new(
            "split-readonly",
            &mut state.read_only,
            "Lock dividers",
        ));
        ui.add(Switch::new(
            "split-disabled",
            &mut state.disabled,
            "Disabled",
        ));
        if ui
            .add(Button::new("Reset sizes").size(ControlSize::Small))
            .clicked()
        {
            state.left = space::XXL * 10.0;
            state.top = space::XXL * 9.0;
        }
    });
    ui.add_space(space::LG);
    let current = state.current;
    let read_only = state.read_only;
    let r = SplitPane::horizontal("notebook", &mut state.left)
        .height(space::HERO * 10.0)
        .min_sizes(space::LG * 10.0, space::XXL * 10.0)
        .accessible_name("Notebook")
        .enabled(!state.disabled)
        .read_only(read_only)
        .show(
            ui,
            |ui| {
                Frame::NONE
                    .inner_margin(Margin::same(space::MD as i8))
                    .show(ui, |ui| {
                        ui.add(Text::label("Notebook").color(TextColor::Muted));
                        let leaves = TITLES
                            .into_iter()
                            .enumerate()
                            .map(|(i, t)| TreeNode::new(i, t))
                            .collect::<Vec<_>>();
                        let nodes = [TreeNode::new(10, "Studio notes").children(&leaves)];
                        TreeView::new("notes", &mut state.tree, &nodes)
                            .accessible_name("Notebook files")
                            .max_height(ui.available_height())
                            .show(ui);
                        state
                            .tree
                            .rows
                            .selected
                            .iter()
                            .copied()
                            .find(|&i| i < TITLES.len())
                    })
                    .inner
            },
            |ui| {
                let source = &mut state.sources[current];
                let preview_source = source.clone();
                SplitPane::vertical("document", &mut state.top)
                    .height(ui.available_height())
                    .min_sizes(space::PAGE, space::HERO * 3.0)
                    .read_only(read_only)
                    .accessible_name("Document preview")
                    .show(
                        ui,
                        |ui| preview(ui, &preview_source, TITLES[current]),
                        |ui| {
                            Frame::NONE
                                .inner_margin(Margin::same(space::LG as i8))
                                .show(ui, |ui| {
                                    ui.add(Text::label("Markdown source").color(TextColor::Muted));
                                    egui::ScrollArea::vertical().id_salt("source-scroll").show(
                                        ui,
                                        |ui| {
                                            ui.add(
                                                Textarea::new("source", source)
                                                    .accessible_name("Note source")
                                                    .rows(4)
                                                    .show_count(false),
                                            );
                                        },
                                    );
                                });
                        },
                    );
            },
        );
    if let Some(next) = r.first.inner
        && next != state.current
    {
        state.current = next;
        ui.ctx().request_repaint();
    }
    ui.add_space(space::MD);
    ui.add(Text::label(format!(
        "Notebook {:.0} pt / Preview {:.0} pt",
        state.left, state.top
    )));
    ui.add(Text::new("Focus a divider, then use its arrow keys. Shift moves faster; Home and End reach the limits.").size(TextSize::Sm).color(TextColor::Muted));
}
fn preview(ui: &mut Ui, source: &str, title: &str) {
    Frame::NONE
        .inner_margin(Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.add(Text::label(title).color(TextColor::Muted));
            egui::ScrollArea::vertical()
                .id_salt("preview-scroll")
                .show(ui, |ui| {
                    #[cfg(feature = "markdown")]
                    dutchy_egui::Markdown::new("note-preview", source)
                        .accessible_name("Note preview")
                        .show(ui);
                    #[cfg(not(feature = "markdown"))]
                    ui.add(Text::new(source));
                });
        });
}
