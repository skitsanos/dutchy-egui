---
name: dutchy-egui-qa
description: Verify Dutchy egui component changes and regressions through public API review, headless interaction, rendering, native Showcase checks, and release qualification.
---

# Dutchy egui QA

Read `AGENTS.md`. Start from the current checkout and compare the change to
the relevant host-facing API, Showcase use, and existing tests. Keep a
verification-only request read-only and preserve work already in the checkout.

## Choose evidence for the change

- **Contract.** Check who owns durable state and what the component returns in
  one frame. Trace requests through their real caller, including a host that
  declines a request. Check new public types against `docs/versioning.md`.
- **Interaction.** Use `egui_kittest` with `tests/common/mod.rs` to exercise
  pointer, keyboard, and AccessKit entry points that the component exposes.
  Assert both the requested action and neighboring actions that must stay
  idle. For compound controls, check hit rectangles, native Tab order, and
  accessible names and roles. Exercise a focused control during relevant
  transitions, such as disabling its parent, resizing, or removing its row.
- **Rendering.** Verify relevant states across at least two themes and pixel
  scales. Check geometry or pixels for the invariant in question, including
  narrow widths when they affect layout. Use snapshots for broad visual
  regressions and inspect any deliberate refresh before accepting it.
- **Native.** When a change depends on real window input, OS dialogs, or native
  focus, use a task-owned Showcase instance and exercise the affected flow.
  Headless AccessKit tests establish tree semantics. Report real screen reader
  coverage only when a reader was used.

For a concrete bug, reproduce it with the regression test before applying the
fix. When verifying a fix that already exists, a minimal reversion in a
temporary copy can confirm the test catches it. Keep failures and visual
proofs outside tracked source, and distinguish an expected regression failure
from a compile error or an unavailable test environment.

## Run checks

From the repository root, run the focused suites relevant to the change in
both feature profiles. For example:

```sh
bash .agents/skills/dutchy-egui-qa/scripts/verify.sh chip
```

For a full qualification, call the same script with no test names. It delegates
to `scripts/check.sh`, the gate shared with CI. Build or render evidence still
needs to match the requested behavior. Report what passed, any reproduction
that remains, and the scope of native verification.
