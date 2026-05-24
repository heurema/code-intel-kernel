# Impact JSON Contract

Status: Phase 1D RepoGraph-only contract draft.

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
  "contract_version": "0.2",
  "status": "partial",
  "impact_scope": "targeted",
  "confidence": "medium",
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

Phase 1D still returns `partial` for mapped changes because impact is conservative and RepoGraph-only.

## Impact scope

Known values:

- `targeted`: changed paths mapped to one or more components.
- `broad`: a manifest, lockfile, workspace config, or build config changed.
- `mixed`: targeted impact plus transitive RepoGraph impact.
- `unknown`: no supported mapping was found.

## Confidence

Known values:

- `high`: direct mapping and scoped command/test evidence are available.
- `medium`: direct or transitive component mapping exists, but command/test recommendations are generic or repo-scoped.
- `low`: only broad repository-level command evidence is available.
- `insufficient`: no supported mapping was found.

## Inputs

`changed_files` are repository-relative paths. The CLI accepts either positional paths or a comma-separated `--changed-files` value.

## Outputs

### impacted_components

Components whose `file_patterns` match changed files, or all components when a root manifest/build file changes.

Each item has:

- `component_id`
- `name`
- `kind`
- `path`
- `impact_kind`: `direct | transitive | broad | uncertain`
- `distance`: `0` for direct, positive for reverse dependency traversal, or `null` for broad/uncertain impact.
- `reason`: stable machine-readable reason string.
- `evidence_ids`: existing evidence IDs from the copied `evidence` collection.

### impacted_workspaces

Workspace entries affected by broad manifest/workspace changes when known.

Each item has `workspace_id`, `name`, `impact_kind`, `distance`, `reason`, and `evidence_ids`.

### recommended_commands

Commands whose scope applies to impacted components or to the repository root.

Each item has:

- `command_id`
- `command`
- `kind`
- `scope_ref`
- `rank`
- `reason`
- `confidence`
- `evidence_ids`

Ranking is deterministic. For manifest/build changes, check/build/static-analysis commands are ranked earlier than in targeted source changes. For targeted changes, scoped commands rank before repo-scoped commands when scopes are available.

### recommended_tests

Test targets whose scope applies to impacted components or to changed test paths.

Each item has `test_id`, `command`, `scope_ref`, `rank`, `reason`, `confidence`, and `evidence_ids`.

### evidence

Evidence collection copied from `inspect`. Every impacted component, workspace, command, and test should reference one or more existing `evidence_ids`.

### warnings

The report includes warnings from `inspect` plus impact-specific warnings.

Impact-specific categories include:

- `repo_graph_only`: impact is path/build/test-level only.
- `unmapped_change`: a changed path could not be mapped to a component or broad repo fact.
- `partial_support`: dependency traversal or build-system extraction was not fully available.
- `missing_command`: an impacted component had no known test target.

## Relationships and traversal

Phase 1D can follow explicit RepoGraph `depends_on` relationships in reverse to compute transitive impacted components.

Currently emitted relationship kinds:

- `defines_component`
- `has_command`
- `has_test`
- `tests`
- `belongs_to_workspace`
- `depends_on` for explicit Cargo workspace path dependencies
- `uses_package_manager`
- `evidence_for`

Defined but not broadly emitted yet:

- `contains`

No relationship is emitted without evidence.

## Conservative behavior

- Source paths are matched against component `file_patterns`.
- Manifest, lockfile, workspace, and build config changes broaden impact.
- Reverse dependency traversal is used only when explicit `depends_on` edges exist.
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
