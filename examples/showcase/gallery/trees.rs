use dutchy_egui::{
    Badge, BadgeVariant, Button, ControlSize, ListSelectionMode, Select, SelectOption, Switch,
    Text, TextColor, TextSize, TreeNode, TreeState, TreeView, tokens::space,
};
use egui::Ui;

const KEYS: &str = "Right opens a branch, then enters it. Left closes it or returns \
    to its parent. Up/Down, range selection and typing work like List. Closing a \
    branch keeps selected descendants.";

pub struct TreesState {
    documents: Vec<TreeNode<'static, u32>>,
    tree: TreeState<u32>,
    mode: Option<u32>,
    read_only: bool,
    disabled: bool,
    opened: Option<u32>,
}
impl Default for TreesState {
    fn default() -> Self {
        let mut tree = TreeState::default();
        tree.expanded.extend([1, 2, 3]);
        Self {
            documents: (100..10100)
                .map(|i| TreeNode::new(i, format!("Project brief {:05}.md", i - 100)))
                .collect(),
            tree,
            mode: Some(2),
            read_only: false,
            disabled: false,
            opened: None,
        }
    }
}
pub fn show(ui: &mut Ui, state: &mut TreesState) {
    ui.add(Text::heading("Room to branch.").size(TextSize::Xl5));
    ui.add(
        Text::new("10,000 documents. One shared row engine. Your selection and expansion.")
            .color(TextColor::Muted),
    );
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        let options = [
            SelectOption::new(0, "No selection"),
            SelectOption::new(1, "Single selection"),
            SelectOption::new(2, "Multiple selection"),
        ];
        ui.add(
            Select::new("tree-mode", &mut state.mode, &options)
                .accessible_name("Tree selection mode"),
        );
        ui.add(Switch::new(
            "tree-readonly",
            &mut state.read_only,
            "Read only",
        ));
        ui.add(Switch::new(
            "tree-disabled",
            &mut state.disabled,
            "Disabled",
        ));
    });
    ui.add_space(space::LG);
    let projects = [TreeNode::new(3, "Research archive").children(&state.documents)];
    let folders = [
        TreeNode::new(2, "Canal Studio").children(&projects),
        TreeNode::new(4, "Shared assets"),
        TreeNode::new(5, "Offline archive").enabled(false),
    ];
    let roots = [
        TreeNode::new(1, "Workspace").children(&folders),
        TreeNode::new(6, "Trash"),
    ];
    let mode = match state.mode {
        Some(0) => ListSelectionMode::None,
        Some(1) => ListSelectionMode::Single,
        _ => ListSelectionMode::Multiple,
    };
    let result = TreeView::new("studio", &mut state.tree, &roots)
        .accessible_name("Studio folders")
        .selection_mode(mode)
        .enabled(!state.disabled)
        .read_only(state.read_only)
        .row_height(48.0)
        .max_height(336.0)
        .show_with(ui, |ui, node, row| {
            ui.add(Text::new(node.label.as_ref()).size(TextSize::Sm));
            if row.branch {
                ui.add(Badge::new("Folder").variant(BadgeVariant::Muted));
            } else if node.key >= 100 && node.key % 7 == 0 {
                ui.add(Badge::new("Unread").variant(BadgeVariant::Primary));
            }
        });
    if let Some(key) = result.list.activated {
        state.opened = Some(key);
    }
    ui.add_space(space::MD);
    ui.add(Text::label(format!(
        "{} selected / {} branches open / rows {}–{}",
        state.tree.rows.selected.len(),
        state.tree.expanded.len(),
        result.list.visible_range.start,
        result.list.visible_range.end
    )));
    ui.add(Text::new(KEYS).size(TextSize::Sm).color(TextColor::Muted));
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        if ui
            .add(Button::new("Collapse all").size(ControlSize::Small))
            .clicked()
        {
            state.tree.expanded.clear();
        }
        if ui
            .add(Button::new("Reveal last document").size(ControlSize::Small))
            .clicked()
        {
            state.tree.expanded.extend([1, 2, 3]);
            state.tree.rows.focused = Some(10099);
        }
        if ui
            .add(Button::new("Reset tree").size(ControlSize::Small))
            .clicked()
        {
            *state = TreesState::default();
        }
    });
    if let Some(key) = state.opened {
        ui.add(
            Text::new(format!("Host received open request for node {key}."))
                .color(TextColor::Primary),
        );
    }
}
