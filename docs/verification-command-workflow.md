# Command workflow qualification

Date: 2026-11-09. Scope: 1.0.28, [DS-087](issues/DS-087.md) through
[DS-092](issues/DS-092.md), on `ticket/ds-087-command-workflow` over 1.0.27
(`0fd402e`). This record predates the release bump; the work ships in
[1.0.28](verification-release-1.0.28.md). No dependency changed.
The [plan](plan-1.0.28.md) and [API guide](commands-shortcuts.md) define the scope.
The [independent review](verification-review-1.0.28.md) later found and fixed
DS-093 through DS-097; its record holds the current gate results.

## Local checks

All 18 gates passed on macOS arm64, Rust/Cargo 1.99.1:

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 590 passed, including 36 Todo tests |
| `cargo test --no-default-features` | 399 passed |
| Clippy: default, all features, no default features | Each uses `cargo clippy --all-targets [feature flags] -- -D warnings`; pass |
| Rustdoc: all features, no dependencies | `RUSTDOCFLAGS='-D warnings'`; pass |
| WASM library: all features and no default features | Both pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| Rust 1.95.0 MSRV | Locked all-features library check passes |
| Native examples | All-features build passes |
| Actual crate archive | `cargo package --allow-dirty --locked --all-features` passes |
| Archive inspector | Approved paths, source bytes, licences and packaged relative links pass |
| CI workflow | actionlint 1.7.12 passes |
| Qualification script | `bash -n scripts/check.sh` passes |

The test totals include doc tests and example tests. No test failures or
ignored tests. Required checks ran after the final Rust changes. The archive
was rebuilt and inspected after the final documentation updates. The package
contains all three bundled font licence notices and excludes root media,
storyboard outputs and generated snapshot differences.

New interaction coverage includes exact event modifiers, platform aliases,
scope precedence, text/modal/popup policy, IME, ambiguity, rebinding, and no
duplicate dispatch/input leakage through discarded passes. Recorder tests
cover capture isolation, conflict feedback, cancellation, clearing and
read-only behavior. SplitButton tests cover independent actions, focus return,
keyboard opening, disabled/busy states and right-side arrow order in RTL hosts.
Nested discovery tests cover breadcrumb/leaf word matching, stable path keys,
disabled ancestors and non-executable branches.

## Rendering and native inspection

The control snapshots cover light 1x, dark 1.5x and light 2x. The nine richer
menu snapshots cover light Orange 1x, dark Orange 1.5x and light Blue 2x, with
root, nested and checked menus at each scale. Todo retains light 1x and dark
2x coverage. Fourteen intentional baseline additions/updates reflect the new
controls and integrations. Reviewed fills, joined borders, arrow spacing,
disabled/loading states, nested placement and trailing shortcut hints.

Native macOS checks passed in both light and dark modes:

- **Todo:** palette discovery opens New task; recorder Escape cancels without
  dismissing its drawer; an occupied search chord is rejected; rebinding New
  task updates hints and replaces the old chord. The split arrow opens file
  alternatives independently, and Tab exits to task search. Existing import
  safety tests remain green; native checks did not invoke a file operation.
- **Canal Studio:** keyboard, toolbar and menu Save share one host counter;
  rebinding updates menu/toolbar hints and replaces the old chord. Nested
  `file export png` discovery executes the PNG request. Checked commands obey
  the host's accept/decline switch. The split primary invokes export directly.

Both task-owned preview apps were closed and their temporary bundles removed.
Private logs, accessibility receipts and native screenshots remain under
`.git/codex-maintenance/commands-1.0.28/`; shared build caches, tracked baselines,
user media/IDE files and unrelated processes were preserved.

## CI and remaining limits

The workflow runs the qualification script on Ubuntu with software Vulkan,
plus library feature checks on macOS and Windows. The official
[checkout v7.0.1 release](https://github.com/actions/checkout/releases/tag/v7.0.1)
was verified and pinned by commit, with read-only repository permissions and
credential persistence disabled. No publishing step is included.

Hosted Linux/Windows CI has not run: an approved push or pull request is still
required. Local macOS checks and native inspection do not prove those hosted
results. The forthcoming version bump and release archive require final
qualification before delivery.

Bindings are single logical chords, session-local and scoped to this app;
there are no system hotkeys or OS menu integration. Focused scope targets one
exact widget ID. Custom host overlays must gate dispatch themselves. Nested
search uses case-insensitive substring words, without fuzzy scoring. Command
execution, persistence, Undo and acceptance of checked requests stay with the
host.
