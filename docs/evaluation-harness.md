# Evaluation Harness

Status: Phase 1F RepoGraph evaluation contract draft.

The evaluation harness measures current `inspect` and `impact` behavior across small fixtures. It is intentionally limited to repository/build/test-level facts.

It does not evaluate symbols, imports, references, call graphs, edit planning, LSP diagnostics, embeddings, or MCP behavior.

## Command

```bash
cargo run --quiet -- eval-fixtures --json
```

The command loads JSON cases from `tests/eval/cases/` and returns an evaluation report.

## Report Shape

```json
{
  "eval_contract_version": "0.1",
  "total_cases": 0,
  "passed_cases": 0,
  "failed_cases": 0,
  "inspect_cases": 0,
  "impact_cases": 0,
  "metrics": {
    "evidence_coverage_pass_rate": 1.0,
    "expected_fact_recall": 1.0,
    "unexpected_warning_count": 0,
    "missing_expected_warning_count": 0,
    "false_broad_count": 0,
    "false_narrow_count": 0,
    "deterministic_output_pass_rate": 1.0
  },
  "cases": [],
  "failures": []
}
```

The eval report has its own contract version. It does not change the `inspect` or `impact` contracts.

## Case Format

Cases live under `tests/eval/cases/*.json`.

```json
{
  "name": "cargo_workspace_dependency_impact",
  "fixture": "tests/fixtures/cargo-workspace-deps",
  "kind": "impact",
  "changed_files": ["crates/b/src/lib.rs"],
  "expect": {
    "status": "partial",
    "confidence": "medium",
    "impact_scope": "mixed",
    "components_contains": ["b", "a"],
    "commands_contains": ["cargo test"],
    "tests_contains": ["cargo test"],
    "warnings_not_contains_categories": ["malformed_manifest"],
    "max_impacted_components": 2
  }
}
```

Known `kind` values:

- `inspect`
- `impact`

Expectations are semantic checks, not full-output snapshots. A case can assert required facts, forbidden facts, expected warning categories, unexpected warning categories, impact status, confidence, scope, and maximum impacted component count.

## Metrics

- `evidence_coverage_pass_rate`: share of cases where all emitted graph facts reference existing evidence.
- `expected_fact_recall`: share of semantic expectations that passed.
- `unexpected_warning_count`: warnings that were explicitly forbidden by a case.
- `missing_expected_warning_count`: expected warning categories that were absent.
- `false_broad_count`: impact or extraction was broader than the case allowed.
- `false_narrow_count`: expected components, commands, tests, or warning categories were missing.
- `deterministic_output_pass_rate`: share of cases where repeated runs produced the same semantic output.

## False Broad vs False Narrow

False broad means the kernel recommends or reports more than the fixture case allows. This is safer than missing impact, but it is still a quality problem because it can create unnecessary work.

False narrow means the kernel misses expected components, commands, tests, or warnings. This is higher risk for build/test impact because a consumer could skip necessary validation.

Phase 1F treats false narrow on core fixtures as a blocker for starting SymbolGraph.

## Current Case Matrix

The initial case set covers:

- minimal Rust crate inspect;
- Cargo workspace dependency inspect;
- Cargo workspace dependency impact;
- explicit Cargo bin inspect;
- Makefile command extraction;
- justfile command extraction;
- Python pytest evidence inspect;
- Python ambiguous tests warning inspect;
- Python manifest impact;
- Go module inspect;
- Go test-file impact;
- malformed Python manifest warning;
- malformed Go manifest warning;
- unknown changed file refusal;
- Python tests without runner evidence refusal;
- malformed Node manifest refusal;
- no-dependency-edge impact boundary.

## Negative Case Rationale

`negative_no_dependency_edge_impact` allows two impacted components for the minimal Cargo fixture because the current RepoGraph represents both the crate package and its library target. The case still prevents fake transitive or broader dependency impact when no `depends_on` edge exists.

## Limitations

- Cases are hand-authored and fixture-sized.
- The harness does not run external ecosystem tools.
- It does not execute recommended commands.
- It does not validate source-level localization.
- It does not score performance yet.
- It does not persist historical trend data.

## SymbolGraph Readiness Gate

Before starting SymbolGraph, the project should have:

- all current inspect and impact eval cases passing;
- 100% evidence coverage on eval cases;
- deterministic output across repeated runs;
- zero false narrow count on core fixtures;
- false broad cases documented and accepted;
- structured warnings that match expectations;
- `where-to-edit` still returning `insufficient_evidence`.
