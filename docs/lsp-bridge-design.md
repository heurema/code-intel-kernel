# LSP Bridge Design

Status: Phase 3A design draft. Not implemented.

Phase 3A designs a read-only LSP diagnostics/reference bridge. It does not add LSP dependencies, spawn language servers, add SQLite, expose MCP, or change `where-to-edit`.

## Motivation

Phase 2G showed that the current evidence stack can refuse safely, but it cannot resolve the highest-risk source questions:

- duplicate same-name symbols in different files;
- "who calls X" and "where is X used" queries;
- definition and reference disambiguation;
- diagnostics and before/after diagnostic deltas;
- semantic type or hover context when syntax facts are not enough.

SymbolGraph-lite is intentionally top-level and syntactic. SourceEvidence can assemble candidates and SourceContext can slice explicit source text, but neither layer should pretend to know references or semantic definitions.

## What LSP Should Solve

The LSP bridge should add evidence-backed, read-only facts from language servers:

- diagnostics for files or a workspace;
- diagnostic deltas between bounded snapshots;
- definitions for explicit file/position selectors;
- references for explicit file/position or symbol selectors;
- document symbols when useful for comparison against SymbolGraph-lite.

These facts can later help disambiguate symbols and explain why evidence is still missing.

## Non-goals

The LSP bridge must not:

- plan edits;
- apply patches;
- format files;
- run code actions;
- rename symbols;
- mutate workspace files;
- replace RepoGraph command/test extraction;
- replace SymbolGraph-lite source discovery;
- embed snippets by default;
- make `where-to-edit` confident by itself;
- expose MCP tools.

## Proposed Architecture

```text
RepoGraph
  -> build/test/repo facts and validation commands

SymbolGraph-lite
  -> fast Rust top-level source facts and deterministic symbol IDs

SourceEvidence
  -> query-linked evidence candidates and missing evidence

SourceContext
  -> bounded read-only snippets for explicit selectors

LSP bridge
  -> diagnostics, definitions, references, and document-symbol facts
     for explicit bounded requests
```

The bridge should be a separate module in a later phase. It should accept explicit selectors and workspace roots, return structured JSON, and degrade to `unavailable` when a language server is missing.

## Proposed Data Contracts

Phase 3B should implement a small contract family:

- `lsp_diagnostics`
- `lsp_diagnostic_delta`
- `lsp_definitions`
- `lsp_references`
- `lsp_symbol_resolution`

All LSP facts should include:

- contract version;
- status;
- language;
- server identity and version when available;
- workspace root;
- request parameters;
- bounded result set;
- evidence;
- warnings;
- limitations;
- missing evidence.

The draft shape is documented in `docs/lsp-json-contract-draft.md`.

## Integration Points

RepoGraph:

- supplies workspace and build/test context;
- remains the source of truth for commands, tests, package managers, and manifests;
- is not replaced by LSP diagnostics.

SymbolGraph-lite:

- supplies source files and top-level symbols;
- can provide explicit file/symbol selectors for LSP requests;
- remains syntactic and deterministic.

SourceEvidence:

- may later include LSP evidence as present or missing evidence;
- must keep selector hints as context handles, not edit locations.

SourceContext:

- can display bounded slices for LSP locations;
- should require explicit file/range selectors.

`where-to-edit`:

- remains `insufficient_evidence` until a later localization-specific readiness gate passes.

## Safety Boundaries

Language servers are external processes. Treat them as slow, noisy, version-dependent, and unavailable by default.

Required boundaries:

- path containment under repository root;
- no symlink escapes;
- ignored/generated directory filtering;
- request timeouts;
- max diagnostics/references per report;
- deterministic ordering of results;
- no mutation methods;
- no code actions;
- no formatting;
- no rename;
- no external network assumption.

## Phased Plan

Phase 3A:

- design only;
- no runtime changes;
- no dependency changes.

Phase 3B candidate:

- Rust only;
- `rust-analyzer` only;
- read-only diagnostics and references for explicit selectors;
- structured `unavailable` when the server is absent;
- no SQLite, MCP, mutation, or `where-to-edit` integration.

Phase 3C:

- LSP eval and adversarial gate.

Phase 3D:

- integrate diagnostics/reference evidence into SourceEvidence as evidence, not localization.

## Risks

- language server availability varies by host;
- server output can be nondeterministic across versions;
- workspace initialization can be slow;
- diagnostics may depend on generated files or build scripts;
- references may be incomplete for malformed projects;
- users may mistake locations for edit targets.

The bridge should prefer `partial` or `unavailable` over guessed facts.
