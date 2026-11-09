# DutchyAgent implementation plan

Status: implemented and locally qualified; unreleased, planned 1.0.30.
The initial offline milestone below is
tracked in [DS-099](issues/DS-099.md); see its
[qualification](verification-dutchy-agent.md). The live extension is DS-100.

## Initial offline milestone

1. A small Provider contract and deterministic Scripted implementation: timed
   Unicode-safe deltas, completion/usage, simulated authentication/rate-limit/
   network errors and cancellation. No networking or new dependency.
2. Host-owned conversations and per-conversation drafts/runs. Request IDs isolate
   stale results. Stop preserves partial text; Retry replaces the last failed or
   stopped reply without duplicating the user turn. Switching conversations
   preserves independent streams; deleting a conversation cancels its requests.
3. A shared-shell chat UI with keyboard-operable conversation navigation,
   Enter-to-send/Shift+Enter newline, IME protection, Stop, retry Alerts,
   first-token Skeleton, simulated usage and polite completion feedback.
4. Assistant Markdown uses stable message identity, native CodeBlock copy and
   line numbers. External links are reported locally and images disabled so the
   offline example has no network/navigation side effects.
5. Settings load process values over the current directory's `.env`, with
   `MODEL_NAME=gpt-6-luna` and `MODEL_EFFORT=low` defaults. Only key presence is
   retained; key values are never displayed, logged or used by the mock. The
   ignored local `.env` stays outside commits and crate archives.
6. Meaningful state/provider/config/keyboard/copy tests, both themes at 1x, 1.5x
   and 2x, narrow rendering, native inspection, all required checks and actual
   archive inspection. Commit/version bump/push remain separate release work.

## Live extension

1. Reuse the existing local credential with redacted Debug and a masked UI;
   retain it in memory only. Start in OpenAI when a key exists, otherwise Mock.
   Keep both as explicit settings; never substitute a mock for a live error.
2. Verify current model/API documentation. Send Responses requests to the fixed
   OpenAI HTTPS endpoint, with configured model/effort, system instruction and
   completed history, `store: false`, streaming choice and an output cap.
3. Reuse ehttp with streaming as a dev-dependency; add current serde_json for
   example request/response parsing. Keep the library dependency surface unchanged.
   Bound transport delivery, SSE frames, reply/body sizes and body-receive time.
4. Preserve host request IDs, source labels, cancellation and retry semantics.
   Report API usage/model distinctly from mock estimates; redact raw error bodies.
   Document cooperative cancellation and unsupported provider features.
5. Test fragmented SSE/Unicode, buffered JSON, truncation, errors, limits,
   real loopback HTTP, cancellation, routing and accessible provider selection.
   Native verification must include one real "Hi" with the configured model,
   explicit Mock selection and both themes; CI remains offline.
6. Run the complete feature, lint, rendering, docs, native, MSRV, WASM and
   actual-archive matrix. Record [live verification](verification-dutchy-agent-live.md)
   and resolve [DS-100](issues/DS-100.md). Release preparation stays separate.

Persistence, tools, attachments, custom endpoints and other providers remain deferred.

## Presentation follow-up (DS-101)

Keep one compact header: title/provider badge on the left, New chat and a named
settings icon on the right. At narrow widths, use a named plus icon for New chat.
Keep action tab order, tooltips, shortcuts and settings focus return. Left-align
conversation titles using Button's padded text alignment; its existing centered
default stays intact. Check long titles, loading alignment, both themes at
1x/1.5x/2x, 480/560/1160-point layouts, native rendering and all required gates.
See [shell verification](verification-dutchy-agent-shell.md).
