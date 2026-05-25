# Source Evidence JSON Contract

Status: Phase 2C prototype contract, version `0.1`.

`source-evidence` assembles evidence from RepoGraph and SymbolGraph-lite for a query. It is read-only and does not identify edit locations.

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
  "contract_version": "0.1",
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

Phase 2C uses deterministic local matching only:

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

Only Rust top-level SymbolGraph-lite facts are available in Phase 2C.

## repo_context

Repo context is attached only when path evidence supports it.

Current kinds:

- `component`
- `command`
- `test`

Repo context items are validation context, not edit instructions.

## warnings

Known categories:

- `ambiguous_query`
- `insufficient_evidence_for_localization`
- `multiple_candidates`
- `no_matching_source_files`
- `no_matching_source_symbols`
- `repo_graph_context_unavailable`
- `symbol_graph_parse_warning`
- `unsupported_language`

## missing_evidence

Known categories:

- `lsp_diagnostics`
- `localization_evaluation`
- `query_relevance`
- `reference_graph`
- `repo_context`
- `source_match`

Missing evidence is first-class output. It should not be converted into guesses.

## Contract Versions

- `source_evidence`: `0.1`
- `inspect`: unchanged, `0.2`
- `impact`: unchanged, `0.2`
- `symbols`: unchanged, `0.1`
- `eval`: `0.3` when source-evidence eval cases are present
