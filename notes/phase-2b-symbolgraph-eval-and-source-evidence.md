# Phase 2B SymbolGraph Eval and Source Evidence

## Scope

Phase 2B adds fixture evaluation for SymbolGraph-lite and documents SourceEvidenceBundle as a future contract.

It does not add LSP, SQLite, MCP, embeddings, call graph, reference resolution, import/export semantic resolution, workspace split, xtask, or confident `where-to-edit`.

## What Changed

- Added `symbols` as an eval case kind.
- Bumped `eval_contract_version` from `0.1` to `0.2`.
- Added `symbol_cases` to the eval report.
- Added semantic symbol expectations for source files, required symbols, forbidden symbols, warnings, evidence coverage, and deterministic output.
- Added SymbolGraph-lite eval cases for top-level Rust symbols, malformed Rust source, and ignored directories.
- Added `docs/source-evidence-bundle.md`.
- Added `docs/localization-readiness-checklist.md`.

## Current Contract Versions

- `inspect`: `0.2`
- `impact`: `0.2`
- `eval`: `0.2`
- `symbols`: `0.1`

## SourceEvidenceBundle Status

SourceEvidenceBundle is documentation only in this phase. It is not wired to a CLI command or `where-to-edit`.

The intended role is to package:

- query or task;
- candidate source files;
- candidate symbols;
- RepoGraph context;
- source evidence;
- warnings;
- limitations;
- missing evidence;
- refusal reason.

## Localization Readiness

Current conclusion: `not_ready_for_confident_localization`.

Top-level symbols are useful source facts, but they do not prove edit relevance. Missing pieces include query-to-symbol relevance, references, source context policy, negative localization eval cases, and a SourceEvidenceBundle runtime prototype.

## Phase 2C Recommendation

Prototype SourceEvidenceBundle generation without edit localization:

- input: query plus repo path;
- output: `partial` or `insufficient_evidence` bundle;
- include SymbolGraph-lite source facts and RepoGraph context where evidence supports it;
- keep candidate files/symbols as context only;
- keep `where-to-edit` returning `insufficient_evidence`.
