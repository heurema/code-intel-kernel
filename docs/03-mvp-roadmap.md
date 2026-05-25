# 03 — MVP Roadmap

Status: historical bootstrap roadmap, updated through Phase 2E.

Current implementation source of truth:

- `notes/next-implementation-plan.md`
- `docs/phase-2a-symbolgraph-lite-plan.md`
- `docs/repo-vs-symbol-graph-boundary.md`

This document no longer promises SQLite, MCP, TypeScript/Python-first SymbolGraph, or confident `where-to-edit` in early phases. Those items are deferred until the core CLI/API contracts are stable and evaluated.

## Verdict

Build a reusable **Code Intelligence Kernel** as a standalone module.

## Directory name

```text
code-intel-kernel
```

## Roadmap overview

```text
Phase 0: Documentation and repo skeleton
Phase 1: RepoGraph inspect, impact, and eval
Phase 1G: SymbolGraph readiness gate
Phase 2A: SymbolGraph-lite
Phase 2B: SymbolGraph eval and SourceEvidenceBundle contract
Phase 2C: SourceEvidenceBundle prototype
Phase 2D: Source-to-repo evidence linking
Phase 2E: Read-only SourceContext slices
Phase 3: LSP diagnostics bridge
Phase 4: EvidenceBundle and ProcessReward
Phase 5: SessionMemory
Phase 6: Read-only MCP surface
Phase 7: Goalrail strict consumer profile
Phase 8: Punk prototype consumer profile
```

## Phase 0 — Documentation and repo skeleton

### Goal

Create the repository shape and preserve decisions before coding.

### Deliverables

```text
Cargo.toml
src/
tests/
docs/
specs/
notes/
```

### Acceptance criteria

- Repository can run a smoke test.
- Docs are committed.
- Next implementation plan exists.
- No heavy dependencies introduced.

## Phase 1 — RepoGraph inspect, impact, and eval

### Goal

Detect repository structure and commands, compute conservative build/test impact, and measure fixture quality.

### Inputs

- `package.json`
- `pnpm-workspace.yaml`
- `turbo.json`
- `nx.json`
- `tsconfig.json`
- `pyproject.toml`
- `Cargo.toml`
- `go.mod`
- `.github/workflows`
- `Makefile`

### Outputs

```text
inspect JSON
impact JSON
eval JSON
components
commands
workspace boundaries
dependency hints
test command candidates
structured warnings
```

### CLI

```bash
code-intel inspect .
code-intel impact src/main.rs Cargo.toml --json
code-intel eval-fixtures --json
```

### Acceptance criteria

- Finds supported package/workspace facts from manifests.
- Finds common test/lint/build commands.
- Produces evidence-backed inspect JSON.
- Produces conservative RepoGraph-only impact JSON.
- Runs fixture-based RepoGraph evaluation with deterministic output.
- Does not require SQLite, MCP, LSP, or embeddings.

## Phase 2A — SymbolGraph-lite

### Goal

Add the first source-level graph layer without pretending to localize edits.

### Initial scope

```text
Rust top-level source facts first, or a language-agnostic source-file graph stub if that is safer.
```

### Outputs

- files;
- functions;
- structs;
- enums;
- traits;
- impl blocks;
- modules;
- parse/source warnings.

### CLI

```bash
code-intel symbols . --json
```

The `symbols` output has its own contract version and remains separate from `inspect` and `impact`.

### Acceptance criteria

- Existing RepoGraph eval remains green.
- Symbol facts are evidence-backed.
- Symbol IDs are deterministic.
- Parse failures produce warnings, not panics.
- No call graph yet.
- No LSP yet.
- No SQLite yet.
- No MCP yet.
- No embeddings.
- `where-to-edit` remains `insufficient_evidence` unless a separately evaluated localization layer exists.

## Phase 2B — SymbolGraph eval and source-level evidence bundle contract

### Goal

Measure SymbolGraph-lite facts through eval cases and document the SourceEvidenceBundle contract.

### Acceptance criteria

- `eval-fixtures` includes `symbols` cases.
- Source files and symbols remain evidence-backed and deterministic.
- SourceEvidenceBundle is documented but not wired to `where-to-edit`.
- Localization remains `not_ready_for_confident_localization`.
- `where-to-edit` remains `insufficient_evidence`.

## Phase 2C — SourceEvidenceBundle prototype

### Goal

Assemble query, RepoGraph context, SymbolGraph-lite facts, evidence, limitations, and missing evidence into one read-only JSON packet.

### Acceptance criteria

- `source-evidence` CLI returns valid JSON.
- `source_evidence` contract version is `0.2` after Phase 2D context-role hardening.
- Eval includes source-evidence cases.
- Candidate files and symbols are evidence-backed.
- No output claims edit locations.
- `where-to-edit` remains `insufficient_evidence`.

## Phase 2E — SourceContext slices

### Goal

Return bounded read-only source snippets for explicit file or SymbolGraph-lite symbol selectors.

### Acceptance criteria

- `source-context` CLI returns valid JSON.
- `source_context` contract version is `0.1`.
- Eval includes source-context cases and `eval_contract_version` is `0.4`.
- Slices are evidence-backed and deterministic.
- Path traversal, ignored paths, symlinks, missing files, non-UTF8 files, and oversized slices produce structured warnings.
- No natural-language localization.
- No output claims edit locations.
- `where-to-edit` remains `insufficient_evidence`.

## Phase 3 — LSP diagnostics bridge

### Goal

Collect diagnostics and precise symbol facts from language servers.

### Scope

Start with TypeScript project diagnostics via a minimal `tsserver` or language-server adapter. Add Python/Pyright later.

### CLI

```bash
code-intel diagnostics .
code-intel diagnostic-delta --before run1 --after run2
```

### Acceptance criteria

- Captures diagnostics snapshot.
- Computes before/after deltas.
- Does not mutate code.

## Phase 4 — EvidenceBundle and ProcessReward

### Goal

Return evidence-backed outputs for agents and evaluate proposed diffs.

### CLI

```bash
code-intel evidence "change login validation copy"
code-intel preflight patch.diff
```

### Acceptance criteria

- EvidenceBundle includes files, symbols, commands, risks, missing evidence.
- ProcessReward includes score, diagnostic deltas, risk flags, and scope checks.
- Output is JSON serializable.

## Phase 5 — SessionMemory

### Goal

Store typed agent events.

### Events

- task_started;
- evidence_collected;
- hypothesis_created;
- hypothesis_rejected;
- edit_planned;
- patch_preflighted;
- diagnostic_delta_recorded;
- test_run_recorded;
- decision_recorded.

### Acceptance criteria

- Events are append-only JSONL.
- Events are indexed into SQLite.
- Memory lookup can retrieve related decisions and rejected hypotheses.

## Phase 6 — Read-only MCP surface

### Goal

Expose high-level tools to agents.

### Tools

- repo_overview;
- where_to_edit;
- symbol_context;
- impact_analysis;
- test_plan;
- patch_preflight;
- memory_lookup.

### Acceptance criteria

- All MCP tools are read-only.
- No shell execution.
- No arbitrary file writes.
- Tools return compact, structured JSON.

## Phase 7 — Goalrail strict consumer profile

### Goal

Document how Goalrail can use the kernel as a strict/control-plane consumer without putting Goalrail-specific logic into core modules.

### Rails

```text
No edit without evidence bundle.
No patch without impact analysis.
No final answer without verification status.
No repeated failed hypothesis without new evidence.
No expanded tools without policy approval.
```

## Phase 8 — Punk prototype consumer profile

### Goal

Document how Punk can use the same kernel in faster prototype mode without forking parser, memory, or policy contracts.

### Differences from Goalrail

- fewer hard gates;
- more exploration;
- faster context bundles;
- lower strictness;
- same underlying facts and memory format.

## 30-day practical sequence

### Sequence

- Finish Phase 1G readiness and boundary documentation.
- Start Phase 2A only as SymbolGraph-lite.
- Keep LSP, SQLite, MCP, embeddings, and confident edit localization deferred.
- Add source-level eval cases before exposing source-level recommendations to consumers.
