# Phase 1E-B Python and Go Extraction

## Scope

This phase improves Python and Go RepoGraph extraction only. It does not add SymbolGraph, Tree-sitter, LSP, SQLite, MCP, embeddings, workspace split, or xtask.

## Python Changes

- Extract Python project names from `pyproject.toml`.
- Keep package manager hints evidence-backed through `requirements.txt`, `uv.lock`, `poetry.lock`, and Poetry config.
- Detect pytest evidence from:
  - `tool.pytest` in `pyproject.toml`;
  - `project.dependencies`;
  - `project.optional-dependencies`;
  - Poetry dev dependency tables;
  - `requirements.txt`;
  - `pytest.ini`.
- Emit `pytest` command and test target only when pytest evidence exists.
- Emit an ambiguity warning when `tests/` exists but no pytest evidence exists.
- Keep malformed `pyproject.toml` as a structured warning without panics.

## Go Changes

- Extract module name from `go.mod`.
- Emit `go test ./...` and `go build ./...` when `go.mod` has a module declaration.
- Detect `*_test.go` files as path evidence and use that evidence for the Go test target when present.
- Parse simple `go.work` `use` members.
- Treat `go.mod` without a module declaration as malformed and avoid fake components/commands.

## Contract Versions

- `inspect`: remains `0.2`; no JSON shape change.
- `impact`: remains `0.2`; no JSON shape change.

## Current Limitations

- No Python imports, modules, package discovery, or virtualenv inspection.
- No Python or Go tooling is executed.
- No Go package graph, `go list`, or source semantics.
- Go workspace member modules are listed but not resolved into component graphs.
- `where-to-edit` remains `insufficient_evidence`.

## Phase 1F Next

- Add a lightweight evaluation harness for inspect/impact quality.
- Score fixture expectations for evidence coverage, warnings, commands/tests, and impact scope.
- Track false broad versus false narrow impact behavior.
- Keep SymbolGraph deferred until RepoGraph quality is measurable and stable.
