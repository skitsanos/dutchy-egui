# Form controls and validation

## Ownership and identity

Values stay in application state. Controls borrow `&mut String`, `&mut bool`, or
`&mut Option<T>`. IDs are stable salts within the parent UI. Field uses its label
as a namespace; supply `Field::id_salt` when labels repeat or can change. Options
use their labels as salts; use `SelectOption::id_salt` for repeated/changing labels.
Keep parent UI namespaces stable too.

Field returns the actual control Response, preserving changed, focus, and blur
events. Clicking an input's visible label focuses its control. Labels sit 8
points above their control, item spacing included.

```rust
use dutchy_egui::{Field, Input};
use dutchy_egui::validation::{FieldValidation, Rule, ValidationTrigger};

#[derive(Default)]
struct Form { email: String, email_check: FieldValidation }

fn show(ui: &mut egui::Ui, form: &mut Form) {
    let rules = [Rule::required("Email is required"), Rule::email("Enter a valid email")];
    let response = Field::new("Email")
        .required(true)
        .help("Use a work address")
        .validation(form.email_check.state())
        .show(ui, |ui| ui.add(Input::new("email", &mut form.email)));
    form.email_check.update(&response, &form.email, ValidationTrigger::OnBlur, &rules);
}
```

The required marker/metadata do not enforce rules themselves. Include a required
rule and explicitly validate every required field at submission.

## Controls

- Input: single-line, placeholder, password mask, shared sizes, disabled/read-only,
  width, and character limit. Password clipboard/accessibility behavior uses egui.
  The text row is vertically centered in the frame.
- [PathInput](#path-input): editable path with a separately focusable Browse button.
- Textarea: native multiline editing, rows, 120-point minimum height, count/limit,
  and read-only/disabled states. Interactive resize handles remain planned.
- Checkbox: square indicator, wrapped clickable label, keyboard activation.
- RadioGroup: typed exclusive values, square indicators, one Tab stop, arrows
  wrapping across enabled choices, Home/End, disabled options.
- Select: typed options, placeholder, disabled options, shared heights. Enter
  opens; Tab/arrows navigate; Enter chooses; Escape dismisses. Choosing a value
  returns focus to the trigger, including choosing the existing value.

Switch, Segmented, Slider, RangeSlider, NumberInput, DatePicker, TimePicker and
the Kbd chip follow the same ownership and ID rules; their decisions, keyboard
behavior and examples are in the [form primitives guide](form-primitives.md).

Use change or explicit submit validation for selection controls. RadioGroup
returns a structural group Response with change events; it does not aggregate
child focus/blur events. Select blur refers to its trigger, including focus
moving into the popup, rather than leaving the entire popup.

Read-only editors retain selection/copy. Disabled controls honor disabled parents
and cannot edit. Counts/limits use Unicode scalar values, rather than bytes,
UTF-16 units, or grapheme clusters; combining marks count separately. Limits do
not truncate host-supplied values: application validation handles those values.

Inside Dark/Primary cards and solid banners, field, checkbox, and radio labels
follow the surface's scoped text color. Inputs, textareas, and select triggers
keep their light fill with foreground text, as on the web.

```rust
use dutchy_egui::{Select, SelectOption};
let choices = [SelectOption::new("editor", "Editor"),
    SelectOption::new("owner", "Owner").enabled(false)];
let mut selected: Option<&str> = None;
// In a UI closure:
// ui.add(Select::new("role", &mut selected, &choices));
```

## Path input

`PathInput` binds a host-owned `String`. Browse emits `OpenFile`, `OpenFolder`
or `SaveFile` for the mode supplied at construction. Use `response` as the
editor's `Response` for `Field` and change/blur validation. Browse is a separate
Tab stop and moves below the editor once the editor would be narrower than the
button. Set `accessible_name` to the field's label: `Field` names the editor,
but Browse takes its name from PathInput and is otherwise "Browse for Path".

```rust
use dutchy_egui::{PathBrowseRequest, PathInput};

fn export_path(
    ui: &mut egui::Ui,
    path: &mut String,
    pick: impl FnOnce(PathBrowseRequest) -> Option<String>,
) {
    let result = PathInput::new("export", path, PathBrowseRequest::SaveFile)
        .accessible_name("Export destination")
        .placeholder("Choose where to save")
        .show(ui);
    if let Some(request) = result.request {
        if let Some(chosen) = pick(request) {
            *path = chosen;
        }
        result.browse.request_focus();
    }
}
```

The host opens the picker, handles cancellation, validates paths, and performs
any filesystem I/O. A typed path can be relative or incomplete. Read-only mode
retains text selection and copying and disables Browse. The Showcase's Path input
page uses `rfd` to choose a file, choose a folder, and select a save destination.

## Validation lifecycle

Built-ins: required, basic email shape, finite decimal number (including exponent
notation), minimum/maximum scalar length. `Rule::custom` accepts a borrowing
application predicate for URLs, patterns, cross-field checks, or domain policy.
No regex engine, URL parser, runtime, or transport is added to the library.
Non-required rules skip whitespace-only values. The first failure supplies its
message. Basic email shape does not verify delivery.

`FieldValidation::update` executes rules on configured events, rather than every
frame. OnSubmit waits for explicit `validate` calls. Edits clear old results and
invalidate pending tickets. `set_error` presents a server/domain error; `clear`
resets touched/result state and invalidates tickets. When changing a value
outside its widget, call `clear` too.

For asynchronous checks:

1. Run synchronous rules, call `begin_async`, and keep its ticket.
2. Execute work in the application's own runtime/service.
3. Call `finish_async(ticket, result)` on the same field state.
4. True accepts the result: request a repaint. False means the ticket is stale,
   reset, already consumed, or belongs to another field.

Pending feedback includes busy metadata. Errors expose named live alerts and
invalid/error-description metadata. Feedback never moves focus. Palette contrast
limitations remain recorded in DS-003 and DS-027; the error message follows
`palette.destructive`, which hosts may override before installing the theme.

The showcase availability check is a deterministic one-second demonstration:
`taken@…` is rejected, with no network request. Editing/resetting while pending
exercises ticket invalidation. The validation button checks all fields; Reset
resets application values and validation together.
