# Source Evidence JSON Contract

Status: Phase 2G validated prototype contract, version `0.3`.

`source-evidence` assembles evidence from RepoGraph and SymbolGraph-lite for a query. It is read-only and does not identify edit locations.

Phase 2E adds `source-context` as a separate explicit-selector slicing layer. Phase 2F adds explicit SourceContext selector hints to `source-evidence`. Phase 2G adds adversarial eval cases for ambiguity and refusal behavior without changing the JSON shape. `source-evidence` output does not include snippets by default.

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
  "contract_version": "0.3",
  "status": "ok | partial | insufficient_evidence",
  "query": "...",
  "confidence": "high | medium | low | insufficient",
  "candidate_files": [],
  "candidate_symbols": [],
  "source_context_selectors": [],
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
- an explicit source-context selector can retrieve read-only context for a candidate;
- a RepoGraph component, command, or test may provide context;
- evidence is insufficient for localization.

Forbidden statements:

- edit this file;
- change this symbol;
- this is the correct location;
- apply this patch.

## Matching Strategy

Phase 2F uses deterministic local matching only:

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

Only Rust top-level SymbolGraph-lite facts are available in the current runtime.

## source_context_selectors

Selector hints are explicit handles for `source-context`.

Current selector kinds:

- `file`
- `file_range`
- `symbol_id`

Each selector hint includes:

- `selector_id`
- `selector_kind`
- `file_path`
- `symbol_id`, `symbol_name`, and `symbol_kind` when available
- `start_line` and `end_line` when available
- `reason`
- `confidence`
- `evidence_ids`
- `limitations`

Selector hints are generated only from evidence-backed `candidate_files` and `candidate_symbols`. They are not edit targets and do not include source text.

Current max selector hints: 12.

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
- `selector_hint_limit_exceeded`
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

- `source_evidence`: `0.3`
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
- max source context selector hints: 12

When a candidate/context limit is exceeded, output is truncated after deterministic ranking and includes `candidate_limit_exceeded`. When selector hints are capped, output includes `selector_hint_limit_exceeded`.

## Ranking

Ranking is deterministic:

- exact symbol name match;
- exact file path match;
- substring match;
- token overlap;
- stable tie-break by path, name, and ID.
