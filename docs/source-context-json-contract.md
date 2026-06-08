# Source Context JSON Contract

Status: Phase 2G validated prototype contract, version `0.1`.

`source-context` returns bounded, read-only source snippets for explicit selectors. It is not natural-language localization and does not identify edit targets.

Phase 2F allows `source-evidence` to emit selector hints that can be passed manually to `source-context`. Phase 2G adds adversarial path-safety and malformed-source eval cases without changing the JSON shape. `source-context` itself remains explicit-selector only.

## Commands

```bash
cargo run --quiet -- source-context --file src/lib.rs --json
cargo run --quiet -- source-context --file src/lib.rs --lines 1:80 --json
cargo run --quiet -- source-context --symbol-id <symbol-id> --json
```

Fixtures or alternate repositories can be selected with `--repo <path>`.

## Top-level Shape

```json
{
  "contract_version": "0.1",
  "status": "ok | partial | insufficient_evidence",
  "selectors": [],
  "slices": [],
  "evidence": [],
  "warnings": [],
  "limitations": []
}
```

## Selectors

Supported selectors are explicit only:

- `file`: repository-relative Rust source path, with optional `line_range`.
- `symbol_id`: deterministic SymbolGraph-lite symbol ID.

No natural-language query selection is supported.

SourceEvidence selector hints map to these existing selectors:

- `selector_kind = "file"` -> `source-context --file <file_path>`
- `selector_kind = "symbol_id"` -> `source-context --symbol-id <symbol_id>`

## Slice Shape

Each slice includes:

- `slice_id`
- `file_path`
- `language`
- `reason`
- `symbol_id`, `symbol_name`, and `symbol_kind` when selected by symbol
- `start_line`, `end_line`
- `start_byte`, `end_byte`
- `context_before_lines`, `context_after_lines`
- `text`
- `truncated`
- `content_hash`
- `evidence_ids`

Every slice must reference evidence in the report-level `evidence` array.

## Limits

Current limits are conservative:

- max slices per report: 8
- max lines per slice: 80
- max bytes per slice: 8000
- max total bytes per report: 20000

When a limit is hit, the report truncates deterministically and emits `slice_truncated` or `source_context_limit_exceeded`.

## Safety

SourceContext enforces repository containment and refuses:

- path traversal outside the repo;
- ignored/generated/cache directories;
- symlink selectors;
- missing files;
- non-UTF8 files;
- binary-looking files;
- unsupported non-Rust files.

Warnings are structured. Non-critical issues do not panic.

## Warning Categories

- `ambiguous_symbol_selector`
- `binary_file`
- `ignored_path`
- `missing_file`
- `non_utf8_file`
- `path_outside_repo`
- `slice_truncated`
- `source_context_limit_exceeded`
- `source_context_not_localization`
- `symbol_not_found`
- `symlink_ignored`
- `unsupported_language`

## Non-goals

SourceContext must not say:

- edit this file;
- edit here;
- apply this patch;
- this is the correct edit location;
- this symbol is the root cause.

It does not use LSP, SQLite, MCP, embeddings, call graph, references, or import/export resolution.

## Contract Versions

- `source_context`: `0.1`
- `source_evidence`: `0.3` after Phase 2F selector hints
- `symbols`: unchanged, `0.1`
- `inspect`: unchanged, `0.2`
- `impact`: unchanged, `0.2`
- `eval`: `0.5` with LSP diagnostics eval cases; `0.4` when source-context eval cases were added
