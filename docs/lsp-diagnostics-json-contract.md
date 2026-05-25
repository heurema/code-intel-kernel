# LSP Diagnostics JSON Contract

Status: Phase 3B-A runtime contract, version `0.1`.

`lsp-diagnostics` is a read-only Rust diagnostics bridge. It may collect diagnostics from `rust-analyzer` when available, and must return structured `unavailable` or `error` JSON when it cannot.

It is not a reference layer, definition layer, edit planner, or localization result.

## Command

```bash
cargo run --quiet -- lsp-diagnostics --file src/lib.rs --json
```

Optional:

```bash
cargo run --quiet -- lsp-diagnostics --repo . --file src/lib.rs --timeout-ms 1500 --max-diagnostics 100 --json
```

The rust-analyzer command defaults to `rust-analyzer`. It can be overridden:

```bash
CODE_INTEL_RUST_ANALYZER=/path/to/rust-analyzer cargo run --quiet -- lsp-diagnostics --file src/lib.rs --json
```

## Top-level Shape

```json
{
  "contract_version": "0.1",
  "status": "ok | partial | unavailable | error",
  "language": "rust",
  "server": {
    "name": "rust-analyzer",
    "command": "rust-analyzer",
    "version": null
  },
  "workspace_root": "...",
  "request": {
    "kind": "diagnostics",
    "files": ["src/lib.rs"],
    "timeout_ms": 1500,
    "max_diagnostics": 100
  },
  "diagnostics": [],
  "evidence": [],
  "warnings": [],
  "limitations": [],
  "missing_evidence": []
}
```

## Status

- `ok`: rust-analyzer completed diagnostics collection within bounds.
- `partial`: diagnostics were collected, but warnings such as truncation or timeout occurred.
- `unavailable`: rust-analyzer is not available; this is valid behavior.
- `error`: request validation or server interaction failed in a structured way.

## Diagnostic Shape

```json
{
  "diagnostic_id": "lsp-diagnostic-src-lib-rs-10-4-message",
  "file_path": "src/lib.rs",
  "start_line": 10,
  "start_character": 4,
  "end_line": 10,
  "end_character": 12,
  "severity": "error | warning | information | hint | unknown",
  "code": "E0000",
  "source": "rust-analyzer",
  "message": "...",
  "evidence_ids": ["..."]
}
```

Lines are 1-based. Characters are LSP character offsets.

## Evidence Rules

Every emitted diagnostic must have at least one evidence ID. Every referenced evidence ID must exist in `evidence`.

Evidence records include:

- file path;
- diagnostic range;
- LSP method/event;
- workspace root;
- server name.

## Warning Categories

- `rust_analyzer_unavailable`
- `lsp_diagnostics_unavailable`
- `path_outside_repo`
- `ignored_path`
- `symlink_ignored`
- `missing_file`
- `unsupported_language`
- `request_timeout`
- `result_limit_exceeded`
- `server_error`
- `no_files_requested`
- `lsp_not_localization`

## Safety

The bridge:

- validates workspace root containment;
- rejects paths outside the repo;
- rejects ignored/generated paths;
- rejects symlinks;
- supports bounded timeout;
- supports bounded max diagnostics;
- sorts output deterministically;
- exposes no mutation LSP methods.

## Non-goals

- No definitions.
- No references.
- No hover/type info.
- No call hierarchy.
- No formatting.
- No code actions.
- No rename.
- No MCP.
- No SQLite.
- No `where-to-edit` integration.

Diagnostics are evidence, not fixes, root-cause claims, or edit targets.

## Contract Versions

- `lsp_diagnostics`: `0.1`
- `inspect`: unchanged, `0.2`
- `impact`: unchanged, `0.2`
- `eval`: unchanged, `0.4`
- `symbols`: unchanged, `0.1`
- `source_evidence`: unchanged, `0.3`
- `source_context`: unchanged, `0.1`
