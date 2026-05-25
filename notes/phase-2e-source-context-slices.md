# Phase 2E: Source Context Slices

Status: implemented, uncommitted.

Phase 2E adds a read-only `SourceContext` layer for bounded source snippets around explicit files or SymbolGraph-lite symbols.

## What changed

- Added `source-context` JSON contract version `0.1`.
- Added explicit selectors:
  - `--file <path>`
  - `--file <path> --lines <start:end>`
  - `--symbol-id <symbol-id>`
- Added bounded snippets with line/byte ranges, truncation flag, content hash, and evidence IDs.
- Added path safety checks for traversal, ignored directories, symlinks, missing files, non-UTF8 files, binary-looking files, and unsupported languages.
- Extended eval with `source_context` cases and bumped eval contract to `0.4`.

## What this is not

SourceContext is not localization. It does not choose files or symbols from a natural-language query. It does not plan patches and does not connect to `where-to-edit`.

## Current limits

- max slices per report: 8
- max lines per slice: 80
- max bytes per slice: 8000
- max total bytes per report: 20000

## Localization readiness

Current conclusion remains `not_ready_for_confident_localization`.

SourceContext gives bounded source text after an explicit selector exists. The kernel still lacks reference/call graph evidence, semantic relevance evidence, LSP diagnostics, and negative localization eval cases.

## Phase 2F recommendation

Add explicit selector hints between SourceEvidenceBundle and SourceContext:

- source-evidence candidate file -> source-context file selector hint;
- source-evidence candidate symbol -> source-context symbol selector hint;
- no snippets by default;
- no edit target;
- no `where-to-edit` integration.
