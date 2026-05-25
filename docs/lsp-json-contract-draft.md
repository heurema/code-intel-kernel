# LSP JSON Contract Draft

Status: Phase 3A draft family. `lsp_diagnostics` has a Phase 3B-A runtime contract in `docs/lsp-diagnostics-json-contract.md`; the other contracts remain draft-only and are not part of runtime CLI output.

All LSP contract versions in this document use `draft-0.1`.

## Shared Shape

```json
{
  "contract_version": "draft-0.1",
  "status": "ok | partial | unavailable | error",
  "language": "rust",
  "server": {
    "name": "rust-analyzer",
    "version": "unknown"
  },
  "workspace_root": ".",
  "request": {},
  "results": [],
  "evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": []
}
```

## Status

- `ok`: request completed and returned bounded evidence.
- `partial`: request completed but results were truncated, incomplete, or warning-bearing.
- `unavailable`: server binary, workspace, or capability is unavailable.
- `error`: request failed in a structured recoverable way.

## Evidence Rules

Every future LSP fact must be evidence-backed.

Evidence should include:

- source file path;
- line/column range;
- byte range when available;
- LSP method used;
- server identity/version when available;
- workspace root;
- request parameters;
- result source, such as diagnostic or location.

No LSP result should be emitted without evidence.

## lsp_diagnostics

Purpose: collect bounded diagnostics for explicit files or a workspace.

Runtime status: implemented as `lsp_diagnostics` version `0.1`. See `docs/lsp-diagnostics-json-contract.md`.

```json
{
  "contract_version": "draft-0.1",
  "status": "ok",
  "language": "rust",
  "server": { "name": "rust-analyzer", "version": "unknown" },
  "workspace_root": ".",
  "request": {
    "kind": "diagnostics",
    "files": ["src/lib.rs"]
  },
  "results": [
    {
      "diagnostic_id": "lsp-diagnostic-src-lib-rs-10-4",
      "path": "src/lib.rs",
      "range": {
        "start_line": 10,
        "start_column": 4,
        "end_line": 10,
        "end_column": 12
      },
      "severity": "error | warning | information | hint",
      "code": "E0000",
      "source": "rust-analyzer",
      "message": "...",
      "evidence_ids": ["..."]
    }
  ],
  "evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": []
}
```

## lsp_diagnostic_delta

Purpose: compare two bounded diagnostic snapshots.

```json
{
  "contract_version": "draft-0.1",
  "status": "ok",
  "language": "rust",
  "server": { "name": "rust-analyzer", "version": "unknown" },
  "workspace_root": ".",
  "request": {
    "kind": "diagnostic_delta",
    "before_snapshot_id": "before",
    "after_snapshot_id": "after"
  },
  "results": {
    "added": [],
    "removed": [],
    "unchanged": [],
    "summary": {
      "added_count": 0,
      "removed_count": 0,
      "unchanged_count": 0
    }
  },
  "evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": []
}
```

## lsp_definitions

Purpose: resolve definitions for an explicit source position or symbol selector.

```json
{
  "contract_version": "draft-0.1",
  "status": "partial",
  "language": "rust",
  "server": { "name": "rust-analyzer", "version": "unknown" },
  "workspace_root": ".",
  "request": {
    "kind": "definition",
    "path": "src/lib.rs",
    "line": 10,
    "column": 4
  },
  "results": [
    {
      "location_id": "lsp-location-src-lib-rs-1-0",
      "path": "src/lib.rs",
      "range": {
        "start_line": 1,
        "start_column": 0,
        "end_line": 5,
        "end_column": 1
      },
      "evidence_ids": ["..."]
    }
  ],
  "evidence": [],
  "warnings": [],
  "limitations": ["Definitions are not edit locations."],
  "missing_evidence": []
}
```

## lsp_references

Purpose: find bounded references for an explicit source position or symbol selector.

```json
{
  "contract_version": "draft-0.1",
  "status": "partial",
  "language": "rust",
  "server": { "name": "rust-analyzer", "version": "unknown" },
  "workspace_root": ".",
  "request": {
    "kind": "references",
    "path": "src/lib.rs",
    "line": 10,
    "column": 4,
    "include_declaration": true
  },
  "results": [
    {
      "location_id": "lsp-reference-src-main-rs-20-8",
      "path": "src/main.rs",
      "range": {
        "start_line": 20,
        "start_column": 8,
        "end_line": 20,
        "end_column": 24
      },
      "kind": "reference | declaration",
      "evidence_ids": ["..."]
    }
  ],
  "evidence": [],
  "warnings": [],
  "limitations": ["References are evidence, not edit targets."],
  "missing_evidence": []
}
```

## lsp_symbol_resolution

Purpose: combine definition/reference/document-symbol facts for explicit symbol disambiguation.

```json
{
  "contract_version": "draft-0.1",
  "status": "partial",
  "language": "rust",
  "server": { "name": "rust-analyzer", "version": "unknown" },
  "workspace_root": ".",
  "request": {
    "kind": "symbol_resolution",
    "symbol_id": "symbol-rust-src-lib-rs-function-render-..."
  },
  "results": {
    "symbol_id": "...",
    "definitions": [],
    "references": [],
    "diagnostics": [],
    "disambiguation": "ambiguous | resolved | insufficient_evidence"
  },
  "evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": []
}
```

## Warning Categories

Draft categories:

- `server_unavailable`
- `capability_unavailable`
- `workspace_open_failed`
- `request_timeout`
- `path_outside_repo`
- `ignored_path`
- `symlink_ignored`
- `result_limit_exceeded`
- `server_error`
- `unsupported_language`
- `lsp_not_localization`

## Contract Discipline

These contracts are drafts. Phase 3A must not change existing runtime contracts:

- `inspect`: `0.2`
- `impact`: `0.2`
- `eval`: `0.4`
- `symbols`: `0.1`
- `source_evidence`: `0.3`
- `source_context`: `0.1`
