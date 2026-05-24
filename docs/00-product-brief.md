# 00 — Product Brief

## Working name

**Code Intelligence Kernel**

## One-sentence description

A local-first, structural code intelligence module that gives AI agents evidence-backed repository understanding, symbol context, impact analysis, test selection, typed session memory, and process-reward signals.

## Why this exists

AI coding consumers need a shared layer that helps agents operate on real codebases without relying only on grep, chat summaries, or embeddings. The module should provide deterministic, inspectable facts about the repository and expose them through CLI, SDK, and later MCP.

## Core thesis

A useful coding-agent layer should be **structural-first**, not embeddings-first.

Priority order:

```text
Repo structure → build/test graph → syntax/symbol graph → LSP facts → snippets → optional semantic search
```

## Example consumers

### Goalrail

Goalrail can use this module as a strict/control-plane consumer:

- require evidence before edits;
- check impact before patching;
- compute diagnostic/test deltas;
- remember failed hypotheses;
- prevent repeated unsafe actions;
- produce auditable agent traces.

### Punk

Punk can use this module as a fast/prototype consumer:

- faster exploration;
- low-friction prototype support;
- lightweight impact analysis;
- session memory;
- fewer hard gates than Goalrail.

## First product boundary

The first version should answer:

1. What is in this repository?
2. Where should an agent look for a task?
3. Which symbols/files are relevant?
4. What tests/lint/build commands are likely affected?
5. Did a proposed diff improve or worsen diagnostics?
6. What did the agent already try in this session?

## Non-goals

- Not a full IDE.
- Not a full Sourcegraph replacement.
- Not an agent by itself.
- Not a vector database project.
- Not a cloud product.
- Not a full MCP integration at the start.
- Not a refactoring engine in Milestone 1.

## Recommended first milestone

A CLI that can run locally:

```bash
code-intel inspect .
code-intel repo-map --json
code-intel where-to-edit "change login validation copy"
code-intel symbol-context "useAuth"
code-intel test-plan --changed-files src/a.ts,src/a.test.ts
code-intel preflight patch.diff
```

## Success criteria for the first milestone

- Runs locally with no external services.
- Produces a useful repo map.
- Extracts TypeScript/Python symbols.
- Stores facts in SQLite.
- Returns evidence bundles with file/symbol/command reasons.
- Computes a basic process reward from diagnostics and scope checks.
- Keeps all memory as typed events, not just natural-language summaries.
