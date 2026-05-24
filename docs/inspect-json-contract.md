# Inspect JSON Contract

Status: Phase 1D inspect contract, version 0.2.

The `inspect` command returns repository/build/test-level facts only. It does not perform symbol-level analysis, LSP diagnostics, code search, impact analysis, or MCP/tool routing.

## Command

```bash
cargo run --quiet -- inspect . --json
```

## Top-level shape

```json
{
  "contract_version": "0.2",
  "repo": {},
  "detected_files": [],
  "package_managers": [],
  "workspaces": [],
  "components": [],
  "commands": [],
  "tests": [],
  "relationships": [],
  "evidence": [],
  "warnings": []
}
```

## Fields

### contract_version

Stable contract identifier for consumers.

Current value:

```json
"0.2"
```

Consumers should reject or explicitly tolerate versions they do not understand.

### repo

```json
{
  "root": "/absolute/repo/path",
  "read_only": true
}
```

`inspect` is read-only. It must not mutate the inspected repository.

### detected_files

Files or directories that triggered detection.

```json
{
  "id": "file-cargo-toml",
  "path": "Cargo.toml",
  "kind": "manifest",
  "evidence_id": "evidence-1"
}
```

Known `kind` values include `manifest`, `lockfile`, `workspace_config`, `build_config`, `test_config`, `workflow`, `container_config`, and `source_hint`.

### package_managers

Package manager or build tool hints.

```json
{
  "id": "package-manager-cargo",
  "kind": "cargo",
  "name": "cargo",
  "evidence_id": "evidence-1"
}
```

Package manager detection is conservative. Ambiguous detection should produce a warning instead of pretending certainty.

### workspaces

Workspace boundaries found in supported manifests.

```json
{
  "id": "workspace-cargo",
  "name": "cargo-workspace",
  "members": ["crates/core"],
  "evidence_id": "evidence-2"
}
```

### components

Repository-level components such as crates, packages, modules, or generic build projects.

```json
{
  "id": "component-rust-package",
  "name": "code-intel-kernel",
  "kind": "rust_crate",
  "path": ".",
  "file_patterns": ["Cargo.toml", "Cargo.lock", "src/**", "tests/**"],
  "evidence_id": "evidence-2"
}
```

Components are not symbols. A Rust crate or Node package may be a component; a function or class is not included yet.

`file_patterns` are RepoGraph-level path scopes. They are not source-level symbol ownership.

### commands

Commands inferred from manifests or build files.

```json
{
  "id": "cmd-cargo-test",
  "kind": "test",
  "command": "cargo test",
  "scope": ".",
  "scope_ref": "repo",
  "confidence": 0.95,
  "evidence_id": "evidence-1"
}
```

Known `kind` values include `test`, `lint`, `build`, `check`, `format`, `typecheck`, and `other`.

### tests

Test targets only.

```json
{
  "id": "test-cargo-test",
  "name": "cargo test",
  "command": "cargo test",
  "scope": ".",
  "scope_ref": "repo",
  "confidence": 0.95,
  "evidence_id": "evidence-1"
}
```

If no test command is supported or confidently inferred, `inspect` should emit a structured warning instead of inventing a target.

### relationships

Minimal RepoGraph edges between repository-level facts.

```json
{
  "id": "relationship-hascommand-repo-cmd-cargo-test",
  "kind": "has_command",
  "src_id": "repo",
  "dst_id": "cmd-cargo-test",
  "evidence_id": "evidence-1"
}
```

Known `kind` values include `contains`, `belongs_to_workspace`, `defines_component`, `has_command`, `has_test`, `tests`, `depends_on`, `uses_package_manager`, and `evidence_for`.

Relationships are still RepoGraph-level. They do not represent symbol references, imports, calls, or type relationships.

### evidence

Every graph fact must point at evidence.

```json
{
  "id": "evidence-1",
  "path": "Cargo.toml",
  "kind": "manifest",
  "field": "package.name",
  "reason": "Cargo package name."
}
```

Evidence IDs are deterministic for the same repository state. They are not UUIDs, timestamps, or random values.

### warnings

Warnings are structured.

```json
{
  "id": "warning-1",
  "severity": "warning",
  "category": "malformed_manifest",
  "message": "Failed to parse package.json: ...",
  "path": "package.json",
  "evidence_id": "evidence-1"
}
```

Known `severity` values:

- `info`
- `warning`
- `error`

Known `category` values:

- `ambiguous_detection`
- `ignored_path`
- `malformed_manifest`
- `missing_command`
- `no_supported_manifests`
- `partial_support`
- `unreadable_manifest`
- `unsupported_pattern`

Warnings should be used for unreadable manifests, malformed manifests, unsupported or partially supported patterns, missing command definitions, ambiguous package/workspace detection, unclear test commands, and ignored generated/build/cache directories.

Non-critical warnings must not fail inspection.

## Invariants

- `contract_version` must be present.
- Top-level fields must be present even when arrays are empty.
- `repo.read_only` must be `true`.
- Every `detected_files[]`, `package_managers[]`, `workspaces[]`, `components[]`, `commands[]`, and `tests[]` entry must have a non-empty `evidence_id`.
- Every `relationships[]` entry must have a non-empty `evidence_id`.
- Every referenced `evidence_id` must exist in `evidence[]`.
- Evidence IDs and warning IDs must be deterministic for the same repository state.
- Missing or unsupported information should produce structured warnings, not guesses.
- `inspect` must remain repository/build/test-level only until SymbolGraph exists.

## Minimal Rust Example

```json
{
  "contract_version": "0.2",
  "repo": {
    "root": "/repo",
    "read_only": true
  },
  "detected_files": [
    {
      "path": "Cargo.toml",
      "kind": "manifest",
      "evidence_id": "evidence-1"
    }
  ],
  "package_managers": [
    {
      "kind": "cargo",
      "name": "cargo",
      "evidence_id": "evidence-1"
    }
  ],
  "workspaces": [],
  "components": [
    {
      "id": "component-rust-package",
      "name": "example",
      "kind": "rust_crate",
      "path": ".",
      "file_patterns": ["Cargo.toml", "Cargo.lock", "src/**", "tests/**"],
      "evidence_id": "evidence-2"
    }
  ],
  "commands": [
    {
      "id": "cmd-cargo-test",
      "kind": "test",
      "command": "cargo test",
      "scope": ".",
      "scope_ref": "repo",
      "confidence": 0.95,
      "evidence_id": "evidence-1"
    }
  ],
  "tests": [
    {
      "id": "test-cargo-test",
      "name": "cargo test",
      "command": "cargo test",
      "scope": ".",
      "scope_ref": "repo",
      "confidence": 0.95,
      "evidence_id": "evidence-1"
    }
  ],
  "relationships": [],
  "evidence": [
    {
      "id": "evidence-1",
      "path": "Cargo.toml",
      "kind": "manifest",
      "field": null,
      "reason": "Rust Cargo manifest detected."
    }
  ],
  "warnings": []
}
```

## Intentionally Not Included Yet

- SymbolGraph facts: functions, classes, imports, references, exports.
- Tree-sitter parsing.
- LSP diagnostics, definitions, references, hovers, or type info.
- SQLite persistence.
- MCP tools.
- Embeddings or semantic search.
- Graph algorithms or impact analysis beyond direct RepoGraph facts.
- `where-to-edit` file recommendations.
