# Phase 1E-A Extractor Quality

## Scope

This phase hardens the existing RepoGraph extractor without adding SymbolGraph, Tree-sitter, LSP, SQLite, MCP, embeddings, workspace split, or xtask.

## What Changed

- Added extractor quality documentation.
- Added Cargo default target detection from `src/lib.rs` and `src/main.rs`.
- Kept explicit Cargo `[[bin]]` target extraction evidence-backed.
- Kept Cargo workspace member detection and path dependency `depends_on` relationships.
- Reworked Makefile and justfile extraction to emit commands only for clear top-level targets:
  - `test`
  - `check`
  - `build`
  - `lint`
  - `fmt`
  - `format`
- Added warnings for ambiguous Makefile/justfile target-like lines.
- Expanded ignored-path handling to include `.git` and `__pycache__`.
- Added a fixture for explicit Cargo bin targets and a runtime temp-dir test for ignored directories.

## Contract Versions

- `inspect`: remains `0.2`; no JSON shape change.
- `impact`: remains `0.2`; no JSON shape change.

## Current Limitations

- No full Cargo resolver or `cargo metadata`.
- Cargo workspace member target extraction is still component-level, not target-level.
- Makefile and justfile parsing is shallow and target-name based only.
- Node, Python, and Go extraction were not expanded in this subphase.
- `where-to-edit` remains an `insufficient_evidence` placeholder.

## Phase 1E-B Next

- Add Python and Go fixture coverage.
- Improve conservative Python/Go command inference where evidence exists.
- Keep SymbolGraph deferred until RepoGraph extraction and impact quality remain stable.
