# Phase 2D Source-Repo Linking and Refusal

## Scope

Phase 2D hardens SourceEvidenceBundle linking and refusal behavior.

It does not add LSP, SQLite, MCP, embeddings, call graph, reference resolution, import/export semantic resolution, workspace split, xtask, or confident `where-to-edit`.

## What Changed

- Bumped `source_evidence` contract from `0.1` to `0.2`.
- Added explicit repo context roles.
- Added source-to-repo context linking by path evidence.
- Added RepoGraph impact context for candidate source paths.
- Added deterministic candidate limits.
- Added clearer ranking reasons.
- Added refusal and missing-evidence taxonomy.
- Added source-evidence eval cases for broad-query limits and malformed source.

## Context Roles

- `containing_component`
- `containing_workspace`
- `verification_command_context`
- `test_command_context`
- `dependency_context`
- `impact_context`
- `ambiguous_context`

## Candidate Limits

- candidate files: 8
- candidate symbols: 12
- repo context items: 12

If a limit is exceeded, output is truncated deterministically and includes `candidate_limit_exceeded`.

## Ranking

Ranking uses deterministic local signals only:

- exact symbol name match;
- exact file path match;
- substring match;
- token overlap;
- stable tie-break by path, name, and ID.

No embeddings, LLM scoring, fuzzy semantic inference, LSP, references, or call graph are used.

## Localization Readiness

Current conclusion: `not_ready_for_confident_localization`.

SourceEvidenceBundle can now explain source-to-repo context more clearly, but it is still evidence assembly. It does not identify edit locations. `where-to-edit` remains `insufficient_evidence`.

## Phase 2E Recommendation

Add read-only source context slices:

- candidate file/symbol to source snippet;
- exact byte/line range evidence;
- snippet limits and redaction rules;
- still no patching;
- still no confident `where-to-edit`.
