# Next Implementation Plan

## Phase 0: repo skeleton

- Keep the current Rust CLI/library skeleton minimal and read-only.
- Confirm `cargo test` runs without external service dependencies.
- Keep docs, specs, examples, and notes as the project source of truth.
- Keep Goalrail/Punk as external consumer profiles, not core concepts.

## Phase 1A: RepoGraph MVP

- Parse local repository manifests: `package.json`, workspaces, `tsconfig.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `Makefile`, and GitHub workflow files.
- Detect packages, workspaces, config files, and command candidates in memory.
- Add evidence-backed `code-intel inspect` JSON output.

## Phase 1B: Inspect contract hardening

- Add `contract_version` to inspect output.
- Document `docs/inspect-json-contract.md`.
- Validate every graph fact has evidence.
- Use structured warnings for malformed, ambiguous, unsupported, missing, and ignored inputs.
- Keep `where-to-edit` as `insufficient_evidence`.

## Phase 1C: RepoGraph inference and relationships

- Improve repository command inference without SymbolGraph.
- Add better workspace/component relationships.
- Add impact-analysis skeleton over RepoGraph facts only.
- Add fixtures for partial and ambiguous manifests.
- Do not add Tree-sitter, LSP, SQLite, MCP, embeddings, workspace split, or xtask yet.

## Phase 1D: RepoGraph impact traversal

- Add direct, transitive, broad, and uncertain impact classifications.
- Add reverse dependency traversal over explicit RepoGraph `depends_on` edges.
- Add command/test recommendation ranking, reasons, confidence, and evidence IDs.
- Keep impact analysis RepoGraph-only until behavior is stable on real repositories.
- Keep `where-to-edit` as `insufficient_evidence`.

## Phase 1E: RepoGraph extraction quality

- Improve command inference across Cargo, Node, Python, Go, Make, just, and workflows.
- Improve workspace/component relationships and dependency edges when manifest evidence is explicit.
- Add more fixtures for ambiguous and partially supported build systems.
- Add lightweight benchmark or quality tasks for inspect/impact behavior.
- Consider storage only after the in-memory graph output has proven stable.

## Phase 2: SymbolGraph MVP

- Add Tree-sitter extraction for TypeScript, TSX, JavaScript, and Python.
- Extract imports, exports, functions, classes, interfaces/types, methods, and test files.
- Build local import edges and ranked `where-to-edit` candidates with reasons.
- Avoid semantic claims that require LSP until Phase 3.

## Phase 3: LSP diagnostics bridge

- Start with TypeScript project diagnostics through a minimal `tsserver` or language-server process bridge.
- Capture diagnostics snapshots without mutating code.
- Store diagnostics in SQLite and compute before/after deltas.
- Add Python/Pyright only after TypeScript diagnostics are stable.

## Phase 4: EvidenceBundle and ProcessReward

- Generate EvidenceBundle JSON with files, symbols, commands, risks, confidence, and missing evidence.
- Implement `preflight` over a patch file using diagnostics delta, scope checks, and impacted-test confidence.
- Keep ProcessReward machine-checkable and conservative.
- Treat missing evidence as a first-class output, not as success.

## Phase 5: optional MCP read-only server

- Add MCP only after CLI/library contracts are useful.
- Expose read-only tools: repo overview, where-to-edit, symbol context, impact analysis, test plan, patch preflight, and memory lookup.
- Do not expose mutation tools, arbitrary shell execution, network calls, or external service dependencies.
- Keep consumer-specific policy outside the kernel.
