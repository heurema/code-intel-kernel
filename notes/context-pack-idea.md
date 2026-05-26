# Context Pack / Deep Repo Understanding Idea

Status: captured idea. Implementation deferred.

## Summary

Context Pack would be a read-only, evidence-backed, token-efficient context assembly layer for deep repository understanding.

It should package enough structured context for another agent or human to reason faster without making the kernel responsible for decisions.

## Boundary

The kernel must not become:

- an agent;
- an IDE;
- a planner;
- a PR generator;
- an edit-localization tool.

Context Pack must not provide edit locations, edit targets, patches, plans, PRs, root-cause claims, recommended changes, or `use this function` / `modify this symbol` guidance.

`where-to-edit` remains `insufficient_evidence`.

## What It May Assemble

- context files;
- observed symbols;
- existing capabilities;
- repo context;
- bounded source slices or SourceContext selectors;
- diagnostics;
- verification commands;
- known facts;
- unknowns;
- ambiguities;
- missing evidence;
- convergence and disagreement between independent evidence channels.

## Budget Modes

- `fast`: RepoGraph plus Impact plus SymbolGraph-lite summary only.
- `deep`: adds SourceEvidence, SourceContext selectors or slices, and LSP diagnostics if available.
- `very_deep`: runs multiple independent evidence channels and reports convergence/disagreement.

## Output Formats

- `compact`: IDs, short reasons, selector hints, no source text by default.
- `standard`: bounded snippets and diagnostic excerpts.
- `full`: complete evidence details within deterministic limits.

## Draft CLI

Do not implement yet.

```bash
code-intel context-pack "<query>" --budget fast --format compact --json
code-intel context-pack "<query>" --budget deep --format standard --json
code-intel context-pack "<query>" --budget very-deep --format compact --json
```

## Evaluation Ideas

- Measure token budget and output compactness.
- Track evidence coverage by layer.
- Add no-edit-target-language tests.
- Add ambiguity and refusal tests.
- Validate convergence/disagreement reporting.
- Compare against raw file exploration.

## Deferred Implementation Rule

Future implementation should start only after an explicit phase gate. The first slice should be CLI-only, read-only, and contract-gated. It should not add MCP, SQLite, embeddings, call graph, runtime planning, or `where-to-edit` integration.
