# Phase 1C Impact Skeleton

## What changed

- Added RepoGraph relationships to inspect output.
- Added component `file_patterns`.
- Added command and test `scope_ref` values.
- Bumped inspect `contract_version` to `0.2`; impact contract remains `0.1`.
- Added RepoGraph-only `analyze_impact` API.
- Added `code-intel impact` CLI output.
- Added semantic tests for stable inspect ordering and conservative impact behavior.

## Relationship model

Current relationship kinds:

```text
contains
belongs_to_workspace
defines_component
has_command
has_test
tests
depends_on
uses_package_manager
evidence_for
```

Phase 1C mostly uses:

```text
defines_component
has_command
has_test
tests
uses_package_manager
evidence_for
```

All emitted relationships carry an `evidence_id`.

## Impact JSON shape

```text
contract_version
status
changed_files
impacted_components
impacted_workspaces
recommended_commands
recommended_tests
evidence
warnings
limitations
```

`status` is usually `partial` for mapped changes because impact is still RepoGraph-only. Unknown paths can return `insufficient_evidence`.

## Current limitations

- No symbol-level localization.
- No import/reference/call graph traversal.
- File pattern matching is intentionally simple.
- Workspace dependency relationships are shallow.
- Commands are scoped conservatively to repo or detected components.
- Manifest changes broaden impact rather than trying to narrow it.

## Why SymbolGraph/LSP/MCP are still deferred

Impact can already provide useful build/test recommendations from RepoGraph. SymbolGraph and LSP should only be added after RepoGraph impact remains stable across real repositories. MCP should wait until CLI/library contracts are useful and conservative.

## Phase 1D next

- Improve command inference.
- Improve workspace/component relationships.
- Add more build-system fixtures.
- Add dependency edges where manifests expose them clearly.
- Consider storage only after graph output stabilizes.
- Keep SymbolGraph deferred unless RepoGraph impact proves stable.
