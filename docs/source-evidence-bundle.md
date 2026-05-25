# SourceEvidenceBundle Contract

Status: Phase 2D prototype contract. Active as read-only evidence assembly, not localization.

A SourceEvidenceBundle is a source-level evidence packet for future review, localization, and impact reasoning. It combines candidate source files, candidate symbols, source evidence, and relevant RepoGraph context while remaining honest about missing evidence.

Phase 2C implements this contract through `source-evidence`. Phase 2D hardens source-to-repo context roles, candidate limits, ranking, and refusal taxonomy. Phase 2E adds SourceContext as a separate explicit-selector snippet layer. SourceEvidenceBundle does not include snippets by default and does not connect to `where-to-edit`.

## Shape

```json
{
  "contract_version": "0.2",
  "status": "ok | partial | insufficient_evidence",
  "query": "change login validation copy",
  "candidate_files": [],
  "candidate_symbols": [],
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

For edit localization, Phase 2D still expects `insufficient_evidence`.

## Missing Evidence

Use `missing_evidence` for facts that would be required before localization:

- symbol references;
- import/export resolution;
- call graph;
- LSP diagnostics;
- explicit source-context selector policy around candidates;
- evaluated negative localization cases;
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
    "No evaluated localization cases"
  ],
  "refusal_reason": "Top-level symbols alone are not enough for reliable edit localization."
}
```

## Non-Goals

- No active edit planning.
- No `where-to-edit` integration.
- No call graph.
- No reference resolution.
- No LSP diagnostics.
- No SQLite persistence.
- No MCP tools.
