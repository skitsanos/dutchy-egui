# Project instructions

- Always communicate in English.
- Build Dutchy as an idiomatic egui library. Keep application state in the host.
- Use the most recent stable crate releases; verify versions when adding or upgrading them.
- Keep runtime dependencies minimal. Prefer small, maintained-in-repository implementations over widget/helper crates. MIT-licensed ideas/code may be adapted with attribution and license notices; preserve all applicable notices.
- Prefer low cognitive complexity, focused functions, and composition. Clippy's cognitive-complexity threshold is 15. Treat 400 lines per code file as a soft cap; split by responsibility before growing a large file.
- All colors, typography, control sizes, spacing, borders, and motion come from shared tokens. Zero corner radius, flat fills, no gradients or blurred shadows.
- Preserve native egui interaction and accessibility behavior. Custom painting must include accessible names/roles and keyboard behavior where applicable.
- Save the implementation plan and reference decisions under `docs/`.
- `docs/issues/` is the Issues Ledger. Use sequential `DS-{number}.md` files with `Problem`, `Remediation`, and `Resolution` sections. Keep entries readable, usually under 500 words; record evidence and leave unresolved limitations explicit.
- Versioning starts at **1.0.0**. Every push to `main` must advance the package version relative to the last pushed version (at least a patch), update `Cargo.lock` and `CHANGELOG.md`, and pass checks. The initial published/pushed baseline is 1.0.0. Never push without user authorization.
- Keep bundled font licenses and third-party notices in the distributed package.
- Run required checks after Rust changes, and fix failures before completion:
  - `cargo fmt --all --check`
  - `cargo test --all-features`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo clippy --all-targets --no-default-features -- -D warnings`
- Verify the renderer at multiple display scales and themes. Use meaningful interaction tests and focused rendering checks; do not claim browser/native parity from compilation alone.
