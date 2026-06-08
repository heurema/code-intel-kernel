# Phase 3B LSP Diagnostics and References Plan

Status: Phase 3B-A diagnostics implemented. Phase 3B-B definitions/references remain deferred.

## Scope

Phase 3B remains narrow:

- Rust only;
- `rust-analyzer` only;
- read-only;
- in-memory;
- no SQLite;
- no MCP;
- no mutation;
- no `where-to-edit` integration.

## Implemented Phase 3B-A Behavior

Phase 3B-A implements:

- detect `rust-analyzer` availability;
- return structured `unavailable` when missing;
- initialize a contained workspace root;
- collect diagnostics for explicit files;
- expose `lsp-diagnostics` CLI;
- produce JSON with evidence IDs.
- include fixture eval coverage for unavailable-server and path-containment behavior.

## Candidate Phase 3B-B Behavior

Phase 3B-B may implement:

- compute diagnostic delta over two in-memory snapshots if feasible;
- support go-to-definition for explicit file/position selectors;
- support find-references for explicit file/position selectors;
- produce JSON with evidence IDs.

## Candidate CLI

Implemented:

```bash
code-intel lsp-diagnostics --repo . --file src/lib.rs --json
```

Draft only:

```bash
code-intel lsp-definition --repo . --file src/lib.rs --line 10 --column 4 --json
code-intel lsp-references --repo . --file src/lib.rs --line 10 --column 4 --json
```

## Acceptance Criteria

- No panic if `rust-analyzer` is unavailable.
- `unavailable` status is structured and documented.
- Diagnostics are bounded and evidence-backed.
- Definitions are deferred until Phase 3B-B.
- References are deferred until Phase 3B-B.
- Output is deterministic enough for tests.
- No mutation-capable LSP methods are called.
- No source snippets are embedded by default.
- Existing `inspect`, `impact`, `symbols`, `source-evidence`, `source-context`, and `lsp-diagnostics` contracts remain unchanged.
- `eval-fixtures` report shape includes `lsp_diagnostics_cases` at contract `0.5`.
- `where-to-edit` still returns `insufficient_evidence`.

## Test Strategy

Preferred:

- unit tests for request/response normalization;
- tests for path containment and ignored paths;
- tests for unavailable server behavior;
- tests for deterministic sorting;
- fixture tests for unavailable-server and path-containment behavior.

Optional/manual:

- integration smoke with real `rust-analyzer` when available.

Real server availability must not be required for normal `cargo test`.

## Deferred

- multi-language support;
- persistent LSP server sessions;
- SQLite storage;
- MCP exposure;
- SourceEvidence integration;
- `where-to-edit` integration;
- diagnostic delta;
- definitions and references;
- code actions, formatting, rename, workspace edits.
