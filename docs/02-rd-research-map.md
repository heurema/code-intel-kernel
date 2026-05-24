# 02 — R&D Research Map

This project should use research as inspiration, not as direct implementation dependency.

## P0 inspirations

### Repository Intelligence Graph / RIG

RIG proposes a deterministic architectural map for repository-aware coding agents. It represents buildable components, tests, packages, dependency edges, and coverage edges. The reported evaluation shows higher accuracy and lower completion time when RIG is provided to agents.

Use for:

```text
RepoGraph design
build/test graph
agent-friendly JSON view
evidence-backed repository structure
```

Why it matters:

```text
LSP knows symbols; RIG-like structure knows what builds, tests, and depends on what.
```

### Codebase-Memory / Tree-sitter knowledge graph

Codebase-Memory builds a persistent Tree-sitter-based knowledge graph via MCP and supports call-graph traversal, impact analysis, and community discovery.

Use for:

```text
SymbolGraph
Tree-sitter extraction
graph-native queries
impact analysis
MCP surface design
```

Why it matters:

```text
Tree-sitter gives broad language coverage and cheap syntax structure.
```

### Lanser-CLI / LSP as process reward

Lanser-CLI frames language servers as not only navigation tools but also process-reward providers. Diagnostics, definitions, references, safe apply checks, and diagnostic deltas can guide agents.

Use for:

```text
LSP bridge
ProcessReward
diagnostic_delta
safe patch preflight
stable selectors
```

Why it matters:

```text
Goalrail needs checkable process signals, not only natural-language self-evaluation.
```

## P1 inspirations

### CodeMEM

CodeMEM uses AST-guided adaptive memory for repository-level iterative code generation, with code-centric context and session memory.

Use for:

```text
typed session memory
forgotten hypothesis detection
rejected approach tracking
error recurrence detection
```

### Code Property Graph / Joern / codebadger

Code property graphs combine AST, control-flow, and data-dependency graphs. codebadger exposes Joern CPG through MCP for program slicing, taint tracking, data-flow analysis, and semantic navigation.

Use for:

```text
security-critical rails
taint analysis
patch risk analysis
data-flow checks
```

Do not implement in Milestone 1. Treat as optional analyzer later.

## P2 inspirations

### RepoHyper

RepoHyper uses search-expand-refine over repo-level semantic graphs.

Use for:

```text
retrieval strategy
seed → graph expansion → rerank → compact context bundle
```

### MCP-Zero

MCP-Zero addresses tool overload with proactive toolchain construction and hierarchical tool routing.

Use for:

```text
tool router
limiting tool schemas exposed to agents
different tool profiles for Goalrail and Punk
```

### RepoZero

RepoZero emphasizes execution-based verification for repository-level generation.

Use for:

```text
local benchmark design
execution-based evaluation
agent task suite
```

## Deferred / watchlist

### CodeComp

Structural context compression for agentic coding. Interesting but too deep for initial local CLI/MCP implementation.

Use later for:

```text
context bundle compression
program-structure-aware summarization
```

## Practical synthesis

The kernel should combine:

```text
RIG-lite RepoGraph
+ Tree-sitter SymbolGraph
+ LSP diagnostics/references
+ typed session memory
+ process reward
+ optional MCP read-only tools
```

Do not begin with embeddings. Add embeddings only for docs, comments, issues, or natural-language metadata after structural retrieval works.
