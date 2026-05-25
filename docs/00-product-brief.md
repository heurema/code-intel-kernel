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

## Product boundary

The current stable foundation is RepoGraph: repository/build/test-level `inspect`, `impact`, and `eval-fixtures`.

The current source-level layer is SymbolGraph-lite plus SourceEvidenceBundle evidence assembly:

- Rust top-level `symbols` output;
- read-only `source-evidence` bundle output.

LSP, SQLite, MCP, embeddings, process reward, call graph, references, and confident edit localization are later layers. `where-to-edit` must return `insufficient_evidence` until evaluated localization evidence exists.

## First product boundary

The first version should answer:

1. What is in this repository?
2. What repository/build/test components are evidenced?
3. Which Rust top-level source facts are evidenced?
4. What tests/lint/build commands are likely affected?
5. When must the kernel refuse to guess?
6. What does fixture evaluation say about current quality?

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
code-intel impact src/main.rs Cargo.toml --json
code-intel eval-fixtures --json
code-intel symbols . --json
code-intel where-to-edit "change login validation copy" --profile=strict --json
code-intel source-evidence "parse repo graph" --json
```

## Success criteria for the first milestone

- Runs locally with no external services.
- Produces evidence-backed RepoGraph inspect output.
- Produces conservative RepoGraph-only impact output.
- Measures quality with fixture-based eval.
- Produces evidence-backed Rust top-level SymbolGraph-lite output.
- Produces read-only SourceEvidenceBundle evidence assembly.
- Keeps `where-to-edit` as `insufficient_evidence`.
- Does not require LSP, SQLite, MCP, embeddings, or external services.
