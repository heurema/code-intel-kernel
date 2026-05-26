# Context Pack JSON Contract Draft

Status: draft-only contract. No runtime output currently implements this shape.

Contract identity: `context_pack = "draft-0.1"`.

## Command Sketch

These commands are design sketches only:

```bash
code-intel context-pack "<query>" --budget fast --format compact --json
code-intel context-pack "<query>" --budget deep --format standard --json
code-intel context-pack "<query>" --budget very-deep --format compact --json
```

## Top-Level Shape

```json
{
  "contract_version": "draft-0.1",
  "query": "...",
  "budget": "fast | deep | very_deep",
  "output_format": "compact | standard | full",
  "status": "ok | partial | insufficient_evidence | unavailable | error",
  "coverage": {
    "level": "high | medium | low | insufficient",
    "covered_layers": [],
    "gaps": []
  },
  "decision_semantics": "not_supported",
  "context_summary": {},
  "context_files": [],
  "observed_symbols": [],
  "existing_capabilities": [],
  "repo_context": [],
  "source_context_selectors": [],
  "source_slices": [],
  "diagnostics": [],
  "verification_commands": [],
  "known_facts": [],
  "unknowns": [],
  "ambiguities": [],
  "missing_evidence": [],
  "convergence": [],
  "disagreements": [],
  "evidence": [],
  "warnings": [],
  "limitations": []
}
```

All top-level fields are required. Empty arrays are valid when a layer is unavailable, out of budget, or unsupported.

## Field Semantics

### contract_version

Always `draft-0.1` for this draft.

### query

The input query string. The query is a context request, not an edit request.

### budget

Allowed values:

- `fast`
- `deep`
- `very_deep`

CLI value `very-deep` maps to JSON value `very_deep`.

### output_format

Allowed values:

- `compact`
- `standard`
- `full`

### status

Allowed values:

- `ok`: requested budget completed within limits;
- `partial`: some evidence was collected, but one or more layers were unavailable, truncated, or ambiguous;
- `insufficient_evidence`: not enough evidence exists to assemble useful context;
- `unavailable`: a required layer for the requested budget is unavailable;
- `error`: validation or runtime failure.

### coverage

Coverage describes how much of the requested context budget is backed by evidence. It is not a decision score and must not be interpreted as edit confidence.

### decision_semantics

Always:

```json
"not_supported"
```

This field is a hard boundary: the kernel is not deciding what to edit, patch, plan, or recommend.

### context_summary

A compact natural-language or structured summary of what evidence was assembled.

It may summarize:

- observed repository areas;
- available read-only layers;
- major missing evidence;
- ambiguity level;
- diagnostic availability.

It must not name an edit target or recommended change.

### context_files

Files that provide context for the query.

Each item should include:

```json
{
  "file_id": "...",
  "path": "...",
  "language": "rust | unknown",
  "source_layer": "repograph | impact | symbolgraph | source_evidence | source_context | lsp_diagnostics",
  "reason": "...",
  "selector_hints": [],
  "evidence_ids": []
}
```

`context_files` are context handles, not relevant-file decisions.

### observed_symbols

Observed source symbols from supported read-only symbol layers.

Each item should include:

```json
{
  "symbol_id": "...",
  "kind": "...",
  "name": "...",
  "path": "...",
  "range": {
    "start_line": 1,
    "end_line": 1
  },
  "source_layer": "symbolgraph | source_evidence | source_context | lsp_diagnostics",
  "reason": "...",
  "evidence_ids": []
}
```

Observed symbols are not target symbols.

### existing_capabilities

Repository capabilities already observed from manifests, commands, components, tests, diagnostics, or source facts.

Examples:

- known test commands;
- known lint or format commands;
- observed component boundaries;
- available diagnostics layer;
- supported source languages.

This field must not recommend reuse or instruct the consumer to call a function.

### repo_context

Repository-level context from RepoGraph and Impact:

- components;
- workspaces;
- command and test context;
- dependency edges;
- impact context;
- structured warnings.

### source_context_selectors

Explicit selector handles that may be passed manually to SourceContext.

Selectors are handles, not edit locations.

### source_slices

Bounded source excerpts. Usually empty in `compact` format.

Every slice must include evidence IDs and deterministic bounds. Slices must respect SourceContext safety and truncation limits.

### diagnostics

Diagnostic excerpts from a read-only diagnostics layer when available.

Diagnostics are observations, not root-cause claims or fixes.

### verification_commands

Commands observed from repository evidence that could be used by a downstream consumer for verification.

Each item should include:

```json
{
  "command_id": "...",
  "command": "...",
  "source": "manifest | script | workflow | repograph",
  "scope": "...",
  "reason": "...",
  "evidence_ids": []
}
```

This field is context. It is not a test plan.

### known_facts

Evidence-backed facts that were observed directly.

### unknowns

Questions the kernel cannot answer with current evidence.

### ambiguities

Multiple plausible interpretations or matches that cannot be disambiguated by current evidence.

### missing_evidence

Evidence categories required for stronger context but absent from the current layers, such as references, definitions, call graph, diagnostics, semantic query relevance, or supported language data.

Missing evidence must not be converted into guesses.

### convergence

Cases where independent evidence channels agree on a contextual fact.

Each item should include:

```json
{
  "convergence_id": "...",
  "fact": "...",
  "evidence_channels": [],
  "evidence_ids": []
}
```

### disagreements

Cases where evidence channels conflict, produce different scopes, or leave ambiguity.

Each item should include:

```json
{
  "disagreement_id": "...",
  "summary": "...",
  "evidence_channels": [],
  "evidence_ids": [],
  "missing_evidence": []
}
```

### evidence

All evidence records referenced by IDs elsewhere in the output.

Evidence records should include:

- `evidence_id`;
- `source_layer`;
- `kind`;
- `path` when applicable;
- bounded range or selector when applicable;
- observed value;
- limitations.

### warnings

Structured warnings from composed layers or Context Pack assembly.

### limitations

Known limits of the assembled pack, including budget truncation, unsupported language coverage, unavailable diagnostics, missing reference layer, and output format omissions.

## Budget and Format Requirements

`fast` must include only RepoGraph, Impact, and SymbolGraph-lite summary context.

`deep` may add SourceEvidence, SourceContext selectors or slices, and LSP diagnostics if available.

`very_deep` may run multiple independent evidence channels and must report `convergence` and `disagreements`.

`compact` should omit source text by default.

`standard` may include bounded source snippets and diagnostic excerpts.

`full` may include complete evidence details within deterministic limits.

## Forbidden Output Semantics

Context Pack output must not contain:

- edit this file;
- edit this symbol;
- target file;
- target symbol;
- apply this patch;
- recommended change;
- root cause;
- use this function;
- modify this function;
- implementation plan;
- PR summary.

No field in this contract should be interpreted as localization. Other agents or humans may use the context to make their own decisions outside the kernel.
