# API extensibility before the first crates.io release

Status: approved 2026-11-10. This plan lists which public types become
`#[non_exhaustive]` under the [versioning policy](versioning.md). Its
implementation is [DS-105](issues/DS-105.md).

## Why

On 1.x, two kinds of addition count as major releases: a new variant on a
public enum, and a new field on a struct whose fields are all public, because
hosts can build such a struct as a literal. No public type carries the mark
today. Dutchy has 75 public enums and 128 public structs, 55 of them with
public fields. Marking the types expected to grow turns those additions into
minor releases. Adding the mark is itself a breaking change, so it belongs in
the first crates.io release.

What the mark changes for hosts:

- For an enum, a `match` needs a wildcard arm. Building variants is unchanged.
- For a struct, neither a struct literal nor `..Default::default()` works
  outside Dutchy. Reading fields, assigning them on an existing value and `..`
  patterns still work.

Inside Dutchy nothing changes. The rule applies only in other crates, and
`tests/` and `examples/` count as other crates.

## Enums: mark 66, keep 9

Mark:

- **Design variants and sizes**, which hosts pass and seldom match on, and
  which grow with the reference system (23): `AccordionVariant`, `AlertVariant`,
  `BadgeVariant`, `BreadcrumbSeparator`, `ButtonVariant`, `CardVariant`,
  `EmptyStateIcon`, `EmptyStateVariant`, `FileUploadVariant`, `ImageAspect`,
  `MenuItemVariant`, `PaginationVariant`, `ProgressVariant`, `SkeletonVariant`,
  `SpinnerSize`, `StatSize`, `StatVariant`, `StatsLayout`, `StepperVariant`,
  `TableDensity`, `TableHeader`, `TabsVariant`, `ToastAppearance`.
- **Token scales and sets** (17): `Accent`, `CardPadding`, `ColumnWidth`,
  `ContainerWidth`, `ControlSize`, `DrawerWidth`, `Family`, `Icon`, `Leading`,
  `MenuWidth`, `ModalWidth`, `ProgressSize`, `Spacing`, `TextColor`, `TextSize`,
  `Tracking`, `Weight`.
- **Placement and behavior options** (14): `CommandScope`, `CrossAlign` (CSS
  also has baseline), `DrawerSide` (top and bottom sheets), `DropdownAlign`,
  `ImageFit`, `LinkPolicy`, `ScribbleSmoothing`, `ScribbleWidth`, `TableAlign`,
  `TableColumnWidth`, `TimeSegment` (a 12-hour picker adds a period segment),
  `ToastPosition` (centered positions), `TooltipPosition` (automatic
  placement), `ValidationTrigger`.
- **Events and outcomes that hosts match on** (6): `MenuEvent`, `OverlayDismissal`,
  `ScribbleAction`, `ToastDismissal`, `ToastEvent`, `ToolbarEvent`.
- **States and errors** (6): `FileRejectionReason`, `FileStatus`, `ImageStatus`,
  `ScribbleError`, `StepperState`, `ValidationState`.

Keep exhaustive:

| Enum | Reason |
| --- | --- |
| `Mode` | Light or dark. Examples and tests match on it in eight files; a new mode would be a major design change anyway. |
| `TableSortDirection` | Hosts sort their own rows by matching it. |
| `Weekday`, `DateSegment`, `FlexDirection` | Complete sets (`FlexDirection` has all four CSS directions). |
| `ToolbarOrientation`, `TrendDirection`, `TrendSentiment` | Complete sets. |
| `AccordionState` | Single or multiple expansion: state that hosts hold and may match on. |

Eight marked enums expose `pub const ALL: [Self; N]`: `Icon`, `Accent`,
`AlertVariant`, `BadgeVariant`, `ButtonVariant`, `CardVariant`,
`ProgressVariant` and `StatVariant`. A new variant changes `N`, which is a
major change, so these become `&'static [Self]`. A `for x in T::ALL` loop then
yields `&T`. The 27 uses in examples and tests need `.iter().copied()` or a
dereference. `Weekday` keeps its array.

## Structs: mark 43, keep 2

| Group | Count | Proposal | Effect on this repository |
| --- | --- | --- | --- |
| `*Response` structs | 27 | Mark | None. Hosts only read them, and no code here builds them or destructures them without `..`. |
| Reports built by Dutchy: `CommandRequest`, `ShortcutConflict`, `MarkdownLink`, `FileRejection`, `FileValidation` | 5 | Mark | `tests/markdown.rs` builds one expected `MarkdownLink`; it compares fields instead. |
| Types hosts build through existing constructors: `Command`, `GalleryItem`, `FileItem`, `FileInfo`, `Theme`, `Palette` | 6 | Mark | None. A custom palette starts from `Palette::new` and assigns the fields it changes. |
| Types hosts build as literals: `OverlayOptions`, `UrlImageOptions`, `ScribbleExportOptions`, `SearchState`, `CodeSpan` | 5 | Mark and add constructors or setters | 23 literal sites (21 in examples and tests, 2 in guides) move to setters. |
| Small fixed pairs: `TableSort`, `TableSelection` | 2 | Keep | Hosts and tests build them. Sorting by several columns would be a new type. |
| Structs that already have a private field: `AccordionItem`, `BreadcrumbItem`, `MenuBarItem`, `MenuItem`, `MenuResponse`, `SelectOption`, `Step`, `Tab`, `TableColumn`, `ToolbarItem` | 10 | No change | Hosts already can't build them as literals. |
| Builders without public fields: `Button`, `Modal`, `DataTable` and others | 73 | No change | Adding a private field is already a minor change. |

New API for the literal group:

- `OverlayOptions`: one chainable setter per field (`dismiss_on_escape`,
  `dismiss_on_backdrop`, `show_close_button`, `initial_focus`, `return_focus`).
  The common `OverlayOptions { return_focus, ..Default::default() }` becomes
  `OverlayOptions::default().return_focus(return_focus)`.
- `UrlImageOptions`: one setter per limit (`max_download_bytes`, `max_pixels`,
  `max_entries`, `max_concurrent`, `response_body_timeout`).
- `ScribbleExportOptions`: `new(scale)` plus a `background` setter.
- `CodeSpan`: `CodeSpan::new(range, color)`. Without it, a host highlighter
  could not build spans at all.
- `SearchState`: no new API. Hosts assign `open` and `query` on a default
  value.

Sites to change: `OverlayOptions` in the DutchyAgent settings, DutchyTodo
commands and editor, three showcase pages, `tests/overlays.rs`,
`tests/url_media_rendering.rs` and the images guide. `UrlImageOptions` in
`tests/url_images.rs` and the URL images guide. `ScribbleExportOptions` in the
showcase sketch page. `SearchState` in `tests/menu_rendering.rs` and
`tests/nested_command_search.rs`.

Matches that may need a wildcard arm: `Accent` in `examples/support/theme.rs`,
`ToolbarEvent` on two showcase pages, `ScribbleAction` on the sketch page,
`StepperState` on the stepper page, and `ToastPosition` and `TooltipPosition`
in their tests.

## Other public surface

- The 287 public constants, the free functions and the method signatures keep
  the policy's rules. They are not listed one by one here.
- The four function and closure aliases stay frozen within a major.
- No public traits exist, so there are no required methods to worry about.

## Delivery

1. Record the work as the next ledger entry (DS-105) on its own branch.
2. Mark the types, convert the `ALL` constants to slices, add the setters and
   `CodeSpan::new`, and update the examples, tests and guides.
3. Add a check to `scripts/check.sh` that lists every public enum and struct
   and fails on one that is neither marked nor on this plan's keep list, so a
   new type can't slip in unmarked.
4. Run the full gates. Rendering must not change, so any changed snapshot
   counts as a fault.
5. Release as 1.1.0, with "Breaking changes" and a migration note in the
   changelog. Publishing is a separate, authorized step: README install text,
   docs.rs metadata (`all-features = true`), `cargo publish --dry-run`, then the
   publish itself from the user's account.

## Limits of this review

The inventory comes from a source scan of every `pub enum` and `pub struct`
reachable through public modules or re-exports. It is not rustdoc's own view
of the API: that needs nightly rustdoc JSON, and no nightly toolchain is
installed. The match and literal sites come from text searches of `examples/`,
`tests/` and `docs/`. The implementation's compiler errors are the final list.
