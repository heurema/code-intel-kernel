# LSP Integration Boundaries

Status: Phase 3A design draft. Not implemented.

The LSP bridge should be a new read-only evidence layer. It should not absorb responsibilities from existing layers.

## RepoGraph Boundary

RepoGraph owns:

- manifests;
- package managers;
- workspaces;
- components;
- build/check/lint/format/test commands;
- command/test scopes;
- dependency edges extracted from manifests;
- build/test-level impact.

LSP may use RepoGraph context to choose a workspace root or explain validation commands later, but it must not replace RepoGraph extraction.

## SymbolGraph-lite Boundary

SymbolGraph-lite owns:

- Rust source file discovery;
- top-level Rust declarations;
- parse status;
- deterministic symbol IDs;
- source-level evidence ranges.

LSP may use SymbolGraph-lite file/symbol selectors for explicit definition/reference requests, but SymbolGraph-lite remains syntactic and does not claim semantic resolution.

## SourceEvidence Boundary

SourceEvidence owns:

- query-linked source candidates;
- source-to-repo context;
- selector hints;
- missing evidence and refusal status.

Later phases may add LSP diagnostics/reference facts to SourceEvidence as evidence. They must not turn candidates into edit locations.

## SourceContext Boundary

SourceContext owns:

- bounded source slices;
- explicit file/range selectors;
- explicit symbol ID selectors;
- path safety and truncation.

SourceContext may later display LSP locations through explicit selectors. It should not accept natural-language localization queries.

## where-to-edit Boundary

`where-to-edit` remains refusal-only until a later localization readiness gate passes.

LSP facts alone are insufficient for confident localization. A future localization layer would need:

- query relevance evidence;
- definition/reference evidence;
- diagnostics evidence where relevant;
- source context;
- RepoGraph validation context;
- negative/adversarial localization eval cases;
- false broad/false narrow localization metrics.

## Forbidden Couplings

Do not:

- call LSP from `where-to-edit` in Phase 3A;
- call LSP from SourceEvidence in Phase 3A;
- embed snippets in LSP reports by default;
- run formatting/code actions/rename;
- expose LSP through MCP before CLI contracts stabilize;
- store LSP output in SQLite before in-memory contracts stabilize.
