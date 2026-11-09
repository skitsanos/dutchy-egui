# DutchyAgent live provider qualification

Released in 1.0.30 with review fixes; see [DS-103](issues/DS-103.md) and the
[release qualification](verification-release-1.0.30.md).

This records DS-100 before the [header/sidebar follow-up](verification-dutchy-agent-shell.md)
(DS-101). Its live-provider evidence still applies; the follow-up records the
current presentation and test counts.

Date: 2026-11-09. [DS-100](issues/DS-100.md), on
`ticket/ds-099-dutchy-agent` over 1.0.29 (`3a92168`). The work is uncommitted
and unpushed. Package version remains 1.0.29; planned release is 1.0.30.
The [guide](dutchy-agent.md) and [plan](plan-dutchy-agent.md) describe scope.

## Native live evidence

The existing ignored `.env` key was reused under explicit user authorization.
The native app selected OpenAI automatically, using `gpt-6-luna`, low effort
and streaming. One actual **Hi** request returned **Hi! How can I help?**
The UI reported **22 input / 11 output tokens**, returned model `gpt-6-luna`,
zero running requests and OpenAI provenance. This was a real Responses request,
not the scripted provider or a protocol fixture.

Settings showed only a masked key indicator. Explicitly selecting Offline mock
and sending Hi in another conversation produced the labelled scripted Rust
example. Returning to the original turn preserved its OpenAI source and actual
usage. The live turn rendered in both light/dark modes on macOS Retina 2x.
The task-owned preview was closed; unrelated processes were untouched.

Live buffered requests and immediate server-side cancellation were not exercised.
Local HTTP tests cover buffering, cancellation and failure behavior. Native window
resizing and screen-reader output were not checked; narrow rendering and accessible
interaction have kittest coverage.

## Protocol, dependencies and limits

Current [model documentation](https://developers.openai.com/api/docs/models/gpt-6-luna)
and [Responses streaming documentation](https://developers.openai.com/api/docs/guides/streaming-responses)
were checked before implementation. The configured model/effort are preserved.
Requests use the fixed OpenAI HTTPS endpoint, system instructions and completed
history, `store: false`, an output cap and the selected streaming mode.

Latest stable registry versions were verified: ehttp 0.7.3 is reused as a
streaming dev-dependency; serde_json 1.0.154 is added only for examples/tests.
The lockfile adds serde_json and its zmij dependency. Library APIs and runtime
dependencies are unchanged.

The decoder handles fragmented UTF-8/CRLF/multiline SSE, public text and refusal
deltas, terminal usage and buffered JSON. Tests reject malformed/truncated,
incomplete and oversized replies; raw server error bodies never reach the UI.
Usage provenance, no-key errors, explicit Mock retry and stable source labels
are covered. Loopback HTTP tests exercise actual POST/authorization/JSON headers,
settings, both response modes, HTTP errors and cancellation without external APIs.

Delivery uses a bounded channel; request/event/output sizes are capped at 2 MiB,
with a 90-second response-body timeout; connection setup uses transport defaults.
DS-103 later replaced that total body timeout with a 5-minute idle watchdog.
Stop prevents local delivery immediately; transport aborts at its next callback
or transport timeout and does not promise immediate server
cancellation or billing termination. There is no automatic history compaction.
`store: false` does not imply zero data retention. Tools, attachments, persistence,
custom endpoints and other providers are outside this example's scope.

The credential remains only in memory, with redacted Debug. Tests inject config
and fixture credentials rather than reading `.env`. Secret files remain ignored
and excluded from crate archives; the original nested-sentinel check is retained
with the [offline record](verification-dutchy-agent.md).

## Local checks

All 21 local gates pass on macOS arm64 with Rust/Cargo 1.99.1:

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 628 passed, including 32 Agent and 36 Todo tests |
| `cargo test --no-default-features` | 405 passed |
| Clippy: default, all features, minimal | All targets, `-D warnings`; all pass |
| Strict rustdoc | All features, `RUSTDOCFLAGS='-D warnings'`; pass |
| WASM library | All-feature and minimal profiles pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| MSRV 1.95.0 | Locked all-feature library check passes |
| Native examples | All-feature build passes |
| Workflow and shell | actionlint, ShellCheck and shell syntax pass |
| Relative documentation links and whitespace | Pass |
| Actual crate build and inspector | 773 files; approved paths, matching bytes, notices and packaged links pass |

No failed or ignored tests. The required Clippy commands use
`cargo clippy --all-targets [feature flags] -- -D warnings`. The existing CI
script picks up the new example through Cargo's `test = true`; no network key
or new workflow step is needed. Hosted CI for this tree awaits an authorized push.

Nine intended Agent baselines cover completed Markdown/code in light/dark at
1x, 1.5x and 2x, plus narrow waiting, error and settings surfaces. They were
updated for explicit provider/source labels; existing component baselines are
unchanged. Accessibility checks run with the example interaction/rendering cases.
Native evidence is separate from these offscreen checks.

Source, configuration, scripts and intended-baseline hashes remained unchanged
through qualification. The archive is rebuilt and inspected after final docs
updates. Gate logs, hashes and the native Hi receipt are retained privately in
`.git/codex-maintenance/dutchy-agent-live/`. No commit, version bump or push was made.
