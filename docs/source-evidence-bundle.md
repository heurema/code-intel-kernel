# SourceEvidenceBundle Contract

Status: Phase 2G validated prototype contract. Active as read-only evidence assembly, not localization.

A SourceEvidenceBundle is a source-level evidence packet for future review, localization, and impact reasoning. It combines candidate source files, candidate symbols, source evidence, and relevant RepoGraph context while remaining honest about missing evidence.

Phase 2C implements this contract through `source-evidence`. Phase 2D hardens source-to-repo context roles, candidate limits, ranking, and refusal taxonomy. Phase 2E adds SourceContext as a separate explicit-selector snippet layer. Phase 2F adds SourceContext selector hints. Phase 2G adds adversarial refusal eval cases. SourceEvidenceBundle does not include snippets by default and does not connect to `where-to-edit`.

## Shape

```json
{
  "contract_version": "0.3",
  "status": "ok | partial | insufficient_evidence",
  "query": "change login validation copy",
  "candidate_files": [],
  "candidate_symbols": [],
  "source_context_selectors": [],
  "repo_context": [],
  "source_evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": [],
  "refusal_reason": null
}
```

## candidate_files

Candidate files should be source files that have direct source-level evidence.

Expected fields:

- `path`: repository-relative source path.
- `language`: source language, initially `rust`.
- `parse_status`: `ok | error | unreadable`.
- `confidence`: `high | medium | low | insufficient`.
- `reason`: stable reason string or enum.
- `evidence_ids`: existing source evidence IDs.

## candidate_symbols

Candidate symbols should be source symbols that have declaration evidence.

Expected fields:

- `symbol_id`: deterministic SymbolGraph-lite symbol ID.
- `kind`: symbol kind.
- `name`: symbol name.
- `path`: repository-relative source path.
- `range`: byte and line/column range.
- `confidence`: `high | medium | low | insufficient`.
- `reason`: stable reason string or enum.
- `evidence_ids`: existing source evidence IDs.

Top-level symbols are not enough to claim edit localization. Candidate symbols must remain context, not instructions.

## source_context_selectors

Selector hints are optional read-only handles for `source-context`.

They may point to:

- a candidate file through `selector_kind = "file"`;
- a candidate symbol through `selector_kind = "symbol_id"`.

They must be evidence-backed, deterministic, capped, and documented as context retrieval handles only.

## repo_context

Repo context may link source facts back to RepoGraph facts when evidence supports it:

- component IDs;
- workspace IDs;
- impacted command IDs;
- impacted test IDs;
- manifest evidence IDs.

RepoGraph remains the source of truth for build/test commands and repository-level impact.

## source_evidence

Source evidence should include:

- source file evidence;
- symbol declaration evidence;
- parse status evidence;
- source range evidence;
- warning evidence when present.

## Context Roles

Repo context items include roles:

- `containing_component`
- `containing_workspace`
- `verification_command_context`
- `test_command_context`
- `dependency_context`
- `impact_context`
- `ambiguous_context`

Every candidate file and symbol must reference existing evidence IDs.

## Status Rules

- `ok`: candidates are evidence-backed and limitations are acceptable for the requested use.
- `partial`: some useful source evidence exists, but important evidence is missing.
- `insufficient_evidence`: the bundle cannot support the requested use without guessing.

For edit localization, Phase 2G still expects `insufficient_evidence`.

## Missing Evidence

Use `missing_evidence` for facts that would be required before localization:

- symbol references;
- import/export resolution;
- call graph;
- LSP diagnostics;
- explicit source-context selector policy around candidates;
- adversarial localization refusal cases;
- RepoGraph-to-SymbolGraph linking if needed.

## Refusal Behavior

If evidence is insufficient, a future bundle should include a refusal reason instead of guessed candidates.

Example:

```json
{
  "status": "insufficient_evidence",
  "candidate_files": [],
  "candidate_symbols": [],
  "missing_evidence": [
    "No reference graph",
    "No query-to-symbol relevance model",
    "No reference/call graph layer"
  ],
  "refusal_reason": "Top-level symbols alone are not enough for reliable edit localization."
}
```

## Non-Goals

- No active edit planning.
- No `where-to-edit` integration.
- No embedded source snippets by default.
- No call graph.
- No reference resolution.
- No LSP diagnostics.
- No SQLite persistence.
- No MCP tools.
