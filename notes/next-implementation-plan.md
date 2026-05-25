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

## Phase 1E-A: RepoGraph extractor quality, Cargo and command files

- Document extractor quality and acceptance criteria before SymbolGraph.
- Improve Cargo target extraction where evidence is cheap and local.
- Improve Makefile/justfile command extraction for clear top-level targets.
- Harden ignored-path handling.
- Add semantic fixture tests for evidence, deterministic output, and impact traversal.

## Phase 1E-B: Python and Go extraction quality

- Add Python and Go fixture coverage.
- Improve conservative Python and Go command inference where manifest/layout evidence exists.
- Keep extraction read-only; do not run ecosystem tooling.
- Keep SymbolGraph deferred unless RepoGraph quality is stable.

## Phase 1F: RepoGraph evaluation harness

- Add lightweight fixture-based scoring for inspect/impact quality.
- Track evidence coverage, warning quality, expected command/test detection, and impact expectations.
- Track false broad versus false narrow impact behavior.
- Use this before deciding whether to start SymbolGraph.

## Phase 1G: RepoGraph decision gate

- Review the eval report before adding source-level intelligence.
- Fix RepoGraph extraction or impact behavior first if eval failures show false narrow, unexpected warnings, or evidence gaps.
- Document acceptable false broad cases before moving on.
- Keep `where-to-edit` as `insufficient_evidence`.
- Capture RepoGraph versus SymbolGraph ownership boundaries.
- Add refusal-oriented eval cases for unsupported or ambiguous behavior.
- Open Phase 2A only when the readiness checklist passes:
  - inspect eval cases pass;
  - impact eval cases pass;
  - false narrow count is zero on core fixtures;
  - false broad cases are documented and acceptable;
  - evidence coverage is 100%;
  - warnings are structured and expected;
  - output is deterministic across repeated runs.

## Phase 2A: SymbolGraph-lite

- Start with a narrow source-level graph.
- Prefer Rust top-level symbol extraction first because the kernel is a Rust crate.
- Extract source files and top-level symbols only.
- Keep every symbol fact evidence-backed and deterministic.
- Add SymbolGraph eval cases before using symbols for localization.
- Do not add call graph, LSP, SQLite, MCP, embeddings, or confident `where-to-edit` yet.

## Phase 2B: SymbolGraph eval and SourceEvidenceBundle contract

- Add SymbolGraph-lite cases to the fixture eval harness.
- Bump eval contract only if eval report shape changes.
- Document SourceEvidenceBundle as a future source-level evidence packet.
- Keep RepoGraph commands/tests as the build/test validation source.
- Keep `where-to-edit` as `insufficient_evidence`.

## Phase 2C: SourceEvidenceBundle prototype

- Add read-only SourceEvidenceBundle generation without edit localization.
- Combine query, SymbolGraph-lite facts, and RepoGraph context only where evidence supports it.
- Add `source-evidence` CLI output with contract version `0.1`, later bumped by Phase 2D.
- Add eval coverage and bump eval contract to `0.3`.
- Return `partial` or `insufficient_evidence` when query-to-symbol relevance is missing.
- Keep candidate files/symbols as context, not edit instructions.

## Phase 2D: Symbol-to-repo context linking

- Link source files/symbols to RepoGraph components by path evidence.
- Add fixture eval for source-to-component relationships.
- Harden SourceEvidenceBundle refusal behavior for ambiguous and no-match queries.
- Add context roles, deterministic candidate limits, and ranking reasons.
- Bump `source_evidence` contract to `0.2`.
- Still avoid confident `where-to-edit` until localization has dedicated negative eval cases.

## Phase 2E: SourceContext slices

- Add explicit-selector read-only source snippets.
- Support file selectors with optional line ranges and SymbolGraph-lite symbol IDs.
- Enforce path containment, ignored-path refusal, symlink refusal, UTF-8 checks, and deterministic truncation.
- Add source-context eval cases and bump eval contract to `0.4`.
- Keep `source_evidence` at `0.2`.
- Keep `where-to-edit` as `insufficient_evidence`.

## Phase 2F: EvidenceBundle and SourceContext integration

- Add selector hints from SourceEvidenceBundle candidates to SourceContext.
- Bump `source_evidence` contract to `0.3`.
- Do not include snippets by default.
- Do not emit edit targets.
- Keep natural-language localization deferred.

## Phase 2G: Localization readiness adversarial gate

- Add adversarial fixtures for ambiguous same-name symbols.
- Add broad-query, ignored-path, malformed-source, and missing-reference cases.
- Prove selector hints stay context handles rather than edit targets.
- Keep `where-to-edit` as `insufficient_evidence`.
- Keep `eval_contract_version` at `0.4` unless the report shape changes.
- Recommended follow-up: Phase 3A LSP diagnostics/reference bridge design if adversarial failures point to missing references/diagnostics; otherwise Phase 2H SymbolGraph-lite hardening if symbol extraction or duplicate-name handling is weak.

## Phase 3A: LSP diagnostics/reference bridge design

- Design the read-only boundary for diagnostics and references before implementation.
- Decide whether the first bridge is Rust-oriented, TypeScript-oriented, or protocol-oriented.
- Define evidence-backed diagnostics/reference contracts and refusal behavior.
- Keep SQLite, MCP, embeddings, mutation tools, and confident `where-to-edit` deferred.
- Produce capability matrix, draft JSON contracts, process safety model, integration boundaries, and Phase 3B plan.

## Phase 3B: LSP diagnostics bridge implementation candidate

- Start with one language/server only after Phase 3A design is accepted.
- Capture diagnostics snapshots without mutating code.
- Keep before/after deltas in memory until storage is justified.
- Add more languages only after one bridge is stable.
- Recommended first candidate: Rust + `rust-analyzer`, read-only diagnostics/definitions/references, structured `unavailable` if the server is missing.

## Phase 3C: LSP eval/adversarial gate

- Add fixture or mocked-response eval coverage for diagnostics, definitions, references, unavailable server behavior, path safety, and deterministic ordering.
- Prove LSP locations remain evidence, not edit targets.
- Keep `where-to-edit` as `insufficient_evidence`.

## Phase 3D: LSP evidence integration into SourceEvidence

- Add LSP diagnostics/reference facts to SourceEvidence only as evidence and missing-evidence resolution.
- Do not embed source snippets by default.
- Do not emit edit targets.
- Keep confident localization deferred until a later readiness gate.

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
