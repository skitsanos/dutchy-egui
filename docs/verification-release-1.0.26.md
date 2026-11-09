# Release qualification — 1.0.26

Date: 2026-11-09. Feature release combining four local branches on top of
1.0.25 (`5d68a93`): the richer-menu work with its review fixes
([DS-073](issues/DS-073.md) to [DS-076](issues/DS-076.md) and
[DS-081](issues/DS-081.md) to [DS-084](issues/DS-084.md)), the DutchyTodo
example and shared example scaffolding ([DS-077](issues/DS-077.md)), the
scroll-fade fix ([DS-078](issues/DS-078.md)), and the documentation from the
independent review ([DS-079](issues/DS-079.md), [DS-080](issues/DS-080.md),
[review record](verification-review-ds073-076.md)) together with the roadmap
rewrite and the [example applications plan](example-applications.md).
Cargo.toml, Cargo.lock and CHANGELOG.md advance together from 1.0.25.

## Integration

The branches were merged in that order. Conflicts were limited to the
CHANGELOG lists (both kept), the roadmap (the rewrite kept, with a 1.0.26 line
for the menus), the documentation index (both link lists combined), the
coverage matrix (the MenuBar row kept, the blank line removed as DS-079
intends) and the open copies of DS-081 to DS-083 (the resolved versions kept).
The example's own scroll-fade override was removed because the library theme
now disables the fade. Ticket statuses DS-073 to DS-084 read "Resolved in
1.0.26"; living guides and the coverage matrix name the release, and the dated
verification records point here instead of being rewritten.

The two DutchyTodo baselines changed because the example header shows the
crate version: only the final digit differs (53 pixels at 1x, 137 at 2x). The
development guide now lists this as a release step. No other baseline changed.

## Gates

Run on macOS 27.2 with Rust 1.99.1 on the merged tree:

- `cargo fmt --all --check`: clean.
- `cargo test --all-features`: 554 passed, 0 failed, 0 ignored, including the
  24 DutchyTodo example tests.
- `cargo test --no-default-features`: 379 passed, 0 failed, 0 ignored.
- `cargo clippy --all-targets -- -D warnings`, plus the all-features and
  no-default-features configurations: clean.
- `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`: clean.
- `cargo check --lib` for `wasm32-unknown-unknown` with all features and with
  no default features: pass.
- `cargo check --all-targets --no-default-features --features bundled-fonts`:
  pass; `cargo test --no-default-features --features markdown`: 402 passed.
- `cargo +1.95.0 check --lib --all-features --locked` (declared minimum): pass.
- `cargo build --examples --all-features` (showcase and DutchyTodo): pass.
- Relative documentation links: none broken.

The Rust-only gates ran before the two baselines, this record and one guide
paragraph were updated; the test suites, the link check and the archive below
ran again afterwards on the final tree.

## Crate archive

`cargo package --allow-dirty --locked --all-features` built and verified the
archive: 685 files. A script compared every ordinary file with the merged
tree and with the root-anchored `include` list: all match, nothing is missing
or unexpected, and no IDE, media, storyboard, tool-settings or
snapshot-difference file is present. The Display font licence carries its
upstream CRLF bytes (DS-080), and the three font licences and
`THIRD_PARTY.md` are included.

## Limits

Native behavior of the review fixes DS-081 to DS-084 was not re-checked in the
running showcase; the menus' earlier macOS observations predate them. Windows,
Linux, browsers and screen readers remain unverified. Commit and push are
separate from this qualification, and the push needs explicit authorization.
