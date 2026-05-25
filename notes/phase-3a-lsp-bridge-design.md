# Phase 3A: LSP Bridge Design

Phase 3A is design-only. It commits to the LSP boundary before adding runtime dependencies or spawning language servers.

## Current Evidence Stack

- RepoGraph: repository/build/test facts, commands, impact, and eval.
- SymbolGraph-lite: Rust top-level source facts.
- SourceEvidence: query-linked evidence candidates and SourceContext selector hints.
- SourceContext: bounded read-only source slices for explicit selectors.
- `where-to-edit`: still `insufficient_evidence`.

## Phase 2G Findings

The adversarial gate passed, but it confirmed missing evidence:

- duplicate symbols require semantic disambiguation;
- "who calls X" requires references or call graph evidence;
- unsupported languages should not be guessed;
- source snippets are context, not proof of root cause;
- selector hints are context handles, not edit locations.

## Why LSP Is Next

LSP is the next missing layer because it can provide read-only diagnostics, definitions, references, and document symbols from language servers. Those facts directly address the Phase 2G refusal categories without turning the kernel into a patch planner.

## Why Implementation Is Deferred

Language servers are external processes. They add process lifecycle, timeout, version, path-safety, and nondeterminism risks. A design pass prevents accidental coupling to `where-to-edit`, mutation-capable LSP methods, SQLite persistence, or MCP exposure.

## Documents Added

- `docs/lsp-bridge-design.md`
- `docs/lsp-capability-matrix.md`
- `docs/lsp-json-contract-draft.md`
- `docs/lsp-integration-boundaries.md`
- `docs/lsp-process-safety.md`
- `docs/phase-3b-lsp-diagnostics-reference-plan.md`

## Decision

Proceed to Phase 3B only after this design is accepted.

Recommended Phase 3B: Rust-only, `rust-analyzer` only, read-only diagnostics/definitions/references, structured `unavailable` behavior, no SQLite, no MCP, no `where-to-edit` integration.

## Open Questions

- Should Phase 3B use a mocked LSP transport first or a real `rust-analyzer` process behind optional tests?
- Should diagnostics be workspace-level by default or explicit-file only first?
- Should definition/reference requests use file/position only, or also SymbolGraph-lite symbol IDs?
- What timeout and result limits are strict enough for large repositories?
