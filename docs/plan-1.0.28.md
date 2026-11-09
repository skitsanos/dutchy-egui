# Plan — shared commands and shortcuts (1.0.28)

Status: implemented, reviewed and qualified for release as 1.0.28 on 2026-11-09;
see the [release record](verification-release-1.0.28.md).
See [the verification record](verification-command-workflow.md). The CI workflow
is linted locally; hosted execution awaits an approved push or pull request.

1. CommandRegistry: application-owned metadata, stable typed keys, checked and
   enabled state, optional keyboard chords, focus scopes, editing/modal policy,
   conflict diagnostics, exact event-modifier matching and once-per-frame routing.
   Adapters produce MenuItem and ToolbarItem descriptions and platform hints.
2. SplitButton: primary action plus separately named menu arrow, shared variants,
   sizes and busy/disabled state; compose existing native popup/focus handling.
3. KeyRecorder: capture one portable chord, ignore repeats and modifier-only
   input, cancel on Escape/Tab/focus loss, clear explicitly and reject host-listed
   conflicts. Recording suppresses command dispatch and IME input is left intact.
4. Nested palette search: show leaf commands with ancestor breadcrumbs, inherit
   disabled ancestors and use full keyed paths so repeated sibling-local keys
   stay distinct. Keep ordinary SearchableSelect behavior unchanged.
5. Todo and Canal Studio: one command definition feeds shortcuts, menus,
   toolbar/split actions and palette. Demonstrate rebinding and rejected checked
   requests; preserve Todo import safety. Keep all execution in the examples.
6. CI: automate existing required gates, partial features, WASM/MSRV and archive
   allowlist/licence checks. Use current official Actions and minimal permissions.

No new runtime dependency. Maintain existing APIs and package version until
release preparation. Each item has DS-087 through DS-092. The independent
[review](verification-review-1.0.28.md) added and resolved DS-093 through DS-097. Acceptance includes
focused interaction tests, light/dark multi-scale rendering, native macOS
inspection, all required checks and the actual crate archive. Commit and push
remain separate from qualification and require explicit authorization.
