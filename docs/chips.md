# Chips

`Chip` renders a compact square label. `selected` makes the body a toggle
button with the current selection exposed to assistive technology. `removable`
adds a separately focusable button named “Remove” followed by the chip label.
Both options can be used on the same chip.

```rust
use dutchy_egui::Chip;

fn show_filter(ui: &mut egui::Ui, selected: &mut bool, visible: &mut bool) {
    if *visible {
        let chip = Chip::new("draft", "Draft")
            .selected(*selected)
            .removable(true)
            .show(ui);
        if chip.remove_requested {
            *visible = false;
        } else if let Some(next) = chip.toggle_requested {
            *selected = next;
        }
    }
}
```

Use a stable key within the parent UI so focus follows the chip when nearby
items change. Tab visits the toggle body and then Remove when both are present.
Enter, Space, pointer clicks and assistive activation raise the corresponding
request. A removable chip with a passive label has one Tab stop. A disabled
chip, including one inside a disabled parent, rejects both actions.

Keep selection, removal, filter policy and focus after removal in the host.
The `remove` response identifies the remove button, and `response` identifies
the body. Chips fit in wrapping rows and shorten long visible labels to fit
while preserving their accessible names. Showcase's **Chips** page filters
documents by status and removes or restores project tags.
