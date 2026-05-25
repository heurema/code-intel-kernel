# Evaluation Harness

Status: Phase 2E fixture evaluation contract.

The evaluation harness measures current `inspect`, `impact`, `symbols`, `source-evidence`, and `source-context` behavior across small fixtures.

It does not evaluate imports, references, call graphs, edit planning, LSP diagnostics, embeddings, or MCP behavior.

## Command

```bash
cargo run --quiet -- eval-fixtures --json
```

The command loads JSON cases from `tests/eval/cases/` and returns an evaluation report.

## Report Shape

```json
{
  "eval_contract_version": "0.4",
  "total_cases": 0,
  "passed_cases": 0,
  "failed_cases": 0,
  "inspect_cases": 0,
  "impact_cases": 0,
  "symbol_cases": 0,
  "source_evidence_cases": 0,
  "source_context_cases": 0,
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

The eval report has its own contract version. Phase 2B bumps it to `0.2` because the report counts `symbol_cases` and accepts `symbols` eval cases. Phase 2C bumps it to `0.3` because the report counts `source_evidence_cases` and accepts `source_evidence` eval cases. Phase 2E bumps it to `0.4` because the report counts `source_context_cases` and accepts `source_context` eval cases. This does not change the `inspect`, `impact`, `symbols`, `source_evidence`, or `source_context` contracts.

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
    "confidence": "high",
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
- `symbols`
- `source_evidence`
- `source_context`

Expectations are semantic checks, not full-output snapshots. A case can assert required facts, forbidden facts, expected warning categories, unexpected warning categories, impact status, confidence, scope, and maximum impacted component count.

Symbol cases use the same file format:

```json
{
  "name": "rust_symbols_basic_symbols",
  "fixture": "tests/fixtures/rust-symbols-basic",
  "kind": "symbols",
  "expect": {
    "source_files_contains": ["src/lib.rs"],
    "symbols_contains": [
      { "name": "top_level_function", "kind": "function" },
      { "name": "Widget", "kind": "struct" }
    ],
    "symbols_not_contains": [
      { "name": "nested_helper", "kind": "function" }
    ],
    "warnings_not_contains_categories": ["parse_error"]
  }
}
```

For `symbols_contains` and `symbols_not_contains`, `kind` and `path` may be supplied to make the match more specific. Symbol eval always checks evidence coverage and deterministic output across repeated extraction.

Source-evidence cases can assert candidate files, candidate symbols, status, confidence, warnings, and missing evidence:

```json
{
  "name": "source_evidence_function_match",
  "fixture": "tests/fixtures/rust-symbols-basic",
  "kind": "source_evidence",
  "query": "top_level_function",
  "expect": {
    "status": "partial",
    "confidence": "medium",
    "candidate_files_contains": ["src/lib.rs"],
    "candidate_symbols_contains": [
      { "name": "top_level_function", "kind": "function" }
    ],
    "warnings_contains_categories": ["localization_not_supported"],
    "missing_evidence_contains": ["no_symbol_reference_layer"]
  }
}
```

Source-context cases use explicit selectors and can assert slices, symbol slices, bounded text, warnings, and max lines:

```json
{
  "name": "source_context_file_slice",
  "fixture": "tests/fixtures/rust-symbols-basic",
  "kind": "source_context",
  "selector_file": "src/lib.rs",
  "selector_lines": "1:8",
  "expect": {
    "status": "ok",
    "slices_contains": ["src/lib.rs"],
    "slice_text_contains": ["pub fn top_level_function"],
    "warnings_contains_categories": ["source_context_not_localization"],
    "max_slice_lines": 8
  }
}
```

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

False narrow remains the higher-risk failure mode for future localization gates.

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
- Rust top-level symbol extraction;
- Rust malformed source parse warning;
- ignored source paths for SymbolGraph-lite.
- SourceEvidenceBundle function match, file match, and no-match refusal.
- SourceEvidenceBundle broad-query candidate limit and malformed-source refusal.
- SourceContext file slice, symbol slice, missing file, and ignored path refusal.

## Negative Case Rationale

`negative_no_dependency_edge_impact` allows two impacted components for the minimal Cargo fixture because the current RepoGraph represents both the crate package and its library target. The case still prevents fake transitive or broader dependency impact when no `depends_on` edge exists.

## Limitations

- Cases are hand-authored and fixture-sized.
- The harness does not run external ecosystem tools.
- It does not execute recommended commands.
- It does not validate source-level localization.
- SourceContext eval validates explicit read-only slicing only.
- It does not score performance yet.
- It does not persist historical trend data.

## Localization Gate

Before `where-to-edit` can stop refusing, the project should have:

- all current inspect and impact eval cases passing;
- all current symbols eval cases passing;
- 100% evidence coverage on eval cases;
- deterministic output across repeated runs;
- zero false narrow count on core fixtures;
- false broad cases documented and accepted;
- structured warnings that match expectations;
- `where-to-edit` still returning `insufficient_evidence`.

Passing Phase 2B eval does not make the kernel ready for confident localization. Top-level symbols are source facts, not edit candidates.
