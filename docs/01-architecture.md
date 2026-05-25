# 01 — Architecture

## Conceptual architecture

```text
Goalrail / Punk / Codex / Claude / other consumer
        ↓
Code Intelligence Kernel
        ↓
RepoGraph now; SymbolGraph-lite + SourceEvidence + SourceContext now; LSP facts + SessionMemory + ProcessReward later
        ↓
CLI / SDK now; optional MCP read-only tools later
        ↓
In-memory now; local SQLite + JSONL event log later
```

## Architectural principles

1. **Structural-first:** Prefer deterministic repository facts before semantic search.
2. **Local-first:** Work on the live local tree; no cloud index required.
3. **Read-only first:** Navigation, analysis, and evidence before mutation.
4. **Typed memory:** Store structured events, not just summaries.
5. **Evidence-backed agent actions:** The kernel should return reasons and confidence.
6. **Small reversible milestones:** Avoid heavy infrastructure until proven necessary.
7. **Different policies, same kernel:** Consumers use the same kernel through generic profiles and external policy mappings.

## Consumer profiles

Core modules must not contain Goalrail-specific or Punk-specific business logic. The kernel should understand generic profiles only:

```text
strict
standard
prototype
research
custom
```

Example mappings belong in consumer or integration code:

```text
Goalrail -> strict
Punk -> prototype
Research agent -> research
Custom project -> custom
```

## Main components

### 1. RepoGraph

RepoGraph maps repository structure:

- packages;
- services;
- apps;
- workspaces;
- build targets;
- test targets;
- lint commands;
- config files;
- generated/vendor/build directories;
- dependency edges.

Initial extractors:

```text
package.json
pnpm-workspace.yaml
yarn.lock / package-lock.json / pnpm-lock.yaml
turbo.json
nx.json
tsconfig.json
pyproject.toml
poetry.lock / uv.lock
Cargo.toml
go.mod
Makefile
Dockerfile
docker-compose.yml
.github/workflows/*.yml
```

### 2. SymbolGraph

SymbolGraph maps source-level structure:

- files;
- modules;
- imports;
- exports;
- functions;
- classes;
- methods;
- types/interfaces;
- tests;
- references only after a separate evaluated reference layer exists.

Current implementation:

```text
Phase 2A: Rust top-level source facts.
Phase 2B: SymbolGraph eval coverage.
Phase 2C-2F: SourceEvidence and SourceContext evidence/context layers.
Phase 2G: adversarial localization readiness gate.
No call graph, LSP runtime, SQLite, MCP, embeddings, or confident edit localization yet.
Later targets can include TypeScript, TSX, JavaScript, Python, Go, Java, Kotlin, and PHP.
```

### 3. LSP bridge

LSP bridge uses language servers for precise facts:

- diagnostics;
- go to definition;
- find references;
- hover/type info;
- implementations;
- call hierarchy where supported.

Phase 3A is design-only. Phase 3B implementation can be minimal:

```text
diagnostics snapshot
definition lookup
references lookup
diagnostic delta before/after
```

### 4. EvidenceBundle

EvidenceBundle is the key future output for agents. It should explain why files/symbols/commands are relevant without turning evidence into edit instructions before localization is evaluated.

Future example claim after a localization-specific gate:

```text
"packages/web/src/auth/LoginForm.tsx is a supported candidate; packages/server/src/auth.ts has lower relevance."
```

Evidence:

```text
- login route renders LoginForm
- LoginForm imports useAuth
- server auth.ts validates tokens, not UI copy
- impacted tests include LoginForm.test.tsx
```

### 5. ProcessReward

ProcessReward is a machine-checkable signal about whether a patch improved or worsened the repository state.

Inputs:

- diagnostic delta;
- affected symbol confidence;
- expected-vs-actual scope;
- test plan confidence;
- risk flags.

### 6. SessionMemory

SessionMemory stores typed agent events:

- task intent;
- selected files;
- selected symbols;
- hypotheses;
- rejected hypotheses;
- edits attempted;
- diagnostics before/after;
- tests run;
- decisions made.

This prevents agents from repeatedly making the same mistake.

## Suggested runtime flow

```text
INTENT
  ↓
SCOPE_REPO
  ↓
COLLECT_EVIDENCE
  ↓
PLAN_EDIT
  ↓
PREFLIGHT_PATCH
  ↓
APPLY_OR_SIMULATE
  ↓
VERIFY
  ↓
RECORD_DECISION
```

## Storage

Start in memory for RepoGraph inspect, impact, and eval.

SQLite, FTS5, and JSONL are later persistence layers, not Phase 1 or Phase 2A requirements.

Do not start with Neo4j or a remote vector DB.

```text
SQLite:
- nodes
- edges
- symbols
- files
- packages
- commands
- diagnostics
- episodes

FTS5:
- docs/comments/snippets

JSONL:
- raw agent events
- traces
- decisions
```

## Interfaces

Milestone 1:

```text
CLI + library API
```

Milestone 2:

```text
SymbolGraph-lite internal API
```

Milestone 3:

```text
LSP diagnostics/reference bridge design, then one narrow read-only implementation candidate
```

Milestone 4:

```text
External project integrations
```
