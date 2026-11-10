# Versioning policy

Status: approved 2026-11-10. The version rule in AGENTS.md points here.

Dutchy stays on 1.x and follows Semantic Versioning as Cargo applies it (see
[SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)).
The promise starts with the first crates.io release, 1.1.0. The Git tags
`v1.0.29` and `v1.0.30` come before it. A Git dependency stays on its tag, so
no host is upgraded by surprise. 1.1.0 breaks some host code written against
those tags, and its changelog lists the changes under "Breaking changes".

## What each number means

| Change | Release |
| --- | --- |
| Bug fixes, documentation, internal changes, rendering corrections toward the reference | Patch (1.0.31) |
| New components, methods or features; new variants or fields on `#[non_exhaustive]` types; a higher MSRV | Minor (1.1.0) |
| Anything that can stop host code compiling or change what it means: removed or renamed items, changed signatures, a new variant on an exhaustive enum, a removed derive or feature, an egui upgrade | Major (2.0.0) |

The AGENTS.md rule that every push advances at least a patch stays. This table
says which number goes up.

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

Public enums and structs that are expected to grow are marked
`#[non_exhaustive]`, so additions to them are minor releases. The
[API extensibility plan](plan-api-extensibility.md) lists every type and what
the change means for examples and tests. Within a major:

- a field added to a `Copy` struct (`Theme`, `Palette`, `ScribbleExportOptions`,
  `UrlImageOptions`) must itself be `Copy`, because dropping `Copy` is a major
  change;
- the function-pointer aliases `IconPainter`, `MenuIconPainter` and
  `BreadcrumbIconPainter`, and the `CodeHighlighter` alias, keep their
  signatures;
- adding a builder method is a minor release, while changing an existing
  method's argument types, including `impl Into<…>` bounds, is a major one.

## Release mechanics

- Every release is tagged `vX.Y.Z` on its commit on main, as since 1.0.29.
- A release with breaking changes starts its changelog section with "Breaking
  changes" and a short migration note.
- After the first crates.io release, CI runs `cargo-semver-checks` (0.51.0
  today) with all features against the newest published version. It catches
  accidental breaks in Dutchy's own API. It does not flag egui upgrades, which
  remain a manual call. The exact command is to be checked when the step is
  added.
- `cargo publish` runs only with the user's authorization, after the usual
  qualification gates.

## Decisions

Approved on 2026-11-10:

1. The first crates.io release is 1.1.0, not 2.0.0. The extensibility pass
   breaks some host code on the Git tags: literal option structs, exhaustive
   matches and loops over `ALL`. Those tags come before the promise, so a
   migration note covers them instead of a new major number.
2. Only the newest major gets fixes.
3. Deliberate token changes are minor releases.
4. The AGENTS.md version rule points at this document.
