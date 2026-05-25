# Extractor Quality

Status: Phase 1E-B RepoGraph extractor baseline.

RepoGraph extraction is repository/build/test-level only. It does not extract symbols, imports, references, call graphs, diagnostics, semantic search results, or edit locations.

## Supported Inputs

Current read-only inputs:

- Cargo: `Cargo.toml`, `Cargo.lock`, default `src/lib.rs`, default `src/main.rs`, explicit `[[bin]]` paths.
- Cargo workspaces: root `workspace.members`, member `Cargo.toml`, explicit member path dependencies.
- Node: `package.json`, package manager lockfiles, `pnpm-workspace.yaml`.
- Python: `pyproject.toml`, `requirements.txt`, `uv.lock`, `poetry.lock`, `pytest.ini`, `tests/`.
- Go: `go.mod`, simple `go.work` members, `*_test.go` as path evidence.
- Command files: `Makefile`, `justfile`.
- Container/workflow hints: Docker Compose files, `Dockerfile`, `.github/workflows/*.yml`.
- Ignored paths: `.git`, `target`, `node_modules`, `dist`, `build`, `.cache`, `.venv`, `__pycache__`, `coverage`.

## Extracted Facts

RepoGraph emits evidence-backed facts for:

- detected files;
- package manager hints;
- workspace roots and members where supported;
- repository-level components;
- command candidates;
- test targets;
- relationships such as `defines_component`, `has_command`, `has_test`, `tests`, `belongs_to_workspace`, `depends_on`, `uses_package_manager`, and `evidence_for`;
- structured warnings.

## Evidence Rules

- Every component, command, test target, package manager, workspace, detected file, and relationship must reference evidence.
- Evidence IDs must be deterministic for the same repository state.
- Unsupported or ambiguous information must produce warnings rather than guessed facts.
- Build/test command extraction must be tied to manifest, target, or build-file evidence.

## Warning Rules

Warnings are structured and use categories such as:

- `malformed_manifest`
- `unreadable_manifest`
- `unsupported_pattern`
- `partial_support`
- `missing_command`
- `ambiguous_detection`
- `ignored_path`
- `no_supported_manifests`

Warnings should not stop inspection unless the caller chooses to treat them as policy failures.

## Command Scope Rules

- Cargo default commands are repo-scoped until finer package scoping is evidence-backed.
- Cargo workspace member components belong to `workspace-cargo`.
- Makefile and justfile commands are emitted only for clear top-level targets: `test`, `check`, `build`, `lint`, `fmt`, and `format`.
- Ambiguous Makefile/justfile target-like lines produce `partial_support` warnings instead of guessed commands.
- Python `pytest` is emitted only when `pyproject.toml`, `requirements.txt`, or `pytest.ini` provides pytest evidence.
- Python `tests/` without pytest evidence produces an ambiguity warning instead of a guessed test command.
- Go modules get `go test ./...` and `go build ./...` from `go.mod` evidence when a module declaration is parsed.
- Go `*_test.go` files can strengthen test-command evidence but are used only as paths, not parsed for source semantics.

## Intentionally Not Extracted Yet

- Full Cargo resolver output.
- `cargo metadata`.
- Shell command bodies from Makefile or justfile recipes.
- Python import/module/package structure.
- Python virtualenvs or installed package inspection.
- Go package graph or `go list` output.
- Go source semantics beyond `*_test.go` path evidence.
- Node workspace package graph.
- Source-level symbols, references, imports, definitions, or call edges.
- LSP diagnostics.
- SQLite persistence.
- MCP tools.
- Embeddings or semantic search.

## Fixture Matrix

Current fixtures cover:

- minimal Rust crate;
- Rust explicit bin target;
- Rust workspace;
- Rust workspace path dependency;
- minimal Node package;
- minimal Python project with tests;
- Python pytest evidence;
- Python ambiguous tests without pytest evidence;
- malformed Python manifest;
- minimal Go module;
- Go module with tests;
- simple Go workspace;
- malformed Go module;
- Makefile project;
- justfile project;
- malformed manifest;
- ignored directories through a runtime temp-dir fixture.

## Acceptance Criteria Before SymbolGraph

SymbolGraph should remain deferred until:

- fixture coverage stays deterministic across repeated runs;
- every graph fact and relationship has valid evidence;
- malformed and partial inputs produce structured warnings without panics;
- RepoGraph impact correctly handles direct, broad, and explicit transitive cases;
- command/test recommendations are conservative enough for build/test planning;
- extractor limitations are explicit in docs and notes.
