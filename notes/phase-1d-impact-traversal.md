# Phase 1D Impact Traversal

## What Changed

- Extended impact output with `impact_scope`, report-level `confidence`, per-item `impact_kind`, `distance`, `reason`, `rank`, and `evidence_ids`.
- Added conservative reverse dependency traversal over explicit RepoGraph `depends_on` relationships.
- Added Cargo workspace member component detection and `depends_on` edges for explicit path dependencies.
- Added deterministic ordering for changed files, impacted facts, warnings, recommendations, and traversal results.
- Kept `where-to-edit` as `insufficient_evidence`.

## Current Contract

`impact` uses `contract_version: "0.2"` because Phase 1D adds impact scope, confidence, impact kind, distance, reasons, ranks, and recommendation evidence IDs.

The report is RepoGraph-only:

- `direct`: changed path matched component file scope.
- `transitive`: reverse dependency traversal found a dependent component.
- `broad`: manifest, lockfile, workspace config, or build config changed.
- `uncertain`: reserved for future cases where a fact is known but cannot be scoped.

Confidence is intentionally coarse:

- `high`: scoped component command/test evidence exists.
- `medium`: component impact exists but commands/tests are generic or repo-scoped.
- `low`: only broad repository-level evidence exists.
- `insufficient`: no supported mapping exists.

## Limitations

- No SymbolGraph, Tree-sitter, import graph, call graph, or LSP diagnostics.
- Dependency traversal only uses explicit RepoGraph `depends_on` edges.
- Cargo dependency extraction is limited to workspace member path dependencies.
- Command ranking is heuristic and build/test-level only.
- No storage layer; all analysis is in memory.

## Why SymbolGraph/LSP/MCP Stay Deferred

RepoGraph impact is now useful as a build/test primitive, but it still needs stronger build-system extraction before source-level confidence is justified. SymbolGraph and LSP should be added only after repository-level impact remains stable across fixtures and real repositories.

## Phase 1E-A Next

- Improve Cargo, Makefile, and justfile extractor quality first.
- Add narrow fixture coverage for existing RepoGraph extractors.
- Keep Python/Go expansion and benchmark harness separate.
- Improve command scope inference before introducing SymbolGraph.
