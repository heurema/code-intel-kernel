# Phase 2A SymbolGraph-lite Plan

Status: draft plan only. Do not implement in Phase 1G.

## Goal

Add the first source-level graph layer without disturbing the stable RepoGraph inspect/impact/eval contracts.

Phase 2A should prove that source facts can be extracted deterministically and with evidence before using them for edit localization.

The previous public SymbolGraph placeholder has been removed from the library exports. Phase 2A should introduce a real internal model rather than extending the old stub.

## Scope

Preferred narrow path:

- start with Rust source files because the repository itself is Rust;
- discover source files from RepoGraph component `file_patterns` and repository traversal;
- extract top-level symbols only;
- expose an internal API, not a public MCP server;
- add SymbolGraph fixture eval cases.

If Rust parsing is too large for the first slice, use a language-agnostic source-file graph with a symbol stub, but keep the same evidence rules.

## Initial Domain Model

Suggested minimal model:

- `SymbolGraph`
- `SourceFile`
- `Symbol`
- `SymbolKind`
- `SymbolEvidence`
- `SymbolWarning`

Initial symbol kinds:

- `function`
- `struct`
- `enum`
- `trait`
- `impl`
- `module`
- `unknown`

## Extraction Rules

- Every source fact must reference evidence.
- Symbol IDs must be deterministic for the same repository state.
- Source file traversal must be sorted.
- Parse failures must produce structured warnings.
- Unsupported syntax must not create guessed symbols.
- SymbolGraph should attach files/symbols to RepoGraph components only when path evidence supports it.

## Known Risks To Cover

- Source files with parse errors.
- Generated code and build artifacts.
- Symlink/path-containment edge cases.
- Large repositories where fixture-scale traversal assumptions may fail.
- Mixed-language repositories where source ownership must remain tied to RepoGraph component evidence.

## Deferred

- Call graph.
- Reference graph.
- Type resolution.
- LSP diagnostics.
- SQLite persistence.
- MCP tools.
- Embeddings.
- Cross-language symbol linking.
- Confident `where-to-edit` localization.

## Acceptance Criteria

- Existing RepoGraph eval remains green.
- SymbolGraph has dedicated fixture cases.
- Symbol facts are evidence-backed.
- Symbol IDs are deterministic.
- Parse failures produce warnings, not panics.
- No fake edit localization is produced.
- `where-to-edit` remains `insufficient_evidence` unless a separate evaluated localization layer exists.

## Contract Discipline

Do not change `inspect`, `impact`, or `eval` contract versions for internal SymbolGraph work unless their JSON output shape changes.

If a public SymbolGraph JSON output is added later, give it its own contract version.
