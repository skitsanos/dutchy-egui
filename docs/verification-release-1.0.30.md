# Release qualification — 1.0.30

Date: 2026-11-09. Follows 1.0.29 (`3a92168`). Cargo.toml, Cargo.lock and
CHANGELOG.md advance together to 1.0.30.

## Contents

- [DS-099](issues/DS-099.md)..[DS-101](issues/DS-101.md): the DutchyAgent
  example, with an offline mock and an optional OpenAI Responses provider.
  Button gains additive `text_align`.
- [DS-102](issues/DS-102.md): tests, baselines and examples leave the package,
  which drops under the crates.io upload limit.
- [DS-103](issues/DS-103.md): nine faults from an independent review of the
  example, including an idle watchdog in place of a total body timeout.
- [DS-104](issues/DS-104.md): `Text` centers in left-to-right wrapping rows.
  This is the release's only change to existing library rendering.

New dev-dependencies, used only by the example: ehttp 0.7.3 (streaming, no
default features) and serde_json 1.0.154, both the newest releases on the
qualification date. Runtime dependencies are unchanged.

## Final qualification

All 21 local gates pass on macOS arm64 with Rust/Cargo 1.99.1:

| Gate | Result |
| --- | --- |
| Formatting | `cargo fmt --all --check`, pass |
| All-feature tests | 642 passed, including 44 DutchyAgent and 36 Todo tests |
| Minimal-feature tests | 407 passed |
| Clippy: default, all features, minimal | Each runs `cargo clippy --all-targets [feature flags] -- -D warnings`; pass |
| Strict rustdoc | All features, no dependencies, `-D warnings`; pass |
| WASM library | All features and minimal; both pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| MSRV 1.95.0 | Locked all-feature library check passes |
| Native examples | All-feature build passes |
| Workflow and shell | actionlint, ShellCheck and shell syntax pass |
| Relative documentation links and whitespace | 437 links resolve; whitespace passes |
| Actual crate build and inspector | Both pass |

No test failures or ignores. The first run of the inspector stopped on links
to this record before it existed; the archive was rebuilt and inspected again
after it was written. No Rust source changed after the checks.

## Rendering changes

Fourteen existing baselines changed in DS-104 and were reviewed one by one:
the DutchyTodo header, the lightbox image counter and the DutchyAgent captures
now center text on their controls. No other snapshot changed. For the version,
the Todo light 1x and dark 2x baselines were regenerated: 107 pixels at 1x and
300 at 2x change, all inside "29" becoming "30".

Native Glow captures of DutchyAgent at 1x light and 2x dark were taken during
DS-103 in offline mock with no `.env` loaded. Todo and showcase native runs
were not repeated; the kittest suite covers both themes and three scales.

## Package

`cargo package --allow-dirty --locked --all-features` builds and verifies a
429-file archive of about 1.8 MB. The inspector checks approved paths, source
bytes, packaged links, required font and third-party notices, and rejects
tests, examples, env files and snapshot comparisons. Cargo's notices that tests
and examples are not packaged are expected since DS-102.

## Limits and delivery

No live OpenAI request was made in this qualification. The DS-100 record holds
the only live evidence, a native turn made before DS-103 changed the transport.
No Linux rehearsal was run locally; the hosted run after the push is the Linux
check. macOS and Windows hosted jobs check library feature profiles, not
native interaction.

The release is one commit on a local `release/1.0.30` branch over local main,
with an annotated `v1.0.30` tag on it. Pushing the branch to main and the tag
needs explicit authorization. crates.io publication is outside this delivery.
