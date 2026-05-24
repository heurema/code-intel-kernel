# Code Intelligence Kernel Bootstrap Kit

Date: 2026-05-24

This archive is a documentation-first bootstrap package for a reusable **Code Intelligence Kernel** that can be used by Goalrail, Punk, and other consumers.

The project is intentionally framed as a reusable module, not a one-off LSP integration and not an embeddings-first code RAG system.

## Recommended directory name

Use:

```bash
code-intel-kernel
```

Rationale: this name is neutral enough to serve multiple consumers while still clearly describing the module. If it later becomes a package, candidates are:

```text
code-intel-kernel
@code-intel/kernel
```

For now, use `code-intel-kernel`.

## What this package contains

```text
README.md
PROMPT_FOR_CODEX.md
docs/
  00-product-brief.md
  01-architecture.md
  02-rd-research-map.md
  03-mvp-roadmap.md
  04-data-model.md
  05-agent-tools.md
  06-goalrail-integration.md
  07-punk-integration.md
  08-metrics-and-benchmarks.md
  09-risks-and-guardrails.md
  10-open-questions.md
  consumers/
    goalrail-profile.md
    punk-profile.md
    example-custom-integration.md
specs/
  domain-model.types.ts
  sqlite-schema.sql
  cli-contract.md
  mcp-tools-contract.md
  evidence-bundle.md
  process-reward.md
prompts/
  01-codex-unpack-and-initialize.md
  02-codex-architecture-review.md
  03-codex-mvp-implementation-plan.md
research/
  references.md
  extraction-notes.md
  papers-to-read.md
examples/
  example-agent-event.json
  example-evidence-bundle.json
  example-process-reward.json
  example-repo-map.json
templates/
  ADR-template.md
  CLAUDE.md.template
  PRD-template.md
config/
  manifest.json
  recommended-directory-name.txt
```

## Strategic decision

**Decision:** Build a reusable Code Intelligence Kernel with project-agnostic core contracts.

**Reason:** Multiple consumers need structured repo understanding, evidence-backed context, process reward, and reusable agent tools.

**Avoid:** project-specific LSP hacks, embeddings-first overbuild, unsafe MCP/tool sprawl, and natural-language-only memory.

## Immediate objective

The first implementation should be a **read-only, structural-first kernel**:

1. RepoGraph: packages, services, commands, tests, build targets.
2. SymbolGraph: Tree-sitter extraction for TypeScript and Python first.
3. LSP bridge: diagnostics, references, definitions, and diagnostic deltas.
4. EvidenceBundle: required evidence before edit planning.
5. ProcessReward: machine-checkable reward signals after diffs.
6. SessionMemory: typed event log for hypotheses, decisions, tests, failures.

## Non-goals for the first milestone

Do **not** build these first:

- Full enterprise code search.
- Full MCP server with mutation tools.
- Embeddings-first code RAG.
- Neo4j or heavy graph infrastructure.
- Full Joern/CPG security analyzer.
- Automatic refactor tools.
- UI/dashboard.

The first milestone should be CLI + local SQLite + optional read-only MCP tool surface.

## First Codex instruction

Use `PROMPT_FOR_CODEX.md` or `prompts/01-codex-unpack-and-initialize.md`.
