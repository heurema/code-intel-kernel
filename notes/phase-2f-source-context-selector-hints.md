# Phase 2F: Source Context Selector Hints

Status: implemented, uncommitted.

Phase 2F adds explicit SourceContext selector hints to SourceEvidenceBundle.

## What changed

- Bumped `source_evidence` contract from `0.2` to `0.3`.
- Added `source_context_selectors` to SourceEvidenceBundle.
- Generated selector hints only from evidence-backed candidate files and candidate symbols.
- Kept SourceEvidence free of source snippets.
- Kept SourceContext as the layer that returns bounded source text from explicit selectors.

## Selector hint behavior

Selector hints can be:

- `file`: pass `file_path` to `source-context --file`.
- `symbol_id`: pass `symbol_id` to `source-context --symbol-id`.

Hints include reason, confidence, evidence IDs, and limitations.

## Limits

Selector hints are capped at 12 and sorted deterministically. If truncated, SourceEvidence emits `selector_hint_limit_exceeded`.

## Localization readiness

Current conclusion remains `not_ready_for_confident_localization`.

Selector hints are context retrieval handles. They are not edit targets and are not connected to `where-to-edit`.

## Phase 2G recommendation

Add an adversarial readiness gate before localization:

- same symbol name in multiple files;
- broad query with many candidates;
- ignored/generated paths;
- malformed source;
- no reference or call graph;
- explicit proof that `where-to-edit` still refuses.
