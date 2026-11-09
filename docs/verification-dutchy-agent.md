# DutchyAgent offline qualification

Released in 1.0.30 with review fixes; see [DS-103](issues/DS-103.md) and the
[release qualification](verification-release-1.0.30.md).

Historical record of the initial offline implementation. The
[live extension](verification-dutchy-agent-live.md) supersedes the provider and
credential-retention limits below; this record preserves the earlier evidence.

Date: 2026-11-09. [DS-099](issues/DS-099.md), on
`ticket/ds-099-dutchy-agent` over 1.0.29 (`3a92168`). The
[plan](plan-dutchy-agent.md) and [guide](dutchy-agent.md) describe the supported
scope. The work is uncommitted and unpushed; Cargo version/lockfile remain
1.0.29 until release preparation. No dependency or library API changed.

## Local gates

All 21 gates pass on macOS arm64, Rust/Cargo 1.99.1:

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 616 passed, including 20 DutchyAgent and 36 Todo tests |
| `cargo test --no-default-features` | 405 passed |
| Clippy: default, all features, minimal | All targets, `-D warnings`; all pass |
| Strict rustdoc | All features, no dependencies, `RUSTDOCFLAGS='-D warnings'`; pass |
| WASM library | All-feature and minimal profiles pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| MSRV 1.95.0 | Locked all-feature library check passes |
| Native examples | All-feature build passes |
| Workflow and shell | actionlint, ShellCheck and shell syntax pass |
| Relative documentation links and whitespace | Pass |
| Actual crate build and inspector | Approved paths, matching bytes, notices and packaged links pass |

Clippy runs as `cargo clippy --all-targets [feature flags] -- -D warnings`.
No failures or ignored tests. The required checks ran after the final Rust
changes; source and intended-baseline hashes are retained privately. The
archive is rebuilt and inspected after final documentation updates.

The 20 example tests cover deterministic streaming/buffering, Unicode chunks,
cancellation, stale events, independent conversations and drafts, Stop/Retry,
deletion/selection repair, request-setting snapshots, config precedence and
redaction, Enter/Shift+Enter, IME composition across frames, code copy, local
link reporting, settings and compact layout.

## Rendering and native evidence

Nine new baselines cover completed replies in light/dark at 1x, 1.5x and 2x,
plus narrow waiting, error and settings surfaces. Existing baselines are
unchanged. Accessibility checks run with every example UI/rendering case.

A geometry regression failed before the message-sizing fix: the last code
glyph ended at y=510 while the nested clip ended at y=497. Natural-height
message content now lets the outer transcript scroll; short code stays fully
visible and long code retains CodeBlock's own height cap. The regression,
updated rendering and native inspection pass. No CodeBlock library change was
needed.

Native macOS checks at Retina 2x passed in both modes:

- Plain Enter sent a message; the first-token Skeleton and running state
  appeared. Markdown rendered headings, lists, tables and a complete four-line
  code sample. Copy changed to Copied and produced a clipboard Toast.
- Command+period stopped a reply. Retry resumed it with one user turn.
- Settings displayed the requested model/effort defaults and a masked key
  presence indicator. A simulated authentication failure showed an Error Alert
  with Retry; switching the scenario to success recovered the same reply.
- Command+N opened another conversation while the earlier reply continued in
  the background. Shift+Enter inserted a newline without sending, and the
  draft survived switching away and back. The earlier reply completed.

The task-owned preview was closed and its temporary bundle removed. Native
receipts and qualification logs are under `.git/codex-maintenance/dutchy-agent/`.
Native window resizing and screen-reader output were not checked; narrow
layouts use kittest evidence.

## Environment and remaining limits

The local `.env` is ignored, never committed and never packaged. An ignored,
non-secret nested sentinel verified that Cargo's explicit includes cannot
accidentally bundle env files; actual archive inspection also rejects them.
Only credential presence reaches application state. Tests use injected config
sources, independent of developer or CI credentials. No plaintext key was
printed or inspected during verification.

All provider behavior is scripted; no HTTP client is instantiated, Markdown
images are disabled and links are reported locally. Model/effort defaults are
configuration values, not a claim of live endpoint compatibility. Token counts
are labelled mock estimates. Live API transport, SSE parsing, persistence and
real usage remain later work. Hosted CI for this tree awaits an authorized push;
current evidence is local qualification and native macOS inspection.
