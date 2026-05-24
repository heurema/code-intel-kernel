# Phase 1B Contract Hardening

## What changed

- Added `contract_version: "0.1"` to inspect JSON.
- Documented the inspect JSON surface in `docs/inspect-json-contract.md`.
- Hardened warnings into structured records with deterministic IDs, severity, category, message, optional path, and optional evidence reference.
- Added semantic tests for top-level contract fields, valid JSON output, deterministic evidence IDs, evidence-backed facts, structured warnings, malformed manifests, and honest `where-to-edit` behavior.
- Added fixture repositories for minimal Rust, Rust workspace, Node, Python, Go, Makefile, justfile, and malformed manifest cases.

## Current inspect JSON contract

Top-level fields:

```text
contract_version
repo
detected_files
package_managers
workspaces
components
commands
tests
evidence
warnings
```

Every graph fact must carry an `evidence_id`, and every referenced `evidence_id` must exist in `evidence`.

Warnings are structured and use:

```text
id
severity
category
message
path
evidence_id
```

## Current limitations

- Manifest and target parsing is intentionally shallow.
- Makefile and justfile parsing only detects simple top-level targets.
- Go workspace member parsing is not implemented.
- Node package manager detection can be ambiguous without lockfiles and reports that as a warning.
- Python test command detection is limited to pytest config or a Python project with a `tests/` directory.
- No persistent storage exists yet.

## Why SymbolGraph/LSP/MCP remain deferred

Phase 1B is about making repository/build/test facts stable enough for future consumers. SymbolGraph, LSP, and MCP depend on this contract being reliable first. Adding them now would mix repo-level contract stabilization with source-level analysis and tool exposure.

## Phase 1C next

- Improve command inference for RepoGraph only.
- Add better workspace/component relationships.
- Add an impact-analysis skeleton over RepoGraph facts only.
- Add more fixtures for partial and ambiguous manifests.
- Keep SymbolGraph deferred until the inspect contract remains stable under real repositories.
