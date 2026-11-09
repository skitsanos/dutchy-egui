# Release qualification — 1.0.29

Date: 2026-11-09. Follow-up to 1.0.28 (`44591ee`), tracked in
[DS-098](issues/DS-098.md). Integrates
[Renovate PR #4](https://github.com/skitsanos/dutchy-egui/pull/4), commit
`34855b8`, changing only the Linux quality runner from Ubuntu 24.04 to 26.04.
Rust source, dependency requirements and dependency resolutions are unchanged.
Cargo.toml, Cargo.lock and CHANGELOG.md advance together to 1.0.29.

Stable actionlint 1.7.12 initially rejected the new hosted label because its
inventory predates Ubuntu 26.04. `.github/actionlint.yaml` adds exactly that
verified label through the official custom-label configuration; no rule is
suppressed. Remove the entry once a stable release includes it. See the
[upstream configuration guide](https://github.com/rhysd/actionlint/blob/v1.7.12/docs/config.md).

## Plan and upstream evidence

Qualify the final release tree locally, review version-only rendering changes,
inspect the actual crate archive, then commit and push once to main. Preserve
Renovate's commit in the release ancestry so PR #4 can be recognized as merged.
Verify the resulting hosted run and clean inspected outputs and merged branches.

The [PR run](https://github.com/skitsanos/dutchy-egui/actions/runs/42635421850)
passed on Ubuntu 26.04.1, image `ubuntu26/20261101.186`, with stable Rust 1.99.1.
Its Linux quality job passed 596 all-feature, 405 minimal, 12 bundled-font-only
and 48 Markdown-only tests, plus the remaining qualification and archive gates.
macOS and Windows library feature checks also passed. The
[1.0.28 main run](https://github.com/skitsanos/dutchy-egui/actions/runs/42635245220)
had already passed on Ubuntu 24.04; this closes the earlier records' pending
hosted-CI check.

## Final qualification

All 21 local gates pass on macOS arm64 with Rust/Cargo 1.99.1:

| Gate | Result |
| --- | --- |
| Formatting | `cargo fmt --all --check`, pass |
| All-feature tests | 596 passed, including 36 Todo tests |
| Minimal-feature tests | 405 passed |
| Clippy: default, all features, minimal | Each runs `cargo clippy --all-targets [feature flags] -- -D warnings`; pass |
| Strict rustdoc | All features, no dependencies, `-D warnings`; pass |
| WASM library | All features and minimal; both pass |
| Bundled fonts only | All targets compile; 12 rendering/CodeBlock tests pass |
| Markdown parser only | 48 library/Markdown/CodeBlock tests pass |
| MSRV 1.95.0 | Locked all-feature library check passes |
| Native examples | All-feature build passes |
| Workflow and shell | actionlint, ShellCheck and shell syntax pass |
| Relative documentation links and whitespace | Both pass |
| Actual crate build and inspector | Both pass |

No test failures or ignores. The runner-inventory failure was corrected before
delivery; a negative probe confirms an unknown runner is still rejected. No
Rust source changed after the checks. The three rendering themes/scales and
the command/Todo interaction regressions remain covered by the full suite.

The Todo light 1x and dark 2x baselines were regenerated for the header version.
Pixel comparison and visual inspection show only `1.0.28` becoming `1.0.29`:
56 changed pixels at 1x, 160 at 2x, all within the final digit. Other snapshots
are unchanged. No library visual behavior was changed.

`cargo package --allow-dirty --locked --all-features` builds and verifies the
733-file archive. The inspector checks its approved paths, source bytes,
generated manifest/lock/VCS metadata, packaged relative links and all required
font/third-party notices. Root media, storyboard work, tooling settings and
generated snapshot comparisons are excluded. The archive is rebuilt and
inspected after final documentation updates, with commit and push as separate
delivery gates.

The final main-push workflow is pending at qualification time. Its eventual
commit, run and job results are recorded in the private delivery receipt.

## Limits and delivery

This is a CI runner and package-version release. Native interaction is not
repeated; the full rendering/interaction suite covers the unchanged Rust tree.
macOS/Windows hosted jobs check library feature profiles, not native desktop
interactions. Registry publication and release tags are outside this delivery.
Post-push CI and cleanup receipts will be retained privately under
`.git/codex-maintenance/release-1.0.29/`.
