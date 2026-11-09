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
  dev-dependencies, as `rfd` already is for the showcase. Expected additions
  are `ehttp` with streaming (already an optional dependency) and
  `serde`/`serde_json` for chat request bodies; verify current versions when
  adding them. No async runtime: network callbacks post to a `std::sync::mpsc`
  channel and request a repaint.
- **Data.** Fictional and generated deterministically in code from a fixed
  seed: invented people and subjects, reserved domains (`example.com`,
  `example.org`), no real products, companies or people, and no bundled images
  or text corpora, because `examples/**` ships in the crate archive.
- **Secrets and network.** API keys come only from an environment variable
  named in the settings, are never persisted or logged and are shown masked.
  Requests go only to the user-configured base URL. A scripted mock provider is
  the default, so every application runs offline and in tests.
- **Shared shell.** `examples/support/` holds the theme switcher, sidebar
  frame, status strip and fictional-data helpers, included with `#[path]`.
  Once stable it informs an AppShell composition for the library.
- **Quality bar.** Fully keyboard-operable flows, polite announcements for
  asynchronous completion, light and dark modes at 1x, 1.5x and 2x, kittest
  interaction tests inside the example and a native inspection before release.
  Each application gets an Issues Ledger ticket recording what it proves and
  the gaps it surfaced, a `verification-*.md` record, and one README line with
  its run command.

## DutchyTodo

The smallest complete application and the tutorial companion to
[getting started](getting-started.md). Buildable with shipped components.

- **Scope.** Projects in a sidebar with task counts (Badge); a task list with
  Checkbox completion, title and notes editing in a Drawer (InlineEdit when it
  lands), DatePicker due dates, priority Badges, Segmented filter for all,
  active and done, SearchInput, completion Progress and Stats, EmptyState per
  filter, Toast with undo for delete and complete, Kbd hints, and a Modal
  confirmation when deleting a project (Dialog preset when it lands).
- **Keyboard.** N creates, Enter edits, Space toggles, Delete removes, Escape
  closes editors; focus returns to the list row.
- **State.** An in-memory keyed store owned by the application, with optional
  import and export through the existing `rfd` dialog.
- **Limits.** No synchronisation, recurring tasks, reminders or notifications.
- **As built** ([DS-077](issues/DS-077.md)). Rows are the keyboard anchor:
  Up/Down, Home and End move between rows, a pointer click focuses a row and a
  double-click edits it, while the Checkbox keeps its own activation. Import
  and export use a tab-separated line format instead of serde, and an import
  replaces the store with an undo toast rather than asking first. Kbd hints
  sit in the status strip and the task search has no global shortcut. The
  support module ships a theme switcher, header, sidebar and status-strip
  frames, a seeded generator with fictional people, and capture hooks
  (`DUTCHY_EXAMPLE_ZOOM`, `DUTCHY_EXAMPLE_MODE`, `DUTCHY_EXAMPLE_CAPTURE`) that
  render a native frame at a chosen scale and mode for verification records;
  the showcase keeps its own header until a later release adopts the shared
  one.

## DutchyAgent

A chat client for an OpenAI-compatible chat completions endpoint with a mock
provider. Buildable with shipped components; the flagship for Markdown and
CodeBlock.

- **Scope.** Conversation list in the sidebar; a transcript of user and
  assistant turns where assistant Markdown renders with CodeBlock copy;
  streamed tokens with a Stop button; a composer Textarea where Enter sends
  and Shift+Enter inserts a newline, with Kbd hints; a settings Drawer for base
  URL, model id, API key variable name (masked), temperature Slider, streaming
  Switch and system prompt; Alerts with retry for authentication, rate-limit
  and network errors; Stats for reported token usage; EmptyState for a new
  conversation; Skeleton until the first token; Toast on copy.
- **Provider contract.** A `Provider` trait with `OpenAiCompatible` (streaming
  server-sent events over `ehttp`) and a `Scripted` mock with deterministic
  replies, simulated latency and error scenarios for tests. Further adapters,
  such as Anthropic's Messages API, implement the same trait. The model id is
  configuration: the example ships a placeholder that must name a model the
  configured provider serves.
- **Limits.** No tool calling, attachments, image input, conversation
  persistence or multiple provider accounts.

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

## Sequence

1. Example scaffolding (`[[example]]` entries, shared shell, test hooks) and
   DutchyTodo.
2. DutchyAgent, including the dev-dependency additions and the mock provider.
3. DutchyMail together with the command-plumbing and workbench milestones.
4. DutchyDash together with the dashboards milestone.

Each lands in its own release with its ticket, verification record and
README line, and never pushes without authorization.
