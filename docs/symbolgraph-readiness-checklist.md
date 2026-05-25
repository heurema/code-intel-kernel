# SymbolGraph Readiness Checklist

Status: Phase 1G decision gate.

Before starting SymbolGraph implementation, the project should satisfy this checklist.

## RepoGraph Quality

- [x] Inspect facts are evidence-backed.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `evidence_coverage_pass_rate = 1.0`; inspect evidence tests in `tests/smoke.rs`.
- [x] Impact facts and recommendations are evidence-backed.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `evidence_coverage_pass_rate = 1.0`; impact evidence tests in `tests/smoke.rs`.
- [x] Inspect output is deterministic across repeated fixture runs.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `deterministic_output_pass_rate = 1.0`.
- [x] Impact output is deterministic across repeated fixture runs.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `deterministic_output_pass_rate = 1.0`.
- [x] Eval evidence coverage is 100%.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `evidence_coverage_pass_rate = 1.0`.
- [x] Eval false narrow count is zero on current core fixtures.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `false_narrow_count = 0`.
- [x] Eval false broad count is zero on current core fixtures.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, metric `false_broad_count = 0`.
- [x] Warnings are structured.
  Evidence: inspect/impact warning assertions in `tests/smoke.rs` and warning categories in `docs/inspect-json-contract.md` and `docs/impact-json-contract.md`.
- [x] Malformed manifests produce warnings, not panics.
  Evidence: eval cases `malformed_pyproject_warning`, `malformed_go_mod_warning`, and `negative_malformed_node_manifest_inspect`.
- [x] Unsupported or ambiguous information produces warnings or refusal, not guesses.
  Evidence: eval cases `negative_unknown_change_impact`, `python_ambiguous_tests_inspect`, and `negative_python_ambiguous_test_impact`.
- [x] `where-to-edit` still refuses to guess.
  Evidence: `cargo run --quiet -- where-to-edit "change login validation copy" --profile=strict --json`; smoke test `where_to_edit_remains_insufficient_evidence_placeholder`.

## Boundary Clarity

- [x] RepoGraph owns build/test/repository-level facts.
  Evidence: `docs/repo-vs-symbol-graph-boundary.md`.
- [x] SymbolGraph will own source/symbol-level facts.
  Evidence: `docs/repo-vs-symbol-graph-boundary.md` and `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] SymbolGraph will not replace RepoGraph command inference.
  Evidence: `docs/repo-vs-symbol-graph-boundary.md`.
- [x] RepoGraph will not pretend to localize edit files.
  Evidence: `docs/repo-vs-symbol-graph-boundary.md`; `where-to-edit` smoke test.
- [x] Consumer-specific policies remain outside core.
  Evidence: `docs/consumers/` profiles and `docs/repo-vs-symbol-graph-boundary.md`.

## Phase 2A Scope Gate

- [x] Phase 2A has a narrow SymbolGraph-lite plan.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] No call graph in Phase 2A.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] No LSP in Phase 2A.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] No SQLite in Phase 2A.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] No MCP in Phase 2A.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] No embeddings in Phase 2A.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md`.
- [x] No confident `where-to-edit` localization until symbol evidence is evaluated.
  Evidence: `docs/phase-2a-symbolgraph-lite-plan.md` and `docs/repo-vs-symbol-graph-boundary.md`.

## Known Gaps Not Covered By This Checklist

- Cyclic dependency edge traversal.
- Symlink escapes and path containment edge cases.
- Unsupported-language repositories beyond current fixtures.
- Mixed root ecosystems.
- Generated code and build artifact edge cases.
- Dynamic/custom build scripts.
- Source parse failures.
- Large-repository behavior.

## Current Conclusion

Decision: `ready_with_constraints`.

RepoGraph is ready to support a narrow SymbolGraph-lite implementation. The constraint is that Phase 2A must add source facts only, keep evidence strict, and avoid edit localization claims until new SymbolGraph eval cases prove useful.
