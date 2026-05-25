# Phase 3B LSP Diagnostics and References Plan

Status: Phase 3A draft. Phase 3B is not implemented.

## Scope

Phase 3B should be narrow:

- Rust only;
- `rust-analyzer` only;
- read-only;
- in-memory;
- no SQLite;
- no MCP;
- no mutation;
- no `where-to-edit` integration.

## Candidate Behavior

Phase 3B may implement:

- detect `rust-analyzer` availability;
- return structured `unavailable` when missing;
- initialize a contained workspace root;
- collect diagnostics for explicit files or workspace if cheap;
- expose `lsp-diagnostics` CLI;
- compute diagnostic delta over two in-memory snapshots if feasible;
- support go-to-definition for explicit file/position selectors;
- support find-references for explicit file/position selectors;
- produce JSON with evidence IDs.

## Candidate CLI

Draft only:

```bash
code-intel lsp-diagnostics --repo . --json
code-intel lsp-diagnostics --repo . --file src/lib.rs --json
code-intel lsp-definition --repo . --file src/lib.rs --line 10 --column 4 --json
code-intel lsp-references --repo . --file src/lib.rs --line 10 --column 4 --json
```

## Acceptance Criteria

- No panic if `rust-analyzer` is unavailable.
- `unavailable` status is structured and documented.
- Diagnostics are bounded and evidence-backed.
- Definitions are bounded and evidence-backed.
- References are bounded and evidence-backed.
- Output is deterministic enough for tests.
- No mutation-capable LSP methods are called.
- No source snippets are embedded by default.
- Existing `inspect`, `impact`, `eval-fixtures`, `symbols`, `source-evidence`, and `source-context` contracts remain unchanged.
- `where-to-edit` still returns `insufficient_evidence`.

## Test Strategy

Preferred:

- unit tests for request/response normalization;
- tests for path containment and ignored paths;
- tests for unavailable server behavior;
- tests for deterministic sorting;
- fixture tests with mocked LSP responses.

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
- code actions, formatting, rename, workspace edits.
