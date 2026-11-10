# Stepper

`Stepper` describes a linear workflow. Its immutable `StepperState<T>` identifies
`NotStarted`, `Current(key)`, or `Complete`; it never advances or submits work.
Earlier steps are displayed as completed, the matching key is current, and later
steps are upcoming. Keys must be unique and stable. Rebuild `Step` values when
changing keys; labels and descriptions can change without changing identity.

```rust
use dutchy_egui::{Step, Stepper, StepperState};

fn navigation(ui: &mut egui::Ui, state: &mut StepperState<u8>) {
    let steps = [
        Step::new(1, "Account").navigable(true),
        Step::new(2, "Details").description("Name the workspace."),
        Step::new(3, "Review"),
    ];
    let response = Stepper::new("setup", state, &steps)
        .accessible_name("Workspace setup")
        .show(ui);
    if let Some(key) = response.requested_step {
        // Apply only after the application's own navigation/validation policy.
        *state = StepperState::Current(key);
    }
}
```

## Presentations

| `StepperVariant` | Behavior |
| --- | --- |
| `Horizontal` (default) | Equal columns, 40-point squares, check marks for completed steps, filled current square; adapts to vertical below 96 points per column. |
| `Vertical` | Leading 40-point squares, outlined current square, title and wrapped supporting description; variable-height rows with connectors. |
| `Compact` | 32-point numbered squares, outlined current square, horizontal scrolling, label/status tooltips and accessible descriptions. |
| `Progress` | A position percentage track and wrapped uppercase labels; narrow layouts reflow labels into fewer columns. |

All variants expose a visible summary and a named accessible step list. Each step
has a native label or optional button, its status and position, and current-step
metadata. Descriptions are visible vertically and accessible in every variant.

## Navigation and state

Indicators are read-only by default. `.navigable(true)` explicitly makes a step
a native button, reachable by Tab and activated by Enter, Space, or pointer.
It receives hover/pressed feedback and a keyboard focus cue. Compact keyboard
focus reveals an off-screen target. Current-step activation returns no request.
There is no automatic activation on focus or custom arrow-key selection model.

`.enabled(false)` on a step or stepper disables navigation without changing the
workflow's position. Disabled parent UIs are respected. Stable component/key IDs
preserve focus through sibling insertion, renamed labels, and variant changes.
The component does not steal focus when the host changes workflow state.
After replacing a panel, the host should explicitly focus its relevant field or
action; the showcase demonstrates this policy.

`StepperResponse` returns step responses in source order, the effective variant,
an optional typed request, a resolved **one-based** current position, and the
position percentage. Requests do not mutate the bound state. The host owns field
values, validation, allowed destinations, Back/Continue controls, asynchronous
work, completion, and any error/retry presentation. This linear model does not
represent skipped, branching, independently completed, or failed steps.

## Boundary states and progress

For a valid current key and multiple steps, progress is its zero-based index
divided by `step_count - 1`, multiplied by 100. This follows the web position
indicator: viewing the last step is 100% position, **not workflow completion**.
Only `Complete` means that all work is finished. Accessible progress text names
the position rather than claiming that a percentage of work is completed.

A single current step starts at 0%; explicit completion is 100%. An empty list
shows “No steps” and 0%, even if marked complete. An unknown current key shows
“Current step unavailable”, no current/completed steps, and 0%; it never silently
chooses a different destination. `NotStarted` shows all steps upcoming at 0%.
Percentages are finite for every boundary case.

## Reference decisions

Reviewed the source checkout at revision
`e91ca1d506e1c7a11b58a5d6857d82208229812a`:
`docs/01-design-system/components/stepper.md`,
`src/components/Stepper/index.tsx`, and `public/assets/js/stepper.js`.
The horizontal fill follows the live component; the documented vertical and
compact treatments use an outlined current square. Completed compact indicators
retain numbers. This port adds explicit safe boundary/completion states and
opt-in native navigation while preserving the source's host-owned progression.
It uses shared tokens and existing Progress/Tooltip primitives, with no copied
third-party implementation or additional dependencies.
