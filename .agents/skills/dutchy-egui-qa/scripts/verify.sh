#!/usr/bin/env bash
set -euo pipefail

repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
cd "$repo"

unset UPDATE_SNAPSHOTS
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"

if (($#)); then
    tests=()
    for suite in "$@"; do
        tests+=(--test "$suite")
    done
    cargo test --locked --all-features "${tests[@]}"
    cargo test --locked --no-default-features "${tests[@]}"
else
    bash scripts/check.sh
    cargo clippy --all-targets -- -D warnings
fi

git diff --check HEAD
