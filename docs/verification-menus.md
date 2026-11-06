# Menus and dropdowns verification — 2026-11-06

Package: **dutchy-egui 1.0.7**, developed locally on `ticket/menus-dropdowns`.
The 1.0.6 accordion/tab release, including revised composite focus behavior, was
committed and pushed to main as `4840e1042894e596524bb355f173c5ba7c3c9894`.
Remote main was checked against that commit. This record was captured before the 1.0.7 release commit; subsequent release
history is in Git. Registry publication was not performed.

Host: macOS aarch64, Rust/Cargo 1.99.1. No dependencies were added or upgraded;
egui 0.37.0 remains the sole direct runtime dependency.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 119 integration tests and one doctest.
- `cargo test --no-default-features`: 86 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`: archive and extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.7/Cargo.toml --example showcase --locked`: independent packaged-example build.
- `git diff --check`; all 122 Rust/Python source files remain below 400 lines.

Menu tests verify grouped roles/names, shortcut descriptions without implicit key
bindings, stable IDs after reordering, pointer/assistive/Enter/Space actions,
Up/Down/Home/End navigation, disabled parents/items, narrow wrapping, and empties.
Dropdown tests cover first/last keyboard opening, disabled skipping, action/Escape
focus return, Tab/Shift+Tab exit, preserved outside-input focus, secondary click,
Shift+F10, disabled closure, and scrolling/flipping within a 240x180 viewport.
Context-menu Tab dismissal returns to its existing target; the next Tab exits.

Search tests separate query edits from commits, verify case-insensitive filters,
active-descendant/selection metadata, disabled options, no matches, Escape/Tab,
retained values, stable editor IDs after sibling insertion, and disabled editing.
Palette tests cover shortcut toggling, named modal ancestry, native input blocking,
Tab/Shift+Tab confinement, filtering/activation, no-match/disabled results, focus
return, and fitting a 280x220 viewport. The named dialog contains the editor/list;
its active descendant supplies semantic focus while the editor retains typing.
Showcase workflows use public APIs and confirm host actions, committed values,
and command shortcut handling from another page.

## Rendered evidence

Reviewed twelve WGPU baselines under `tests/snapshots/`:
`menus-{light-1x,dark-1_5x,purple-2x}-{inline,dropdown,search,commands}.png`.
They show groups, shortcut alignment, disabled/destructive actions, keyboard
highlights, flat bordered popups, editor focus, and command modal/backdrop.
Baselines pass without update mode. Search-result review removed a second hover
highlight under a stationary pointer so only the active candidate is emphasized.

Desktop and 480-point showcase previews are under `target/visual-proof/`:
`menus-desktop.png`, `menus-narrow.png`, `dropdowns-desktop.png`, and
`dropdowns-narrow.png`. Menu widths and result/dialog bounds are also asserted.

## Native evidence

The rebuilt Glow showcase ran through a temporary macOS app bundle. Project
Actions opened its grouped menu; Down/Enter chose Duplicate and incremented the
host action count, restoring trigger focus. Secondary click opened the context
menu at the target. Escape dismissed it.

Coordinate-based input into Search Frameworks filtered to Iced; Enter committed
it while preserving editable input. Cmd+K opened command search, typing Create
filtered results, Tab/Shift+Tab remained inside, and Enter returned Create,
incrementing the host count. Focus returned to Search Frameworks. Light/dark
command surfaces were inspected. Native accessibility review exposed an extra
empty dialog; the correction was rebuilt and rechecked: one named Command Palette
dialog now contains its search input and list. The global shortcut also opened
from Overview. Native Glow execution and WGPU rendering are separate evidence.

## Limits

Native execution is macOS only, without a running screen reader. Windows/Linux
GUI execution, browser WASM execution, and an actual Rust 1.95 minimum-toolchain
run remain outstanding. AccessKit checks and native tree inspection provide
narrower evidence than screen-reader acceptance.

The host owns state, query initialization, selection, action execution, shortcut
registration for hints, and focus if it removes a saved return target. Keep
calling the command palette while closed to finish deferred restoration.
Context menus use the documented two-step Tab exit for existing targets.
Nested submenus, checked/radio actions, menu bars, fuzzy ranking, asynchronous
search, virtualization, and general modal/drawer APIs remain outside this step.
Native popup placement can settle after a sizing pass. Source palette contrast
limitations remain in [DS-003](issues/DS-003.md); no whole-system parity is claimed.

The same 1.0.7 package includes the subsequent
[close-control feedback and spacing correction](verification-close-controls.md);
the test totals above include those regressions.
