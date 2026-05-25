# Source Evidence JSON Contract

Status: Phase 2D prototype contract, version `0.2`.

`source-evidence` assembles evidence from RepoGraph and SymbolGraph-lite for a query. It is read-only and does not identify edit locations.

Phase 2E adds `source-context` as a separate explicit-selector slicing layer. `source-evidence` output does not include snippets by default.

## Command

```bash
cargo run --quiet -- source-evidence "parse repo graph" --json
```

With an explicit fixture or repository:

```bash
cargo run --quiet -- source-evidence "top_level_function" --repo tests/fixtures/rust-symbols-basic --json
```

## Top-level Shape

```json
{
  "contract_version": "0.2",
  "status": "ok | partial | insufficient_evidence",
  "query": "...",
  "confidence": "high | medium | low | insufficient",
  "candidate_files": [],
  "candidate_symbols": [],
  "repo_context": [],
  "source_evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": [],
  "refusal_reason": "..."
}
```

## Semantics

This contract packages evidence candidates. It does not say what to edit.

Allowed statements:

- a source file is an evidence candidate;
- a top-level source symbol is an evidence candidate;
- a RepoGraph component, command, or test may provide context;
- evidence is insufficient for localization.

Forbidden statements:

- edit this file;
- change this symbol;
- this is the correct location;
- apply this patch.

## Matching Strategy

Phase 2D uses deterministic local matching only:

- case-insensitive substring match on source file paths and symbol names;
- token overlap between query terms and file/symbol names;
- simple path-based RepoGraph component context.

No embeddings, LLM scoring, fuzzy semantic inference, references, imports, call graph, LSP, SQLite, or MCP are used.

## candidate_files

Each candidate file has:

- `path`
- `language`
- `parse_status`
- `confidence`
- `reason`
- `evidence_ids`

Every evidence ID must exist in `source_evidence`.

## candidate_symbols

Each candidate symbol has:

- `symbol_id`
- `kind`
- `name`
- `path`
- `range`
- `confidence`
- `reason`
- `evidence_ids`

Only Rust top-level SymbolGraph-lite facts are available in Phase 2D.

## repo_context

Repo context is attached only when path evidence supports it.

Current kinds:

- `component`
- `command`
- `workspace`
- `test`

Each context item includes a `role`:

- `containing_component`
- `containing_workspace`
- `verification_command_context`
- `test_command_context`
- `dependency_context`
- `impact_context`
- `ambiguous_context`

Repo context items are validation context, not edit instructions.

## warnings

Known categories:

- `ambiguous_query`
- `candidate_limit_exceeded`
- `insufficient_evidence_for_localization`
- `localization_not_supported`
- `multiple_candidates`
- `no_repo_component_context`
- `no_matching_source_files`
- `no_matching_source_symbols`
- `parse_error_present`
- `query_too_broad`
- `repo_graph_context_unavailable`
- `symbol_graph_parse_warning`
- `unsupported_language`

## missing_evidence

Known categories:

- `ambiguous_source_match`
- `candidate_limit_exceeded`
- `localization_not_supported`
- `no_call_graph`
- `no_lsp_diagnostics`
- `no_repo_component_context`
- `no_source_match`
- `no_symbol_reference_layer`
- `parse_error_present`
- `query_relevance`
- `query_too_broad`
- `unsupported_language`

Missing evidence is first-class output. It should not be converted into guesses.

## Contract Versions

- `source_evidence`: `0.2`
- `source_context`: `0.1`
- `inspect`: unchanged, `0.2`
- `impact`: unchanged, `0.2`
- `symbols`: unchanged, `0.1`
- `eval`: `0.4`

## Candidate Limits

Phase 2D caps output deterministically:

- max candidate files: 8
- max candidate symbols: 12
- max repo context items: 12

When a limit is exceeded, output is truncated after deterministic ranking and includes `candidate_limit_exceeded`.

## Ranking

Ranking is deterministic:

- exact symbol name match;
- exact file path match;
- substring match;
- token overlap;
- stable tie-break by path, name, and ID.
