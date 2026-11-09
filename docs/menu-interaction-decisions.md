# Menu interaction decisions

DS-075 reviews the expanded command tree introduced by DS-073. The screenshot
showed two active sibling rows, detached full-width children, and a child row
slightly below its parent. These are input ownership and geometry problems;
they do not require changing the entire menu metaphor.

## Chosen desktop presentation

Keep cascading menus and their existing arrow/Enter/Escape navigation. Connect
the frames, align row starts, and fit child widths to labels, checks, shortcuts
and arrows. Preserve the shared trailing inset for arrows and hints. The root
still uses the host's preferred MenuWidth; children have a 200-point minimum,
capped by that width and the viewport. A short format list therefore takes
less horizontal space than a wider root. Native popup flipping and scrolling
remain responsible for edge constraints.

Use one accent target per input owner. Real pointer movement switches to hover
ownership; a key or assistive action switches back to keyboard ownership. Open
ancestors keep muted context and an accent arrow instead of competing orange
fills. When the pointer owns
the accent and rests outside the rows, the keyboard-focused row shows the same
muted context (DS-082), so Enter never acts on an unmarked row. Their labels keep the same weight and geometry before and after opening.
Pointer hover follows menu focus only when the menu already owns it, preserving
typing in an external editor beside an inline menu.

Immediate Escape closes one level. Tab leaves the entire command surface in
the direction recorded on the key event. Shortcut hints remain descriptive;
the host owns command execution, checked values and key bindings.

## Alternative considered

A single-panel drill-down menu could replace cascades on narrow windows or for
deep command trees. A persistent path/back action would keep the panel at one
width and avoid sideways pointer travel. It also hides sibling context, adds
backtracking and needs a distinct keyboard/accessibility contract. Treat it as
an opt-in presentation if a real application needs it; do not silently change
desktop menu behavior based on a screenshot. Searchable flat commands already
have SearchableSelect and CommandPalette APIs; this work does not merge those
patterns into menus.

## Evidence

See [DS-074](issues/DS-074.md), [DS-075](issues/DS-075.md) and the
[verification report](verification-menu-polish.md) for native event reproduction,
interaction regressions, theme/scale rendering and qualification limits.
