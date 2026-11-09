# DutchyAgent

A native chat example for Dutchy Markdown and CodeBlock, with OpenAI and an
explicit offline mock. Run from the checkout root:

```sh
cargo run --example dutchy_agent --all-features
```

The example requires `bundled-fonts` and `markdown`. Its HTTP and JSON crates
are dev-dependencies; the library gains no runtime dependency or API. All
conversation state, drafts, requests and settings belong to the application.
Nothing is saved between launches.

## Configuration and providers

The gitignored `.env` in the current working directory can contain:

```dotenv
OPENAI_API_KEY=
MODEL_NAME=gpt-6-luna
MODEL_EFFORT=low
```

With a nonempty key, the app starts in **OpenAI** mode. Without one, it starts
in **Offline mock** mode. Choose either explicitly in **Chat settings → Provider**.
Changes affect new requests; running and completed replies keep their original
provider labels. Live failures produce an Alert with Retry, never a scripted
fallback. Selecting OpenAI without credentials gives a configuration error.

Process environment values take precedence over `.env`, including an explicitly
empty key. Missing/empty model settings use the defaults above. Low, medium and
high reasoning effort are supported session choices. The default model supports
them; another model may reject a choice. Model and effort are sent as configured.

Credentials remain in memory for authorization, have redacted Debug output,
and appear only as a configured/masked indicator. The app does not save or log
them. File changes take effect after relaunch; drawer edits are session-only.
Env files are excluded from git and crate archives, including nested files.

The small loader supports blank/comment lines, `KEY=value`, optional `export`,
single/double quoted literals and trailing comments. It does not evaluate shell
commands, expand variables, interpret escape sequences or mutate process
environment variables. Unrelated variables are ignored. Invalid settings get
generic feedback without echoing their contents.

## Live OpenAI

Requests use the fixed HTTPS Responses API endpoint at
`https://api.openai.com/v1/responses`. Each sends the current system instruction
and completed conversation messages, `reasoning.effort`, `store: false`, and a
2,048-token output limit. `store: false` is a request setting, not a promise of
zero data retention. No custom endpoint, temperature control or tools are exposed.

Streaming is on by default. Turning it off buffers a JSON response instead.
The bounded SSE decoder handles fragmented Unicode, emits public text/refusal
deltas and completes on the terminal response event. It does not display
reasoning events. Truncation, malformed payloads, rate limits and connection
errors become readable Alerts without exposing raw server bodies. Network
callbacks wake egui and deliver through a bounded channel, without an async runtime.

Usage is labelled **OpenAI tokens**, with the returned model and actual API
counts. Missing usage is unavailable; it is never estimated. Text, request bodies
and buffered events are limited to 2 MiB. The response-body timeout is 90 seconds;
connection setup uses the HTTP transport's defaults, not an application-wide deadline.
Oversized conversations require a new conversation; there is no automatic compaction.

Stop immediately discards further local delivery and preserves partial text.
The HTTP reader aborts at its next callback or transport timeout. This does not guarantee
immediate server cancellation or that billing has stopped. Deleting a conversation
or quitting also discards delivery for its pending requests.

## Chat flow

The header pairs the title with a provider badge. **New chat** and the named
settings icon sit on the right; narrow windows use a named plus icon for New chat.
Sidebar conversation names share a left inset, with long names truncated inside
their full-width targets. The [shell verification](verification-dutchy-agent-shell.md)
records the layout and focus checks.

Write a message or use **Try the Rust example** with the selected provider.
Enter sends; Shift+Enter adds a line. IME composition keeps Enter until committed.
Command/Control+N starts another conversation, Command/Control+comma opens settings,
and Command/Control+period stops the current reply. All actions have buttons.

Conversations retain independent drafts and streams while another is selected.
Retry replaces the latest stopped/failed reply without duplicating the user turn,
using the current provider/settings. Deleting the final conversation creates an
empty replacement. Late events cannot revive cancelled requests or alter other chats.

Assistant turns use Markdown with copyable, numbered CodeBlocks. Link activation
is reported inside the app and image loading is disabled. Copy produces a Toast.
Skeletons mark the wait for the first chunk; completion is announced without moving
focus. Scrolling follows the bottom until the user scrolls away.

## Offline mock and limits

Offline mock sends no requests. Its deterministic, timed replies exercise headings,
lists, tables and Rust code. Settings offer success, authentication failure,
rate limiting and an interrupted connection. Restore **Successful reply** before
retrying an intentional error. Disabling streaming delivers the whole scripted reply
after its initial delay. Mock token counts are explicitly labelled estimates.

Tools, attachments, image input, persistence, multiple accounts and other providers
remain future work. See the [application plan](example-applications.md),
[milestone plan](plan-dutchy-agent.md), [live verification](verification-dutchy-agent-live.md),
and the original [offline verification](verification-dutchy-agent.md).
