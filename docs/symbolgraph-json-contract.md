# SymbolGraph JSON Contract

Status: Phase 2A SymbolGraph-lite contract.

The `symbols` command returns source-level facts for Rust source files. It is intentionally narrow and in-memory.

It does not perform edit localization, call graph analysis, reference resolution, import/export resolution, LSP diagnostics, SQLite persistence, MCP serving, or embeddings.

Phase 2B adds fixture evaluation for this contract through `eval-fixtures`, but does not change the `symbols` contract version.

## Command

```bash
cargo run --quiet -- symbols . --json
```

Alias:

```bash
cargo run --quiet -- symbol-graph . --json
```

## Top-level Shape

```json
{
  "contract_version": "0.1",
  "repo": {
    "root": "/abs/path",
    "read_only": true
  },
  "source_files": [],
  "symbols": [],
  "evidence": [],
  "warnings": [],
  "limitations": []
}
```

## source_files

Each source file has:

- `id`: deterministic source file ID.
- `path`: repository-relative path.
- `language`: currently `rust`.
- `parse_status`: `ok | error | unreadable`.
- `evidence_ids`: existing evidence IDs from `evidence`.

## symbols

Each symbol has:

- `id`: deterministic symbol ID.
- `kind`: supported symbol kind.
- `name`: declared symbol name, or deterministic placeholder for `impl_block`.
- `path`: repository-relative source file path.
- `range`: byte and line/column range.
- `evidence_ids`: existing evidence IDs from `evidence`.

Line numbers are 1-based. Columns are 0-based byte columns from Tree-sitter.

## Supported Rust Symbol Kinds

- `function`
- `struct`
- `enum`
- `trait`
- `type_alias`
- `const`
- `static`
- `module`
- `impl_block`

`impl_block` names use deterministic placeholders such as `impl@12:0`.

## Evidence Rules

- Every source file must reference at least one evidence ID.
- Every symbol must reference at least one evidence ID.
- Warning evidence IDs, when present, must reference an existing evidence item.
- Evidence points to source file paths and symbol declaration locations.

No source file or symbol fact should be emitted without evidence.

## Warning Behavior

Warnings are structured:

```json
{
  "id": "warning-parseerror-src-lib-rs",
  "severity": "warning",
  "category": "parse_error",
  "message": "Rust source file parsed with syntax errors; no symbols were extracted.",
  "path": "src/lib.rs",
  "evidence_id": "evidence-src-lib-rs-source-file"
}
```

Known categories:

- `ignored_path`
- `parse_error`
- `symlink_ignored`
- `unreadable_source`

Parse errors do not panic. The source file can still be listed with `parse_status = "error"`, but symbols are not extracted from that file in Phase 2A.

## Deterministic IDs

IDs are based on:

- repository-relative path;
- symbol kind;
- symbol name;
- byte range.

IDs must be stable for the same repository state.

## Ignored Paths

SymbolGraph-lite ignores generated, dependency, build, and cache directories:

- `.git`
- `target`
- `node_modules`
- `dist`
- `build`
- `.cache`
- `.venv`
- `__pycache__`
- `coverage`

Symlinks are not followed.

## Intentionally Not Included

- Nested functions.
- Methods inside `impl` blocks.
- Calls.
- References.
- Imports/exports.
- Macro expansion.
- Generated symbols.
- Semantic type resolution.
- LSP diagnostics.
- SQLite persistence.
- MCP tools.
- Edit localization.
- `where-to-edit` candidates.

## Evaluation

SymbolGraph-lite eval cases use semantic assertions over:

- source files;
- symbol name/kind/path matches;
- forbidden nested or ignored symbols;
- parse warning categories;
- evidence coverage;
- deterministic output.

The eval harness report uses `eval_contract_version = "0.4"` after Phase 2E because it includes `symbol_cases`, `source_evidence_cases`, and `source_context_cases`. The `symbols` JSON contract remains `0.1`.
