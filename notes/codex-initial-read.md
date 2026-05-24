# Codex Initial Read

## Product understanding

Code Intelligence Kernel is a reusable, local-first code intelligence layer for agents. It should expose deterministic repository facts, symbol context, diagnostics, evidence bundles, process-reward signals, and typed session memory through a CLI/library first, with optional read-only MCP later.

The kernel must remain project-agnostic. Goalrail and Punk are consumer profiles and usage examples, not core dependencies. Core behavior should be represented through generic profiles: `strict`, `standard`, `prototype`, `research`, and `custom`.

## Minimum viable implementation

The MVP should start read-only and structural-first:

- RepoGraph scanner for repository shape, package/workspace hints, commands, and test/build/lint candidates.
- SQLite persistence for nodes, edges, commands, diagnostics, and typed events.
- SymbolGraph extraction for TypeScript/JavaScript/Python after RepoGraph.
- Minimal LSP diagnostics bridge after structural extraction works.
- EvidenceBundle and ProcessReward JSON outputs for agent preflight and verification.

## Main risks

- Embeddings-first overbuild before structural facts are useful.
- Encoding Goalrail/Punk names into core contracts instead of using generic profiles.
- Exposing MCP mutation tools or arbitrary shell execution too early.
- Treating Tree-sitter syntax facts as precise semantic truth.
- Relying on natural-language memory instead of typed events.
- Expanding infrastructure to Neo4j, vector DBs, dashboards, or cloud services before local SQLite proves insufficient.

## Files planned to create

- `Cargo.toml`
- `.gitignore`
- `src/lib.rs`
- `src/main.rs`
- `src/core/mod.rs`
- `src/core/repo_graph.rs`
- `src/core/symbol_graph.rs`
- `src/core/evidence.rs`
- `src/core/process_reward.rs`
- `src/core/memory.rs`
- `src/storage/mod.rs`
- `src/storage/sqlite.rs`
- `src/adapters/mod.rs`
- `src/adapters/tree_sitter.rs`
- `src/adapters/lsp.rs`
- `tests/smoke.rs`
- `notes/next-implementation-plan.md`

I also plan to update the contract/docs layer so profiles are generic and consumer-specific Goalrail/Punk mappings stay outside the core.

## Assumptions

- Rust is the right first implementation language for the kernel CLI/library skeleton. TypeScript remains an early target language for repository analysis, not the implementation language.
- No full functionality should be implemented in this pass; stubs should only define boundaries and return explicit placeholder data.
- The bootstrap archive was not present in the project directory, so it was unpacked from `/Users/vi/Downloads/code-intel-kernel-bootstrap.zip`.
- No commit should be made unless explicitly requested.
