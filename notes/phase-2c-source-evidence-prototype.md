# Phase 2C SourceEvidenceBundle Prototype

## Scope

Phase 2C adds a read-only SourceEvidenceBundle prototype.

It does not add LSP, SQLite, MCP, embeddings, call graph, reference resolution, import/export semantic resolution, workspace split, xtask, or confident `where-to-edit`.

## What Changed

- Added `src/core/source_evidence.rs`.
- Added `SOURCE_EVIDENCE_CONTRACT_VERSION = "0.1"`.
- Added `code-intel source-evidence "<query>" --json`.
- Added source-evidence eval cases and `source_evidence_cases`.
- Bumped `eval_contract_version` from `0.2` to `0.3`.
- Added semantic tests for exact symbol match, file path match, no match, ambiguous query, malformed source, ignored directories, evidence coverage, deterministic output, and CLI JSON.

## Matching Strategy

The prototype uses deterministic matching only:

- case-insensitive substring match on file paths and symbol names;
- token overlap between query and file/symbol names;
- simple path evidence to attach RepoGraph components, commands, and tests.

It does not use embeddings, LLM scoring, fuzzy semantic inference, source parsing beyond SymbolGraph-lite, references, call graph, or LSP.

## Current Contract Versions

- `inspect`: `0.2`
- `impact`: `0.2`
- `eval`: `0.3`
- `symbols`: `0.1`
- `source_evidence`: `0.1`

## Localization Readiness

Current conclusion: `not_ready_for_confident_localization`.

SourceEvidenceBundle can assemble evidence candidates, but top-level symbols plus string matching are not enough to decide edit locations. `where-to-edit` remains `insufficient_evidence`.

## Phase 2D Recommendation

Harden symbol-to-component linking and refusal behavior:

- source file to RepoGraph component by path evidence;
- symbol to source file to component context;
- component to commands/tests context;
- negative eval cases for false confident localization;
- still no edit localization.
