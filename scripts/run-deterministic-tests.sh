#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo run --quiet -- eval-fixtures --json
python3 research-radar/bin/validate_reports.py
git diff --check
