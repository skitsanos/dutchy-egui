# Color picker

`ColorPicker` edits an application-owned `egui::Color32`. It is a native Dutchy
addition, independent of the web registry. The trigger uses shared control
heights, a transparency preview and a hex readout. Its anchored editor contains
flat palette buttons, hex input, channel sliders and stepped numeric inputs.
All surfaces use Dutchy tokens; no extra runtime dependency is introduced.

```rust
use dutchy_egui::{ColorPicker, ColorPreset, Field};
use egui::Color32;

# fn demo(ui: &mut egui::Ui, color: &mut Color32) {
let palette = [
    ColorPreset::new("Ink", Color32::from_rgb(18, 15, 12)),
    ColorPreset::new("Orange", Color32::from_rgb(249, 116, 21)),
];
let response = Field::new("Annotation color").show(ui, |ui| {
    ui.add(ColorPicker::new("annotation-color", color)
        .alpha(true)
        .palette(&palette)
        .accessible_name("Annotation color"))
});
if response.changed() {
    // Persist, repaint or invalidate an application-owned preview here.
}
# }
```

## Editing contract

- Values commit immediately. `Response::changed()` reports changes to the bound
  color; opening, closing and invalid text do not count as changes. Escape and
  Done close without rolling back edits. A host needing Apply/Cancel binds a draft.
- Default RGB mode preserves the host's alpha and accepts three/six hexadecimal
  digits, optionally prefixed by `#`. Alpha mode also accepts four/eight digits,
  with alpha last; RGB-only text in alpha mode sets alpha to 255. Hex is sRGB.
- Invalid and partial text remains editable, never corrupts the host value,
  and resets to the current color on blur or close. External host changes win.
- The default palette uses semantic theme colors. `.palette(...)` replaces it;
  an empty palette hides it. Give presets distinct names. Choosing a preset
  keeps the editor open. RGB mode preserves the current alpha on palette edits.
- `.read_only(true)` allows opening, inspecting and copying while blocking edits.
  `.enabled(false)` and disabled parents block interaction and close the popup.
- Color32 uses premultiplied storage. Temporary straight channels retain useful
  RGB while alpha is zero; this is transient editor state, not serialized color
  data. Other low-alpha conversions retain Color32's normal rounding limits.

## Keyboard and accessibility

The named ColorWell trigger exposes the selected color and popup state.
Enter/Space opens it; initial focus moves to hex after popup sizing. Palette
buttons, hex, sliders, number inputs, Copy hex and Done retain native egui
keyboard and assistive actions. Channels have descriptive names and bounds.
Escape/Done return focus to the trigger. Outside clicks retain their target's
normal behavior. Copy hex copies the canonical displayed representation.

## Scope

Shared Small/Medium/Large triggers, viewport-constrained scrolling, stable IDs,
named palettes, RGB/alpha, hex and copy are implemented.
HSV/HSL entry, eyedropper permissions, saved palette/history services and
wide-gamut/HDR color management are future additions. The component does not
change the application theme; the showcase demonstrates host-owned previews.
