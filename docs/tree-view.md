# TreeView

`TreeView` adds hierarchy to the shared List row engine: the same fixed-height
virtualization, keyed identity, pointer/text handling, selection ranges,
type-ahead and roving Tab stop. No helper crate or new runtime dependency.

```rust
use dutchy_egui::{egui, TreeNode, TreeState, TreeView};

fn folders(ui: &mut egui::Ui, state: &mut TreeState<u64>) {
    let files = [TreeNode::new(2, "Brief.md"), TreeNode::new(3, "Research.md")];
    let roots = [TreeNode::new(1, "Projects").children(&files)];
    let result = TreeView::new("folders", state, &roots)
        .accessible_name("Project files")
        .empty_text("No project files")
        .show(ui);
    if let Some(key) = result.list.activated {
        // The host performs the node's action.
    }
}
```

The host retains `TreeState`: `rows` is ListState (selected keys, focused key,
range anchor and transient prefix), and `expanded` is a set of branch keys.
The host also owns node data/order and actions. Node descriptors borrow child
slices; construct them from application data when drawing. Empty children
mean a leaf. Keys must be unique throughout the complete tree. Later duplicate
subtrees are ignored. Traversal uses an explicit stack rather than recursion.

## Interaction

| Input | Result |
| --- | --- |
| Right on closed branch | Expand, keeping focus and selection |
| Right on open branch | Move to its first enabled child; selection follows |
| Right on leaf | No action |
| Left on open branch | Collapse, keeping focus and selection |
| Left on closed branch or leaf | Move to its parent; selection follows |
| Disclosure click | Toggle expansion and focus the row; keep selection |
| Row/text click | Select and focus using List's modifier model |
| Enter / double-click on the row body | Return an activation request for the host |

The [W3C tree pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)
defines the navigation model. Left/Right use unmodified keys; modified
horizontal shortcuts remain with the host. Up/Down, Home/End, Page keys, Space,
Command selection/ranges, prefix search and Tab/Shift+Tab use
[List's behavior](lists.md). Search and range selection operate on the flattened open
view; Command means Control on Windows/Linux and Command on macOS. Selection is
per key, without cascading checkbox state. Enter does not automatically open
branches.

Collapsing keeps selected descendants and their nested expansion. If focus
becomes hidden, it moves to the nearest visible ancestor without selecting
that ancestor. Removing keys prunes selection/expansion. Disabling a branch
disables its subtree and prunes its selection, while retaining expansion.
Selecting a new row normally replaces hidden selections too; additive gestures
retain them. Changing to Single keeps the first selected enabled key in full
tree order; None clears selection.

`result.list.response.changed()` reports widget selection changes;
`result.expansion_changed` reports widget expansion changes/pruning. Host edits
made before drawing are already the supplied state. Read-only suppresses
selection, expansion and activation edits while retaining navigation through
already-open branches. Disabled suppresses all interaction and scrolling.

## Rendering and accessibility

Rows use List's size/height limits, surfaces, focus policy and clipped passive
content. Tree tokens add a 20-point indent and a 24-point disclosure slot.
`show_with` receives `TreeRow` after the slot; labels remain non-selectable so
they cannot intercept row clicks. Use passive text/icons/badges. Embedded
interactive controls, variable-height rows, drag/reorder, lazy loading and the
optional `*` expand-siblings shortcut are not supported.

Only open branches contribute rows. Only viewport rows paint content, with
egui overscan. Focus proxies and required ancestor nodes survive virtualization
without drawing hidden content. Tree/TreeItem roles include expansion,
selection, levels and sibling positions, both zero-based as AccessKit defines
them; platform adapters report levels from one. Child groups carry their
complete sibling counts. Assistive Expand/Collapse actions use the
same keyed expansion path. Metadata traversal/validation scans supplied nodes;
this is row virtualization, not lazy data virtualization.

The showcase's **Tree view** page demonstrates a folder hierarchy over 10,000
documents, passive badges, mode switches, collapse and programmatic reveal.
