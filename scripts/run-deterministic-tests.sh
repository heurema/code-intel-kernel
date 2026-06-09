#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo run --quiet -- eval-fixtures --json
python3 research-radar/bin/validate_reports.py
python3 -m unittest tests.test_research_radar_experiment_proposal_contract
python3 -m unittest tests.test_research_radar_shared_intake_dependency
git diff --check
