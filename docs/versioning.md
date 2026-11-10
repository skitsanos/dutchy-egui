# Versioning policy

Dutchy stays on 1.x and follows Semantic Versioning as Cargo applies it (see
[SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)).
The promise starts with the first crates.io release, 1.1.0. The Git tags
`v1.0.29` and `v1.0.30` come before it, so 1.1.0 can break code written
against them; its changelog lists those changes under "Breaking changes" with a
migration note. A Git dependency stays on its tag, so no host is upgraded by
surprise.

## What each number means

| Change | Release |
| --- | --- |
| Bug fixes, documentation, internal changes, rendering corrections toward the reference | Patch (1.0.31) |
| New components, methods or features; new variants or fields on `#[non_exhaustive]` types; a higher MSRV | Minor (1.1.0) |
| Anything that can stop host code compiling or change what it means: removed or renamed items, changed signatures, a new variant on an exhaustive enum, a removed derive or feature, an egui upgrade | Major (2.0.0) |

Every push to main advances at least the patch number. This table says which
number goes up.

## egui

Dutchy re-exports egui as `dutchy_egui::egui`, and its widgets take and return
egui types. A new egui release changes those types, so upgrading to it is a
major release even when Dutchy's own API stays the same. egui ships a few
releases a year, so expect a Dutchy major about as often.

| Dutchy | egui | MSRV |
| --- | --- | --- |
| 1.x | 0.37 | 1.95 |

Hosts use `dutchy_egui::egui`, or depend on the same egui minor, so the types
match. Each major that upgrades egui adds a row. Only the newest major gets
fixes.

## Rust version

`rust-version` follows egui's, which is 1.95 today. Raising it without an egui
upgrade is a minor release and is noted in the changelog. Cargo's MSRV-aware
resolver, the default for edition 2024 workspaces, keeps hosts on older
compilers on the previous minor.

## Features

Adding a feature is a minor release. Removing a feature, or taking one out of
the defaults, is a major release. The current features are `bundled-fonts` and
`markdown` (on by default), `url-images` and `png-export`.

## Rendering and tokens

Dutchy is a design system, so pixels change between releases:

- A correction that brings a widget closer to the reference or fixes its layout,
  such as [DS-104](issues/DS-104.md), is a patch.
- A deliberate change to token values, default sizes or default variants is a
  minor release, with a "Visual" note in the changelog. It can move host
  layouts without stopping them compiling.
- Renaming or removing a token constant, or changing its type, is a major
  release.
- A change to default behavior that hosts rely on, such as focus return,
  dismissal or keyboard handling, is a major release unless it fixes a
  documented bug.

## Deprecation

An item to be removed gets `#[deprecated(since = "…", note = "use …")]` in a
minor release and is removed in the next major.

## Extensibility

Public enums, and structs whose fields are all public, are `#[non_exhaustive]`
so that new variants and fields are minor releases. Hosts build option structs
from `default()` or a constructor and chain setters, match marked enums with a
wildcard arm, and read response fields without building them. Structs with a
private field can already grow. The `ALL` lists are slices, so they can grow
too.

These types stay exhaustive:

| Type | Reason |
| --- | --- |
| `Mode` | Light or dark; a new mode would be a major design change anyway. |
| `TableSortDirection` | Hosts sort their own rows by matching it. |
| `Weekday`, `DateSegment`, `FlexDirection` | Complete sets; `FlexDirection` has all four CSS directions. |
| `ToolbarOrientation`, `TrendDirection`, `TrendSentiment` | Complete sets. |
| `AccordionState` | Single or multiple expansion, held and matched by hosts. |
| `TableSort`, `TableSelection` | Small pairs that hosts build; sorting by several columns would be a new type. |

`scripts/check_api.py` runs in the quality gate. It fails when a public enum, or
a struct with only public fields, is neither marked nor in this table, and when
an entry in this table is missing or marked.

Within a major:

- a field added to a `Copy` struct (`Theme`, `Palette`, `ScribbleExportOptions`,
  `UrlImageOptions`) must itself be `Copy`, because dropping `Copy` is a major
  change;
- the function-pointer aliases `IconPainter`, `MenuIconPainter` and
  `BreadcrumbIconPainter`, and the `CodeHighlighter` alias, keep their
  signatures;
- adding a builder method is a minor release, while changing an existing
  method's argument types, including `impl Into<…>` bounds, is a major one.

## Release mechanics

- Every release is tagged `vX.Y.Z` on its commit on main.
- A release with breaking changes starts its changelog section with "Breaking
  changes" and a short migration note.
- CI runs `cargo-semver-checks` with all features against the published
  crates.io 1.1.0 baseline. The baseline is explicit, not a floating latest
  version; advance it deliberately when adopting a newer compatibility floor.
  The check catches accidental breaks in Dutchy's own API, not egui upgrades,
  which remain a manual call. The local command is part of `scripts/check.sh`;
  see [development](development.md) for the tool installation.
- Publishing to crates.io is a separate, authorized step after the quality
  gate.
