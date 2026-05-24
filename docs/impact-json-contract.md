# Impact JSON Contract

Status: Phase 1C RepoGraph-only contract draft.

The `impact` command returns conservative repository/build/test-level impact from changed file paths. It does not perform symbol-level analysis.

## Command

```bash
cargo run --quiet -- impact src/main.rs Cargo.toml --json
```

Also accepted:

```bash
cargo run --quiet -- impact --changed-files src/main.rs,Cargo.toml --json
```

## Top-level shape

```json
{
  "contract_version": "0.1",
  "status": "partial",
  "changed_files": [],
  "impacted_components": [],
  "impacted_workspaces": [],
  "recommended_commands": [],
  "recommended_tests": [],
  "evidence": [],
  "warnings": [],
  "limitations": []
}
```

## Status

Known values:

- `ok`: reserved for future fully-supported RepoGraph impact cases.
- `partial`: impact was produced, but it is conservative and RepoGraph-only.
- `insufficient_evidence`: changed files could not be mapped to RepoGraph facts.

Phase 1C usually returns `partial` for mapped changes because SymbolGraph is not implemented.

## Inputs

`changed_files` are repository-relative paths. The CLI accepts either positional paths or a comma-separated `--changed-files` value.

## Outputs

### impacted_components

Components whose `file_patterns` match changed files, or all components when a root manifest/build file changes.

### impacted_workspaces

Workspace entries affected by broad manifest/workspace changes when known.

### recommended_commands

Commands whose scope applies to impacted components or to the repository root.

### recommended_tests

Test targets whose scope applies to impacted components or to changed test paths.

### evidence

Evidence collection copied from `inspect`. Every impacted component, workspace, command, and test should reference an existing `evidence_id`.

### warnings

The report includes warnings from `inspect` plus impact-specific warnings.

Impact-specific categories include:

- `repo_graph_only`: impact is path/build/test-level only.
- `unmapped_change`: a changed path could not be mapped to a component or broad repo fact.

## Conservative behavior

- Source paths are matched against component `file_patterns`.
- Manifest, lockfile, workspace, and build config changes broaden impact.
- Unknown paths return `insufficient_evidence` if nothing else maps.
- Missing or unsupported data becomes a structured warning, not a guess.

## Intentionally Not Included Yet

- Symbol-level impact.
- Import/reference/call graph traversal.
- LSP diagnostics.
- Tree-sitter parsing.
- SQLite persistence.
- MCP tools.
- Embeddings or semantic search.
