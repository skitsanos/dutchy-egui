# 1.0.28 independent review

Date: 2026-11-09. Scope: the uncommitted command workflow, [DS-087](issues/DS-087.md)
through [DS-092](issues/DS-092.md), on `ticket/ds-087-command-workflow` over
1.0.27 (`0fd402e`). The [qualification record](verification-command-workflow.md)
covers the original implementation. Nothing was committed or pushed during
the review; the fixes ship in [1.0.28](verification-release-1.0.28.md).

## Findings and fixes

| Ticket | Finding | Fix |
| --- | --- | --- |
| [DS-093](issues/DS-093.md) | Bare Enter, Space and arrow bindings were swallowed app-wide | Dispatch leaves them to a focused control; the recorder needs a modifier unless the host opts in |
| [DS-094](issues/DS-094.md) | macOS Control was dropped from Command chords; Command bindings fired on Command+Control | Control is kept; dispatch compares chords exactly |
| [DS-095](issues/DS-095.md) | Disabled and busy split buttons showed a seam and an unfaded divider; a focused Outline arrow hid its chevron | The joint is painted once, the divider fades, focus rings are repainted, the chevron follows focus |
| [DS-096](issues/DS-096.md) | The column chooser lost its roadmap link; coverage had no rows for the new controls | Roadmap and coverage updated |
| [DS-097](issues/DS-097.md) | CI snapshot tests failed under Linux software Vulkan | Linux-only pixel allowance in `kittest.toml`; macOS stays strict |

## Evidence

- **Before the fixes:** all 21 local gates passed on a frozen copy of the
  tree, with 590/399/12/48 tests. Scratch probes reproduced DS-093 and DS-094.
  An arm64 Docker rehearsal of the CI test step failed 3 of 590 tests. It used
  Ubuntu 24.04, the workflow's packages and software Vulkan.
- **Native macOS showcase:** the split primary, Command+S, rejection of an
  occupied chord and rebinding all reached the host, and the menu hint
  followed the new chord.
- **Each fix was test-first.** The six new tests failed for the reported
  reason before the change and pass after it.
- **After the fixes:** all 21 gates passed again in the checkout. Tests
  passed: 596 with all features, 405 without default features, 12 with bundled
  fonts only and 48 with the Markdown parser only. The archive inspector and
  audit passed after the final documentation edits. The Linux rehearsal passed
  596 tests with no failures.
- **Rendering:** the three command-control baselines were regenerated and
  reviewed at light 1x, dark 1.5x and light 2x. Keyboard focus on each half of
  Outline and Primary split buttons was inspected in a scratch 2x render.
  Menu, Todo and all other baselines are unchanged.

## Limits

- Hosted CI has not run. The x86_64 runner may rasterize differently from the
  arm64 rehearsal.
- The native check stopped when the showcase window was covered, because
  eframe pauses input while occluded. A relaunch after the fixes was covered
  too, so DS-093 to DS-095 are verified by interaction and pixel tests, not
  natively. DutchyTodo was not re-checked natively.
- Windows, Linux desktops and screen readers remain unverified.
