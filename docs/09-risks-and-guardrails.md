# 09 — Risks and Guardrails

## Main risks

### 1. Embeddings-first overbuild

Risk: building a vector search system before structural retrieval works.

Guardrail:

```text
RepoGraph + SymbolGraph + LSP first.
Embeddings only later for docs/issues/comments.
```

### 2. MCP/tool sprawl

Risk: exposing too many tools or mutation capabilities too early.

Guardrail:

```text
Read-only MCP only in early milestones.
Tool router later.
No arbitrary shell/file writes.
```

### 3. Natural-language-only memory

Risk: storing chat summaries that cannot be queried reliably.

Guardrail:

```text
Typed events + JSONL + SQLite indexes.
Summaries can be derived, not primary.
```

### 4. False precision from Tree-sitter

Risk: syntax graph is mistaken for semantic type-level truth.

Guardrail:

```text
Tree-sitter for broad structure.
LSP for definitions, references, diagnostics, types.
```

### 5. Heavy graph infrastructure too early

Risk: Neo4j/enterprise graph stack slows MVP.

Guardrail:

```text
SQLite first.
Graph DB only after real query pressure.
```

### 6. Mutation before evidence

Risk: agents edit files before knowing correct scope.

Guardrail:

```text
Strict profile requires EvidenceBundle before edits.
```

### 7. Diagnostics treated as full correctness

Risk: no diagnostics does not mean code is correct.

Guardrail:

```text
ProcessReward combines diagnostics, tests, scope, and risk flags.
```

### 8. Cross-project divergence

Risk: Goalrail and Punk implement separate code intelligence.

Guardrail:

```text
One shared kernel, different policies.
```

## Security posture

Milestone 1 should not:

- execute arbitrary shell commands;
- run network requests;
- write files except its own cache/database/logs;
- expose mutation/refactor tools;
- store secrets;
- index `.env`, secrets, or private credentials.

## Ignore patterns

Default ignore candidates:

```text
node_modules/
dist/
build/
.next/
.cache/
coverage/
.venv/
__pycache__/
target/
.git/
.env
.env.*
*.pem
*.key
```

## Decision review cadence

Review every 30 days during R&D, then every 90 days after stable MVP.
