# SourceEvidenceBundle Contract

Status: Phase 2B design contract. Not an active CLI command or localization API.

A SourceEvidenceBundle is a future source-level evidence packet for review, localization, and impact reasoning. It should combine candidate source files, candidate symbols, source evidence, and relevant RepoGraph context while remaining honest about missing evidence.

Phase 2B only documents this contract. It does not connect SymbolGraph-lite to `where-to-edit`.

## Shape

```json
{
  "contract_version": "0.1",
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

Every candidate file and symbol must reference existing evidence IDs.

## Status Rules

- `ok`: candidates are evidence-backed and limitations are acceptable for the requested use.
- `partial`: some useful source evidence exists, but important evidence is missing.
- `insufficient_evidence`: the bundle cannot support the requested use without guessing.

For edit localization, Phase 2B expects `insufficient_evidence`.

## Missing Evidence

Use `missing_evidence` for facts that would be required before localization:

- symbol references;
- import/export resolution;
- call graph;
- LSP diagnostics;
- source snippets or file context around candidates;
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

- No CLI command in Phase 2B.
- No active edit planning.
- No `where-to-edit` integration.
- No call graph.
- No reference resolution.
- No LSP diagnostics.
- No SQLite persistence.
- No MCP tools.
