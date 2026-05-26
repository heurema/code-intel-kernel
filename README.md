# Code Intelligence Kernel

`code-intel-kernel` is a local-first Rust CLI/library for evidence-backed codebase understanding.

It is meant to be a small reusable kernel that exposes structured facts about a repository. Downstream tools can use those facts for navigation, validation, review, and future agent workflows without depending only on grep, chat history, or embeddings.

It is not an IDE, agent, MCP server, vector database, or edit planner.

## Core Ideas

- **RepoGraph:** repository/build/test structure, components, commands, workspaces, and manifest-backed relationships.
- **SymbolGraph-lite:** source-level facts with deterministic IDs and evidence.
- **SourceEvidence:** evidence assembly for queries, not edit localization.
- **SourceContext:** bounded read-only source slices from explicit selectors.
- **Context Pack (draft):** future token-efficient context assembly, not localization or planning.
- **Evaluation:** fixture-based checks for evidence coverage, deterministic output, and refusal behavior.
- **Refusal:** missing evidence is a first-class result, not a reason to guess.

## Usage

```bash
cargo run --quiet -- inspect . --json
cargo run --quiet -- impact src/main.rs Cargo.toml --json
cargo run --quiet -- eval-fixtures --json
cargo run --quiet -- symbols . --json
cargo run --quiet -- source-evidence "parse repo graph" --json
cargo run --quiet -- source-context --file src/lib.rs --json
cargo run --quiet -- where-to-edit "change login validation copy" --profile=strict --json
```

## Contracts and Docs

JSON contracts and phase notes live under `docs/` and `notes/`.

The README intentionally avoids tracking every contract version and phase checkpoint. Use the contract docs as the source of truth when implementing against the CLI output.

Important docs:

- `docs/inspect-json-contract.md`
- `docs/impact-json-contract.md`
- `docs/symbolgraph-json-contract.md`
- `docs/source-evidence-json-contract.md`
- `docs/source-context-json-contract.md`
- `docs/context-pack-design.md`
- `docs/context-pack-json-contract-draft.md`
- `docs/evaluation-harness.md`
- `docs/repo-vs-symbol-graph-boundary.md`

## Boundaries

- Read-only first.
- Local-first.
- Evidence-backed facts over guesses.
- No mutation tools.
- No embeddings-first retrieval.
- No confident edit localization until a dedicated readiness gate proves it is safe.
- Draft designs do not imply runtime implementation.

## Development

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

The project is intentionally small and read-only first. Prefer explicit evidence and structured warnings over guesses.
