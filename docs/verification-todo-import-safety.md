# DutchyTodo import safety

Date: 2026-11-09. Follow-up to 1.0.26 (`2feb3bf`), tracked in
[DS-085](issues/DS-085.md) and [DS-086](issues/DS-086.md).
This record covers the initial fix qualification on
`ticket/ds-085-todo-import-safety`, before the version bump from 1.0.26.
The fixes ship in [1.0.27](verification-release-1.0.27.md); that record covers
the final release tree. No dependencies changed.

## Plan and decisions

1. Bind each Undo request to an opaque store-history identity. Replacing the
   store on import or undo-import invalidates old requests, clears old
   notifications and closes editors/confirmations tied to the old IDs.
   A returned event is checked again when consumed; restoring an earlier store
   does not revive its old actions. The current import still offers Undo.
2. Use checked monotonic counters for projects and tasks. IDs `0..=u64::MAX - 1`
   are supported; `u64::MAX` is the exhausted counter state. Reject maximum
   record IDs with their input line number. Allocation failure leaves records
   and drafts intact. Show project errors as toasts and task errors inside the
   open drawer, where they remain visible.
3. Complete failed/disconnected file jobs with an error and clear the busy
   state. Catch unwinding worker panics, send the error and request repaint.
   Parsing failure leaves the current store and its valid Undo history intact.
   No change to the text exchange header or normal export format.

## Qualification

Nine regressions cover stale completion/deletion/project Undo,
two successive imports, undo-import, failed import, maximum/near-maximum IDs,
draft retention, and disconnected/panicking worker recovery. The original
maximum-ID and disconnected-channel regressions were run against the old code
and failed before the fixes.

All 14 gates pass on the final Rust source:

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 563 passed, including 33 Todo tests |
| `cargo test --no-default-features` | 379 passed |
| Clippy: default, all features, no default features | All targets, `-D warnings`, pass |
| Rustdoc: all features, no dependencies | `RUSTDOCFLAGS="-D warnings"`, pass |
| WASM library: all features and no default features | Pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| Rust 1.95.0 MSRV | Locked all-features library check passes |
| Native examples | `cargo build --examples --all-features` passes |

Clippy commands use `cargo clippy --all-targets [feature flags] -- -D warnings`.
No test failures or ignores. The light Orange 1x and dark Blue 2x Todo
baselines pass unchanged and were visually reviewed. No tracked snapshot,
font, font licence or library source changed; all changed Rust files remain
under 400 lines.

A standalone probe compiles the actual store and format source with
`rustc -C opt-level=3 -C overflow-checks=no`. Maximum-ID imports return errors;
the last valid project/task allocation succeeds; further allocations fail
without mutation; removal/restoration and export/import still work at capacity.
This checks optimized boundary arithmetic, not a full release-mode GUI build.

## Native macOS

The fixed example was driven through the real `rfd` Open dialog on a Retina
display, using disposable text fixtures in both light and dark mode:

- Completion offered Undo. Import replaced the records, showing only the new
  import notification. Undo-import restored the original completed task and
  cleared notifications. Automated stale-event tests also check requests
  already returned by the UI, beyond what the native toast timing can prove.
- A maximum project ID reported `Line 2`; a maximum task ID reported `Line 3`.
  The original task list survived both failures, Import/Export stayed enabled,
  and a subsequent valid near-maximum import succeeded.
- Exhausted project creation kept the sidebar draft. Exhausted task creation
  kept the drawer open with its title draft and a visible, accessible error.
  Scrolling kept footer actions reachable; Cancel closed the drawer and the
  original task remained. Native text values were set through accessibility.

The task-owned preview was closed after checking. Screenshots, test logs and
probe receipts are private under `.git/codex-maintenance/todo-import-safety/`.

## Package and cleanup

`cargo package --allow-dirty --locked --all-features` builds the actual crate
archive in an isolated target directory. All 691 archive files match the
approved source paths, with Cargo's generated metadata checked separately.
Media, storyboard work, IDE files and snapshot differences are excluded;
font licences and third-party notices are included. Relative documentation
links pass, and `Cargo.toml`/`Cargo.lock` match the released baseline.

The closed task preview bundle and compiled boundary probe were removed.
Private evidence, shared build caches, tracked assets/snapshots, ignored
user-owned work and the active branch were preserved. This initial fix task
did not push a release; release cleanup is recorded separately.

## Limits

The example remains an in-memory desktop application, without persistence or
synchronisation. Monotonic counters do not recycle deleted IDs. No new widget
or runtime dependency. Native platform qualification is macOS only; automated
accessibility checks do not replace a screen-reader session. Worker recovery
handles unwinding panics, not a process built with `panic=abort`.
