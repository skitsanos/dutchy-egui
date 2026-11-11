# Example applications

The showcase is the component gallery. Scenario applications under `examples/`
prove that the library composes under realistic, application-owned state,
show hosts how to structure that state, and surface the gaps that feed the
[component roadmap](component-roadmap.md). Each application is small enough
to read in one sitting and complete enough to feel like a product.

## Shared rules

- **Packaging.** Each application is a Cargo example in its own directory
  (`examples/dutchy_todo/main.rs` and modules) with a `[[example]]` entry that
  sets `path`, `required-features` where the example needs `markdown`, and
  `test = true` so `cargo test` runs the unit and `egui_kittest` tests kept
  inside the example. Snake-case names give the commands
  `cargo run --example dutchy_todo --all-features` and so on. Files follow the
  400-line soft cap and the cognitive-complexity threshold.
- **Dependencies.** No new runtime dependency. Example-only crates are
  dev-dependencies, as `rfd` already is for the showcase. DutchyAgent reuses
  `ehttp` with streaming and adds `serde_json` for request/response bodies.
  Verify current versions when adding or upgrading. No async runtime: network
  callbacks post to a bounded `std::sync::mpsc` channel and request a repaint.
- **Data.** Fictional and generated deterministically in code from a fixed
  seed: invented people and subjects, reserved domains (`example.com`,
  `example.org`), no real products, companies or people, and no bundled images
  or text corpora: the examples are public.
- **Secrets and network.** API keys come from process configuration or the
  gitignored local `.env`, are never saved by the application or logged, and
  are shown masked. DutchyAgent retains a redacted in-memory credential for
  the fixed OpenAI Responses HTTPS endpoint. It starts in OpenAI when a key
  exists, otherwise Offline mock; both are explicit choices. Tests use injected
  configuration and local fixtures, never machine credentials or external APIs.
- **Shared shell.** `examples/support/` holds the theme switcher, sidebar
  frame, status strip and fictional-data helpers, included with `#[path]`.
  Once stable it informs an AppShell composition for the library.
- **Quality bar.** Fully keyboard-operable flows, polite announcements for
  asynchronous completion, light and dark modes at 1x, 1.5x and 2x, kittest
  interaction tests inside the example and a native inspection before release.
  Each application gets an Issues Ledger ticket recording what it proves and
  the gaps it surfaced, and one README line with its run command.

## DutchyTodo

The smallest complete application and the tutorial companion to
[getting started](getting-started.md).

- **Scope.** Projects in a sidebar with task counts (Badge); a task list with
  Checkbox completion, title and notes editing in a Drawer (InlineEdit when it
  lands), DatePicker due dates, priority Badges, Segmented filter for all,
  active and done, SearchInput, completion Progress and Stats, EmptyState per
  filter, Toast with undo for delete and complete, Kbd hints, and a Modal
  confirmation when deleting a project (Dialog preset when it lands).
- **Keyboard.** N creates by default, Enter edits, Space toggles, Delete removes,
  Escape closes editors; focus returns to the list row. Command/Control+F
  searches, Command/Control+K opens command discovery and a Shortcuts drawer
  rebinds application commands.
- **State.** An in-memory keyed store owned by the application, with optional
  import and export through the existing `rfd` dialog.
- **Limits.** No synchronisation, recurring tasks, reminders or notifications.
- **As built** ([DS-077](issues/DS-077.md)). Rows are the keyboard anchor:
  Up/Down, Home and End move between rows, a pointer click focuses a row and a
  double-click edits it, while the Checkbox keeps its own activation. The
  viewport reveals the focused task when focus arrives or the task moves after
  sorting. The focus outline fits inside the row. Import
  and export use a tab-separated line format instead of serde, and an import
  replaces the store with an undo toast rather than asking first. Replacing
  the store (also through undo-import) discards older Undo actions and closes
  editors bound to the old records. Project and task IDs range from zero to
  `u64::MAX - 1`; the maximum value is reserved for exhaustion. Import errors
  leave the current list intact, and exhausted creation preserves drafts with
  an error message. Kbd hints sit in the status strip. Commands share live
  hints, availability and bindings across the header, palette, sidebar and New
  task split action; file operations keep their busy gate. The support module ships a theme switcher, header, sidebar and status-strip
  frames, a seeded generator with fictional people, and capture hooks
  (`DUTCHY_EXAMPLE_ZOOM`, `DUTCHY_EXAMPLE_MODE`, `DUTCHY_EXAMPLE_CAPTURE`) that
  render a native frame at a chosen scale and mode for inspection;
  the showcase keeps its own header until a later release adopts the shared
  one.

## DutchyAgent

[DutchyAgent](dutchy-agent.md) connects OpenAI Responses and an explicit
offline mock to the Markdown and CodeBlock composition.

- **Scope.** Conversation list in the sidebar; a transcript of user and
  assistant turns where assistant Markdown renders with CodeBlock copy;
  streamed tokens with a Stop button; a composer Textarea where Enter sends
  and Shift+Enter inserts a newline, with Kbd hints; a settings Drawer for
  provider, model id, reasoning effort, masked key presence, streaming Switch
  and system prompt; Alerts with retry for authentication, rate-limit
  and network errors; reported token usage; EmptyState for a new
  conversation; Skeleton until the first token; Toast on copy.
- **Provider contract.** An application-owned Router shares request, delta,
  completion, failure and cancellation messages between OpenAI and Scripted.
  OpenAI uses Responses SSE or buffered JSON over `ehttp`; Scripted uses a
  deterministic clock and simulated errors. No public provider trait is added
  to the library. Model settings load from process values over the ignored
  `.env`: `MODEL_NAME` (default `gpt-6-luna`) and `MODEL_EFFORT` (default
  `low`). Live errors never trigger a scripted fallback.
- **Limits.** No tool calling, attachments, image input, conversation
  persistence, temperature control, custom endpoints or multiple provider accounts.

## DutchyMail

A three-pane mail client over a fictional in-memory mailbox. It replaces the
generic file and record explorer as the demonstration of the command-plumbing
and workbench milestones, so it follows them.

- **Scope.** A folder TreeView with unread Badges; a virtualized message List
  with selection, unread weight and thousands of generated messages; a
  SplitPane between list and reading pane; Markdown bodies with Avatar
  initials; Toolbar, MenuBar and context Dropdown sharing one command tree
  (DS-073); the shortcut registry binding archive, reply, compose and search
  with matching Kbd hints; SearchInput with fuzzy matching; a compose Modal
  with Field, Input and Textarea and a discard confirmation; Toast undo for
  archive and delete; StatusBar counts and simulated sync state; EmptyState for
  empty folders and no selection; Skeleton for simulated loading; Sidebar.
- **Limits.** No network, mail protocols, HTML mail, attachment contents or
  persistence.

## DutchyDash

An operations dashboard of fictional service metrics, demonstrating the
dashboards milestone, so it follows the charts family.

- **Scope.** Stats grid with trends; sparkline, bar, donut, gauge and calendar
  heatmap charts; a DataTable of services and incidents with sort and filter
  through Segmented controls and Chips; a time-range Segmented control; Alerts;
  BackgroundTasks for simulated jobs; Skeleton loading; an auto-refresh Switch
  that respects reduced motion; a details Drawer.
- **Limits.** No live data sources.

## Later candidates

DutchyNotes (Markdown notes with a preview split, folder tree and fuzzy
search), DutchyBoard (kanban columns with reorderable cards), an onboarding
wizard with Stepper and PinInput, and DutchyLogs over the LogViewer candidate.
Preferences live inside each application as a settings Drawer, adopting
ThemePicker, KeyRecorder and PathInput as they land, rather than as a separate
application.

## Order

1. Workbench releases: editing helpers next to [List](lists.md),
   [TreeView](tree-view.md), [SplitPane](split-pane.md) and
   [InlineEdit](inline-edit.md); DutchyMail composes them once available.
2. DutchyDash together with the dashboards milestone.

Each lands in its own release with its ledger ticket and README line.
