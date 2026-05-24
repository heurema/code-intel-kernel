# Extraction Notes

## What to extract from repos

### Config/build layer

- package manager;
- workspace roots;
- packages/apps/services;
- build commands;
- lint commands;
- test commands;
- typecheck commands;
- generated/vendor directories;
- CI workflows.

### Source layer

- files;
- imports/exports;
- functions/classes/methods;
- type/interface definitions;
- test files;
- source-test relationships.

### LSP layer

- diagnostics;
- definitions;
- references;
- implementations;
- type/hover info;
- call hierarchy where available.

### Memory layer

- tasks;
- hypotheses;
- rejected hypotheses;
- decisions;
- diagnostics before/after;
- tests run;
- commands selected.

## Extraction priority

```text
1. Config files with deterministic parsing.
2. Tree-sitter for syntax.
3. LSP for semantic precision.
4. FTS snippets for docs/comments.
5. Optional embeddings after structural retrieval works.
```

## Why not embeddings first

Embeddings are useful for natural-language similarity, but they do not reliably answer:

- which package owns this symbol;
- which tests run for this change;
- whether diagnostics improved;
- whether the changed file is inside allowed scope;
- whether a dependency edge comes from a real config.

## Design rule

Every fact should be labeled:

```text
source: config | tree-sitter | lsp | heuristic | memory | external
confidence: 0.0..1.0
evidence: path/symbol/diagnostic/event
```
