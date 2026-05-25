# Phase 2A SymbolGraph-lite

## Scope

This phase adds an in-memory SymbolGraph-lite layer for Rust top-level source facts.

It does not add LSP, SQLite, MCP, embeddings, call graph, reference resolution, import/export resolution, workspace split, xtask, or confident `where-to-edit`.

## What Changed

- Added `src/core/symbol_graph.rs`.
- Added `SYMBOLS_CONTRACT_VERSION = "0.1"`.
- Added `code-intel symbols <repo-path> --json`.
- Added Tree-sitter Rust parsing through `tree-sitter` and `tree-sitter-rust`.
- Added Rust source fixtures for top-level symbols, malformed source, and ignored directories.
- Added semantic tests for source files, symbols, evidence, deterministic IDs, parse warnings, ignored paths, and CLI JSON.

## Supported Facts

SymbolGraph-lite currently extracts:

- Rust source files;
- top-level functions;
- structs;
- enums;
- traits;
- type aliases;
- consts;
- statics;
- modules;
- impl blocks with deterministic placeholder names.

## Unsupported Facts

SymbolGraph-lite does not extract:

- nested functions;
- methods inside impl blocks;
- call graph;
- references;
- imports/exports;
- macro-expanded symbols;
- semantic type facts;
- test coverage;
- edit locations.

## Contract Versions

- `inspect`: `0.2`
- `impact`: `0.2`
- `eval`: `0.1`
- `symbols`: `0.1`

## Current Limitations

- Rust only.
- Top-level declarations only.
- Parse errors suppress symbol extraction for the affected file.
- Symlinks are ignored rather than followed.
- No SymbolGraph eval harness integration yet; Phase 2A uses direct tests.
- `where-to-edit` remains `insufficient_evidence`.

## Phase 2B Recommendation

Add source-level evidence bundles over evaluated SymbolGraph-lite facts.

Do not add confident edit localization until:

- SymbolGraph fixtures cover more Rust syntax and parse failures;
- RepoGraph eval remains green;
- source-level facts have dedicated eval coverage;
- false localization risk is explicitly measured.
