# 03 — MVP Roadmap

## Verdict

Build a reusable **Code Intelligence Kernel** as a standalone module.

## Directory name

```text
code-intel-kernel
```

## Roadmap overview

```text
Phase 0: Documentation and repo skeleton
Phase 1: RepoGraph MVP
Phase 2: SymbolGraph MVP
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

## Phase 1 — RepoGraph MVP

### Goal

Detect repository structure and commands.

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
repo-map.json
packages
commands
workspace boundaries
dependency hints
test command candidates
```

### CLI

```bash
code-intel inspect .
code-intel repo-map --json
code-intel test-plan --changed-files src/a.ts,src/a.test.ts
```

### Acceptance criteria

- Finds packages/workspaces in a simple JS/TS repo.
- Finds common test/lint/build commands.
- Writes nodes and edges to SQLite.
- Produces a compact repo map.

## Phase 2 — SymbolGraph MVP

### Goal

Extract code symbols with Tree-sitter.

### Initial languages

```text
TypeScript
TSX
JavaScript
Python
```

### Outputs

- files;
- imports;
- exports;
- functions;
- classes;
- methods;
- test files;
- simple import edges.

### CLI

```bash
code-intel symbol-context "LoginForm"
code-intel where-to-edit "change login validation copy"
```

### Acceptance criteria

- Extracts symbols from TS/Python examples.
- Maps imports between local files.
- Returns ranked candidate files with reasons.
- Does not require an LSP yet.

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
- Stores diagnostics in SQLite.
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

### Week 1

- Initialize repository.
- Implement RepoGraph scanning.
- Implement SQLite schema.
- Create examples and smoke tests.

### Week 2

- Implement Tree-sitter SymbolGraph for TS/Python.
- Implement where-to-edit heuristic.
- Implement evidence bundle JSON.

### Week 3

- Add diagnostics bridge for TypeScript.
- Implement diagnostic delta.
- Add process reward scoring.

### Week 4

- Add session event memory.
- Add read-only MCP skeleton.
- Create Goalrail/Punk consumer profile notes.
