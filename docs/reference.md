# Reference decisions

The source revision and overall plan are in [plan.md](plan.md).

## Colors

Canonical current CSS uses HSL values. Older prose hex labels sometimes disagree
with those values. The port rounds the CSS values into sRGB bytes:

| Role | Port value | Source |
| --- | --- | --- |
| Orange | #F97415 | hsl(25 95% 53%) |
| Purple | #8249DF | hsl(263 70% 58%) |
| Crimson | #EB1414 | hsl(0 84% 50%) |
| Blue | #2463EB | hsl(221 83% 53%), port addition |
| Green | #0D9165 | hsl(160 84% 31%), port addition |
| Background | #FAF8F5 | hsl(40 30% 97%) |
| Foreground | #120F0C | hsl(25 20% 6%) |
| Border | #E7DFDA | hsl(25 20% 88%) |
| Secondary | #F1EAE4 | hsl(25 30% 92%) |
| Secondary text | #30251D | hsl(25 25% 15%) |
| Muted | #F3F1ED | hsl(40 20% 94%) |
| Muted text | #70645C | hsl(25 10% 40%) |
| Destructive | #EF4343 | hsl(0 84% 60%) |
| Success | #16A249 | hsl(142 76% 36%) |
| Warning | #F59F0A | hsl(38 92% 50%) |
| Info | #0973DC | hsl(210 92% 45%) |

Delft Blue and Emerald Green are accents of this port: the web stylesheet
defines only orange, purple and crimson. Blue neighbours the Info status hue
(221 against 210) and Green the Success hue (160 against 142), as Crimson
neighbours Destructive; each accent differs from its status neighbour in
lightness so the pair stays distinguishable side by side. Both fills reach at
least 3.8:1 against each neutral background, and white text reaches 5.2:1 on
Blue and 4.0:1 on Green. `Accent::ALL` lists every preset in documented order
for theme pickers.

Light colors match the measured localhost reference. Dark neutrals use the
documented #0A0A0A background, #FAFAFA foreground, #262626 muted surface, #A3A3A3
muted text, and #333333 border. Secondary dark surfaces map to muted; status
colors and accents retain their source values. This completes a documented
theme whose runtime overrides are absent from the current web stylesheet.

White text on primary/status fills follows the source by decision
([DS-003](issues/DS-003.md)); several combinations have limited contrast.
Status and accent text on neutral surfaces keeps the source colors as well
([DS-027](issues/DS-027.md)). Palette fields are public so applications can
choose appropriate accessible text/fill pairs, including `on_primary` and
`on_status`, and `TextColor::Custom` sets an explicit color on any text the
host renders.

## Geometry and behavior

- Logical points preserve the web scale at 1x; host display scaling remains intact.
- Shared control heights are 32/48/56; paddings are 16/32/40 horizontally.
- Borders use 1/2/4/8; focus is a 2-point outline with a 2-point gap.
  Shared focus cues follow keyboard/assistive input; pointer activation retains
  native focus without an extra ring. The input mode persists in egui's
  temporary memory, so programmatic focus shows a cue until the first pointer
  press and again after the next key, text, IME, clipboard or assistive event.
  Accordion rings are inset to keep all four edges clear of adjacent fills.
  Selected tabs use their active mark alone (DS-011); WCAG 2.4.7 expects a
  visible focus indicator on every focusable item, so a keyboard user entering
  a strip on the selected tab sees no change (recorded in DS-036).
- No gradients, rounded corners, or blurred shadows are introduced. Borders are
  solid except the file drop zone, whose dashed 2-point edge follows the
  file-upload component and its live implementation; the effects token page's
  "no dashed borders" statement is a documented exception.
- Buttons use egui's standard control for pointer/keyboard semantics inside a
  scoped variant style. Disabled/loading state also honors a disabled parent.
- Badges dismiss only through the close affordance and report a named button.
- The spinner paints three square edges, reserves the full rotated bounds, and
  uses the documented 0.6-second rotation. Hidden/paused/reduced-motion indicators
  do not request animation frames.
- Custom text supports em tracking and point line height; named presets match
  the documented scale. Bundled face weights are selected explicitly.
- Native egui controls inherit the theme; they do not automatically acquire
  every web component variant, tracking rule, or behavior. The style is
  installed for both egui themes: `Mode` selects the palette, so a later
  theme-preference change cannot drop native widgets to egui defaults.
- Uppercase is a visual treatment. Buttons, tabs, accordion headers,
  breadcrumbs, badges, headings, field labels, and uppercase `Text` expose
  their accessible names and values as the host wrote them.
- Focused controls are revealed once, when focus arrives or when layout moves
  them relative to their container; manual scrolling is never undone.
